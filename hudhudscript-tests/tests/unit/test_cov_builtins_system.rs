//! Coverage tests for `crates/hudhudscript-vm/src/vm/builtin_system.rs`.
//!
//! Exercises the system-module builtin dispatchers (tokenomics, channel,
//! Terminal, stdin, log, exec, tcp/udp/ws, unix sockets, daemon, fs) through
//! compiled scripts. Every assertion pins an exact value or the full
//! distinctive sentence of an exact error message. VM-raised system errors
//! travel inside the `Runtime error: ` transport prefix (see
//! `assert_error_contains` below). Only deterministic behavior is tested:
//! no clocks, no randomness, no network, no child processes; filesystem
//! work stays in per-test temp directories under the system temp dir.

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::vm::host_access::HostAccessPolicy;
use hudhudscript_vm::vm::VM;

// ── helpers ────────────────────────────────────────────────────────────

fn compile(source: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(source).expect("source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("source must compile")
}

fn run(source: &str) -> VM {
    let bytecode = compile(source);
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("script must execute");
    vm
}

fn run_err(source: &str) -> String {
    let bytecode = compile(source);
    let mut vm = VM::new();
    vm.execute(&bytecode)
        .err()
        .expect("script must fail")
        .message
}

/// System builtin errors travel through `compile_codes::runtime_error`,
/// which bakes a `Runtime error: ` transport prefix into the stored message
/// (the exec dispatcher adds an `[E0038] ...` render on top). Assert on the
/// full distinctive sentence inside the transport wrapper.
fn assert_error_contains(source: &str, sentence: &str) {
    let msg = run_err(source);
    assert!(msg.contains(sentence), "unexpected error: {}", msg);
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("hudhud-cov-sys-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir must be created");
    dir
}

fn owned(vm: &VM, name: &str) -> Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
}

// ── tokenomics module ──────────────────────────────────────────────────

#[test]
fn tokenomics_session_cost_is_zero_and_budget_health_is_one() {
    let vm = run("let c = tokenomics.session_cost()\nlet h = tokenomics.budget_health()");
    assert_eq!(owned(&vm, "c").as_number(), Some(0.0));
    assert_eq!(owned(&vm, "h").as_number(), Some(1.0));
}

#[test]
fn tokenomics_usage_reports_zero_daily_and_monthly() {
    let vm = run("let u = tokenomics.usage()");
    let u = owned(&vm, "u");
    let obj = u.as_object().expect("usage() must return an object");
    assert_eq!(obj.get("daily").expect("daily field").as_int(), Some(0));
    assert_eq!(obj.get("monthly").expect("monthly field").as_int(), Some(0));
}

#[test]
fn tokenomics_unknown_method_is_rejected() {
    assert_error_contains("let x = tokenomics.bogus()", "Unknown tokenomics method: bogus");
}

// ── channel module ─────────────────────────────────────────────────────

#[test]
fn channel_send_and_notify_return_null_for_valid_string_args() {
    let vm = run("let s = channel.send(\"news\", \"hello\")\nlet n = channel.notify(\"ping\")");
    assert!(owned(&vm, "s").is_null());
    assert!(owned(&vm, "n").is_null());
}

#[test]
fn channel_send_validates_argument_count_and_types() {
    assert_error_contains("let x = channel.send(\"news\")", "channel.send() requires at least 2 arguments: channel_name, text");
    assert_error_contains("let x = channel.send(7, \"hello\")", "channel.send() first argument must be a string (channel name)");
    assert_error_contains("let x = channel.send(\"news\", 7)", "channel.send() second argument must be a string (message text)");
}

#[test]
fn channel_notify_validates_argument_count_and_type() {
    assert_error_contains("let x = channel.notify()", "channel.notify() requires at least 1 argument: text");
    assert_error_contains("let x = channel.notify(9)", "channel.notify() first argument must be a string (message text)");
}

#[test]
fn channel_unknown_method_is_rejected() {
    assert_error_contains("let x = channel.bogus()", "Unknown channel method: bogus");
}

// ── Terminal module ────────────────────────────────────────────────────

#[test]
fn terminal_color_helpers_wrap_text_in_exact_ansi_codes() {
    let vm = run("let r = Terminal.red(\"err\")\nlet b = Terminal.bold(\"hi\")");
    assert_eq!(
        owned(&vm, "r").as_string(),
        Some("\u{1b}[31merr\u{1b}[0m".to_string())
    );
    assert_eq!(
        owned(&vm, "b").as_string(),
        Some("\u{1b}[1mhi\u{1b}[0m".to_string())
    );
}

#[test]
fn terminal_strip_removes_ansi_sequences() {
    // strip() of red()'s exact output must yield the plain text; this also
    // cross-checks the two implementations against each other.
    let vm = run("let colored = Terminal.red(\"go\")\nlet s = Terminal.strip(colored)");
    assert_eq!(owned(&vm, "s").as_string(), Some("go".to_string()));
}

#[test]
fn terminal_color_requires_a_string_argument() {
    assert_eq!(
        run_err("let x = Terminal.red()"),
        "terminal.red requires a string argument"
    );
}

#[test]
fn terminal_unknown_method_is_rejected() {
    assert_eq!(
        run_err("let x = Terminal.bogus()"),
        "Unknown Terminal method: bogus"
    );
}

// ── stdin module ───────────────────────────────────────────────────────

#[test]
fn stdin_unknown_method_is_rejected() {
    assert_eq!(run_err("let x = stdin.bogus()"), "Unknown stdin method: bogus");
}

// ── log module ─────────────────────────────────────────────────────────

#[test]
fn log_info_and_error_build_structured_entries() {
    let vm = run("let i = log.info(\"deploy ok\")\nlet e = log.error(\"disk full\")");
    let i = owned(&vm, "i");
    let entry = i.as_object().expect("log.info() must return an object");
    assert_eq!(entry.get("level").expect("level").as_string(), Some("info".to_string()));
    assert_eq!(
        entry.get("message").expect("message").as_string(),
        Some("deploy ok".to_string())
    );
    // No structured data argument was passed, so no "data" field exists.
    assert!(entry.get("data").is_none());

    let e = owned(&vm, "e");
    let err_entry = e.as_object().expect("log.error() must return an object");
    assert_eq!(err_entry.get("level").expect("level").as_string(), Some("error".to_string()));
    assert_eq!(
        err_entry.get("message").expect("message").as_string(),
        Some("disk full".to_string())
    );
}

#[test]
fn log_setlevel_and_level_roundtrip_and_validation() {
    // Owns the process-global log level for the duration of this test; no
    // other test in this binary touches setLevel, so this is deterministic.
    let vm = run("let ok = log.setLevel(\"warn\")\nlet lvl = log.level()");
    assert_eq!(owned(&vm, "ok").as_bool(), Some(true));
    assert_eq!(owned(&vm, "lvl").as_string(), Some("warn".to_string()));
    // An invalid level is rejected with the exact allowed set, and the
    // invalid value must not have overwritten the level set above.
    assert_eq!(
        run_err("let x = log.setLevel(\"nope\")"),
        "Invalid log level: 'nope'. Valid: debug, info, warn, error, trace"
    );
    let after = run("let lvl = log.level()");
    assert_eq!(owned(&after, "lvl").as_string(), Some("warn".to_string()));
}

#[test]
fn log_unknown_method_is_rejected() {
    assert_eq!(run_err("let x = log.bogus()"), "Unknown log method: bogus");
}

// ── exec module ────────────────────────────────────────────────────────

#[test]
fn exec_module_denied_by_restrictive_host_policy() {
    let bytecode = compile("let x = exec.run(\"echo hi\")");
    let mut vm = VM::new();
    vm.set_host_access_policy(HostAccessPolicy::restrictive());
    let error = vm.execute(&bytecode).err().expect("restrictive policy must block exec");
    assert_eq!(
        error.message,
        "Host access denied: module 'exec' is not allowed by host_access policy"
    );
}

#[test]
fn exec_denied_command_is_blocked_before_spawn() {
    let bytecode = compile("let x = exec.run(\"rm -rf /tmp/hudhud-cov-never\")");
    let mut vm = VM::new();
    let mut policy = HostAccessPolicy::permissive();
    policy.exec.deny.insert("rm".to_string());
    vm.set_host_access_policy(policy);
    let error = vm.execute(&bytecode).err().expect("denied command must fail");
    assert_eq!(
        error.message,
        "Host access denied: command 'rm' is not allowed by host_access policy"
    );
    // The guard fired before any child process could be spawned.
    assert!(!std::path::Path::new("/tmp/hudhud-cov-never").exists());
}

#[test]
fn exec_run_without_command_argument_errors() {
    // parse_cmd fails in the VM's exec dispatcher and the error is re-wrapped
    // via compile_codes::runtime_error(e.to_string()), whose Display render
    // carries the E0038 title on top of the transport prefix.
    assert_error_contains("let x = exec.run()", "exec: command argument required");
}

#[test]
fn exec_unknown_method_is_rejected() {
    assert_eq!(run_err("let x = exec.bogus()"), "Unknown exec method: bogus");
}

// ── tcp / udp / ws modules ─────────────────────────────────────────────

#[test]
fn net_modules_reject_unknown_methods() {
    assert_eq!(run_err("let x = tcp.bogus()"), "Unknown tcp method: bogus");
    assert_eq!(run_err("let x = udp.bogus()"), "Unknown udp method: bogus");
    assert_eq!(run_err("let x = ws.bogus()"), "Unknown ws method: bogus");
}

// ── unix socket module ─────────────────────────────────────────────────

#[test]
fn unix_fd_methods_validate_connection_object() {
    // No connection object at all.
    assert_error_contains("let x = unix.write()", "unix: expected connection object");
    assert_error_contains("let x = unix.read()", "unix: expected connection object");
    assert_error_contains("let x = unix.close()", "unix: expected connection object");
    // A receiver that is not an object cannot carry an "fd" field.
    assert_error_contains("let x = unix.write(5, \"data\")", "unix: missing fd");
}

#[test]
fn unix_connect_and_http_validate_path_strings() {
    assert_error_contains("let x = unix.connect(42)", "unix.connect: expected path string");
    assert_error_contains("let x = unix.http(42)", "unix.http: expected socket path");
}

#[test]
fn unix_unknown_method_is_rejected() {
    assert_error_contains("let x = unix.bogus()", "Unknown unix method: bogus");
}

// ── daemon module ──────────────────────────────────────────────────────

#[test]
fn daemon_unknown_method_is_rejected() {
    assert_eq!(run_err("let x = daemon.bogus()"), "Unknown daemon method: bogus");
}

// ── fs module (temp-dir backed) ────────────────────────────────────────

#[test]
fn fs_write_read_exists_roundtrip_in_temp_dir() {
    let dir = temp_dir("rw");
    let file = dir.join("note.txt");
    let source = format!(
        "let made = fs.mkdir_p(\"{d}\")\nlet wrote = fs.write(\"{f}\", \"hello\")\nlet text = fs.read(\"{f}\")\nlet present = fs.exists(\"{f}\")\nlet absent = fs.exists(\"{d}/missing.txt\")",
        d = dir.display(),
        f = file.display()
    );
    let vm = run(&source);
    assert!(owned(&vm, "made").is_null());
    assert_eq!(owned(&vm, "wrote").as_bool(), Some(true));
    assert_eq!(owned(&vm, "text").as_string(), Some("hello".to_string()));
    assert_eq!(owned(&vm, "present").as_bool(), Some(true));
    assert_eq!(owned(&vm, "absent").as_bool(), Some(false));
    // The write really hit the filesystem.
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "hello");
}

#[test]
fn fs_stat_reports_exact_size_and_kind() {
    let dir = temp_dir("stat");
    let file = dir.join("data.bin");
    std::fs::write(&file, b"12345").expect("fixture must be written");
    let source = format!(
        "let s = fs.stat(\"{f}\")\nlet d = fs.stat(\"{dir}\")",
        f = file.display(),
        dir = dir.display()
    );
    let vm = run(&source);
    let s = owned(&vm, "s");
    let stat = s.as_object().expect("fs.stat() must return an object");
    assert_eq!(stat.get("size").expect("size").as_number(), Some(5.0));
    assert_eq!(stat.get("is_file").expect("is_file").as_bool(), Some(true));
    assert_eq!(stat.get("is_dir").expect("is_dir").as_bool(), Some(false));

    let d = owned(&vm, "d");
    let dir_stat = d.as_object().expect("directory stat must return an object");
    assert_eq!(dir_stat.get("is_dir").expect("is_dir").as_bool(), Some(true));
    assert_eq!(dir_stat.get("is_file").expect("is_file").as_bool(), Some(false));
}

#[test]
fn fs_copy_counts_bytes_and_rename_moves_the_file() {
    let dir = temp_dir("cpmv");
    let src = dir.join("a.txt");
    std::fs::write(&src, "abcde").expect("fixture must be written");
    let dst = dir.join("b.txt");
    let source = format!(
        "let n = fs.copy(\"{s}\", \"{t}\")\nlet moved = fs.rename(\"{t}\", \"{u}\")",
        s = src.display(),
        t = dst.display(),
        u = dir.join("c.txt").display()
    );
    let vm = run(&source);
    assert_eq!(owned(&vm, "n").as_number(), Some(5.0));
    assert!(owned(&vm, "moved").is_null());
    assert!(!dst.exists(), "renamed file must no longer exist at source");
    assert_eq!(std::fs::read_to_string(dir.join("c.txt")).unwrap(), "abcde");
}

#[test]
fn fs_symlink_readlink_and_realpath_are_exact() {
    let dir = temp_dir("link");
    let target = dir.join("t.txt");
    std::fs::write(&target, "T").expect("fixture must be written");
    let link = dir.join("l.txt");
    let source = format!(
        "let made = fs.symlink(\"{t}\", \"{l}\")\nlet back = fs.readlink(\"{l}\")\nlet real = fs.realpath(\"{t}\")",
        t = target.display(),
        l = link.display()
    );
    let vm = run(&source);
    assert!(owned(&vm, "made").is_null());
    assert_eq!(owned(&vm, "back").as_string(), Some(target.display().to_string()));
    // realpath must agree with the OS canonicalizer (independent oracle).
    let canonical = std::fs::canonicalize(&target).expect("target must canonicalize");
    assert_eq!(owned(&vm, "real").as_string(), Some(canonical.display().to_string()));
}

#[test]
fn fs_watch_reports_path_and_existence() {
    let dir = temp_dir("watch");
    let file = dir.join("w.txt");
    std::fs::write(&file, "xyz").expect("fixture must be written");
    let source = format!("let w = fs.watch(\"{f}\")", f = file.display());
    let vm = run(&source);
    let w = owned(&vm, "w");
    let watch = w.as_object().expect("fs.watch() must return an object");
    assert_eq!(watch.get("path").expect("path").as_string(), Some(file.display().to_string()));
    assert_eq!(watch.get("exists").expect("exists").as_bool(), Some(true));
    assert_eq!(watch.get("size").expect("size").as_number(), Some(3.0));
}

#[test]
fn fs_unknown_method_is_rejected() {
    assert_eq!(run_err("let x = fs.bogus()"), "Unknown fs method: bogus");
}
