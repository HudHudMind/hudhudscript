//! Coverage tests for `crates/hudhudscript-vm/src/vm/builtin_value.rs`.
//!
//! Exercises the value-module builtin dispatchers (Promise combinators, http
//! sandbox gates, file sandbox gates, linalg, stats, env lookup) and the
//! `type_name_of` classification used by method-dispatch error messages.
//! Every assertion pins an exact value or the full distinctive sentence of
//! an error message; runtime errors carry a "Runtime error: " prefix. No
//! network call is ever attempted: the http tests stop at sandbox gates or
//! at method-name parsing, and file tests use temp directories only.
//!
//! Skipped surfaces (see report): `require_string_arg` has no caller in the
//! VM (dead code, unreachable from scripts); real http requests and
//! blocking stdin ops are out of scope by determinism policy.

use hudhudscript_bytecode::{PromiseState16, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::vm::{SandboxConfig, VM};

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
    vm.execute(&bytecode).err().expect("script must fail").message
}

fn err_contains(msg: String, sentence: &str) {
    assert!(msg.contains(sentence), "expected {:?} in: {}", sentence, msg);
}

fn owned(vm: &VM, name: &str) -> Value16 {
    vm.get_variable_owned(name).unwrap_or_else(|| panic!("{} must be published", name))
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("hudhud-cov-val-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir must be created");
    dir
}

fn sandbox(network: bool, read: bool, write: bool) -> SandboxConfig {
    SandboxConfig {
        allowed_paths: vec![],
        allowed_hosts: vec![],
        allow_file_read: read,
        allow_file_write: write,
        allow_network: network,
        allow_process: false,
        allowed_commands: vec![],
        denied_commands: vec![],
    }
}

// ── Promise module ─────────────────────────────────────────────────────

#[test]
fn promise_resolve_wraps_the_argument() {
    let vm = run("let p = Promise.resolve(5)");
    match owned(&vm, "p").as_promise_state() {
        Some(PromiseState16::Resolved(inner)) => assert_eq!(inner.as_int(), Some(5)),
        _ => panic!("expected Resolved(5)"),
    }
}

#[test]
fn promise_resolve_without_argument_resolves_null() {
    let vm = run("let p = Promise.resolve()");
    match owned(&vm, "p").as_promise_state() {
        Some(PromiseState16::Resolved(inner)) => assert!(inner.is_null()),
        _ => panic!("expected Resolved(null)"),
    }
}

#[test]
fn promise_reject_stringifies_its_argument() {
    let vm = run("let a = Promise.reject(\"boom\")\nlet b = Promise.reject(42)");
    match owned(&vm, "a").as_promise_state() {
        Some(PromiseState16::Rejected(msg)) => assert_eq!(msg, "boom"),
        _ => panic!("expected Rejected(boom)"),
    }
    // Non-string values are stringified through value_to_string.
    match owned(&vm, "b").as_promise_state() {
        Some(PromiseState16::Rejected(msg)) => assert_eq!(msg, "42"),
        _ => panic!("expected Rejected(42)"),
    }
}

#[test]
fn promise_all_combines_resolved_promises_and_plain_values() {
    let vm = run("let p = Promise.all([Promise.resolve(1), 2])");
    match owned(&vm, "p").as_promise_state() {
        Some(PromiseState16::Resolved(inner)) => {
            let items = inner.as_array().expect("all() must resolve to an array");
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].as_int(), Some(1));
            assert_eq!(items[1].as_int(), Some(2));
        }
        _ => panic!("expected Resolved(array)"),
    }
}

#[test]
fn promise_all_rejects_when_an_input_rejects() {
    let vm = run("let p = Promise.all([Promise.reject(\"nope\"), 1])");
    match owned(&vm, "p").as_promise_state() {
        Some(PromiseState16::Rejected(msg)) => assert_eq!(msg, "nope"),
        _ => panic!("expected Rejected(nope)"),
    }
}

#[test]
fn promise_all_and_race_require_an_array() {
    err_contains(run_err("let x = Promise.all(5)"), "Promise.all() requires an array");
    err_contains(run_err("let x = Promise.race(\"x\")"), "Promise.race() requires an array");
}

#[test]
fn promise_race_wins_with_the_first_settled_value() {
    let vm = run("let p = Promise.race([7, Promise.resolve(9)])");
    match owned(&vm, "p").as_promise_state() {
        Some(PromiseState16::Resolved(inner)) => assert_eq!(inner.as_int(), Some(7)),
        _ => panic!("expected Resolved(7)"),
    }
}

#[test]
fn promise_race_on_empty_array_rejects() {
    let vm = run("let p = Promise.race([])");
    match owned(&vm, "p").as_promise_state() {
        Some(PromiseState16::Rejected(msg)) => assert_eq!(msg, "Promise.race() on empty array"),
        _ => panic!("expected Rejected"),
    }
}

#[test]
fn promise_unknown_method_is_rejected() {
    err_contains(run_err("let x = Promise.bogus()"), "Unknown Promise method: bogus");
}

// ── http module: sandbox gates only (no request is ever issued) ───────

#[test]
fn http_is_blocked_by_default_sandbox_network_gate() {
    err_contains(run_err("let x = http.get(\"http://example.com/\")"), "Sandbox: network access is not allowed");
}

#[test]
fn allow_network_clears_the_gate_and_reaches_method_parsing() {
    // After allow_network() the sandbox gate no longer fires; the unknown
    // method then fails at HttpMethodId parsing instead. This proves the
    // flag flipped (otherwise the sandbox message would return).
    let bytecode = compile("let x = http.bogus()");
    let mut vm = VM::new();
    vm.allow_network();
    let error = vm.execute(&bytecode).err().expect("unknown method must fail");
    assert_eq!(error.message, "Unknown HTTP method: bogus");
    assert!(!error.message.contains("Sandbox"));
}

#[test]
fn http_host_allowlist_denies_unlisted_hosts() {
    let mut cfg = sandbox(true, true, false);
    cfg.allowed_hosts = vec!["api.example.com".to_string()];
    let bytecode = compile("let x = http.get(\"http://other.example/x\")");
    let mut vm = VM::new();
    vm.with_sandbox(cfg);
    let error = vm.execute(&bytecode).err().expect("unlisted host must fail");
    err_contains(error.message, "Sandbox: host is not in the allowed list for URL 'http://other.example/x'");
}

// ── file module: sandbox gates and reads ──────────────────────────────

#[test]
fn file_read_returns_exact_content_and_exists_is_exact() {
    let dir = temp_dir("read");
    let file = dir.join("f.txt");
    std::fs::write(&file, "exact payload").expect("fixture must be written");
    let source = format!(
        "let text = file.read(\"{f}\")\nlet yes = file.exists(\"{f}\")\nlet no = file.exists(\"{d}/none.txt\")",
        f = file.display(),
        d = dir.display()
    );
    let vm = run(&source);
    assert_eq!(owned(&vm, "text").as_string(), Some("exact payload".to_string()));
    assert_eq!(owned(&vm, "yes").as_bool(), Some(true));
    assert_eq!(owned(&vm, "no").as_bool(), Some(false));
}

#[test]
fn file_list_reports_the_single_entry_of_a_directory() {
    let dir = temp_dir("list");
    std::fs::write(dir.join("solo.txt"), "1").expect("fixture must be written");
    let source = format!("let names = file.list(\"{d}\")", d = dir.display());
    let vm = run(&source);
    let names = owned(&vm, "names");
    let items = names.as_array().expect("list must return an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].as_string(), Some("solo.txt".to_string()));
}

#[test]
fn file_write_and_append_are_denied_by_default_sandbox() {
    // Default sandbox: allow_file_write == false; nothing may be created.
    err_contains(run_err("let x = file.write(\"/tmp/hudhud-cov-val-out.txt\", \"x\")"), "Sandbox: file write access is not allowed");
    err_contains(run_err("let x = file.append(\"/tmp/hudhud-cov-val-out.txt\", \"x\")"), "Sandbox: file write access is not allowed");
    assert!(!std::path::Path::new("/tmp/hudhud-cov-val-out.txt").exists());
}

#[test]
fn file_read_is_denied_when_reads_are_disabled() {
    let bytecode = compile("let x = file.read(\"/etc/hostname\")");
    let mut vm = VM::new();
    vm.with_sandbox(sandbox(true, false, false));
    let error = vm.execute(&bytecode).err().expect("disabled reads must fail");
    err_contains(error.message, "Sandbox: file read access is not allowed");
}

#[test]
fn file_path_allowlist_admits_only_prefixed_paths() {
    let allowed = temp_dir("allow");
    let outside = temp_dir("deny");
    let in_file = allowed.join("in.txt");
    std::fs::write(&in_file, "inside").expect("fixture must be written");
    let out_file = outside.join("out.txt");
    std::fs::write(&out_file, "outside").expect("fixture must be written");

    let mut cfg = sandbox(false, true, false);
    cfg.allowed_paths = vec![allowed.display().to_string()];
    let mut ok_vm = VM::new();
    ok_vm.with_sandbox(cfg.clone());
    let good = compile(&format!("let t = file.read(\"{f}\")", f = in_file.display()));
    ok_vm.execute(&good).expect("allowed path must read");
    assert_eq!(owned(&ok_vm, "t").as_string(), Some("inside".to_string()));

    let mut denied_vm = VM::new();
    denied_vm.with_sandbox(cfg);
    let bad = compile(&format!("let t = file.read(\"{f}\")", f = out_file.display()));
    let error = denied_vm.execute(&bad).err().expect("unlisted path must fail");
    err_contains(error.message, &format!("Sandbox: path '{}' is not in the allowed paths list", out_file.display()));
}

#[test]
fn file_unknown_method_is_rejected() {
    assert_eq!(run_err("let x = file.bogus()"), "Unknown file method: bogus");
}

// ── stats module ───────────────────────────────────────────────────────

#[test]
fn stats_mean_and_median_are_exact() {
    let vm = run(
        "let a = stats.mean([1, 2, 3])\nlet b = stats.mean([1, 2, 3, 4])\nlet c = stats.median([3, 1, 2])\nlet d = stats.median([4, 1, 3, 2])",
    );
    assert_eq!(owned(&vm, "a").as_number(), Some(2.0));
    assert_eq!(owned(&vm, "b").as_number(), Some(2.5));
    assert_eq!(owned(&vm, "c").as_number(), Some(2.0));
    assert_eq!(owned(&vm, "d").as_number(), Some(2.5));
}

#[test]
fn stats_variance_std_dev_min_max_are_exact() {
    let vm = run(
        "let v = stats.variance([1, 2, 3, 4, 5])\nlet s = stats.std_dev([1, 2, 3, 4, 5])\nlet lo = stats.min([4, -2, 9])\nlet hi = stats.max([4, -2, 9])",
    );
    assert_eq!(owned(&vm, "v").as_number(), Some(2.0));
    let sd = owned(&vm, "s").as_number().expect("std_dev must be a number");
    assert!((sd - 2.0f64.sqrt()).abs() < 1e-12, "std_dev was {}", sd);
    assert_eq!(owned(&vm, "lo").as_number(), Some(-2.0));
    assert_eq!(owned(&vm, "hi").as_number(), Some(9.0));
}

#[test]
fn stats_quantile_and_distributions_are_exact() {
    let vm = run(
        "let q = stats.quantile([10, 20, 30, 40], 0.5)\nlet p = stats.normal_pdf(0, 0, 1)\nlet u = stats.uniform_pdf(0.5, 0, 1)\nlet o = stats.uniform_pdf(2, 0, 1)\nlet c = stats.uniform_cdf(0.25, 0, 1)",
    );
    // quantile: idx = round(0.5 * 3) = 2 -> third element of sorted array.
    assert_eq!(owned(&vm, "q").as_number(), Some(30.0));
    let pdf = owned(&vm, "p").as_number().expect("pdf must be a number");
    assert!((pdf - 0.398_942_280_401_432_7).abs() < 1e-12, "pdf was {}", pdf);
    assert_eq!(owned(&vm, "u").as_number(), Some(1.0));
    assert_eq!(owned(&vm, "o").as_number(), Some(0.0));
    assert_eq!(owned(&vm, "c").as_number(), Some(0.25));
}

#[test]
fn stats_error_paths_report_exact_messages() {
    assert_eq!(run_err("let x = stats.bogus([1])"), "Unknown stats method: bogus");
    assert_eq!(run_err("let x = stats.mean(\"nope\")"), "stats: expected array");
    assert_eq!(run_err("let x = stats.mean([1, \"x\"])"), "stats: expected number in array");
    assert_eq!(run_err("let x = stats.mean()"), "stats.mean() expects 1 argument");
    assert_eq!(run_err("let x = stats.quantile([1, 2])"), "stats.quantile() expects 2 arguments");
}

// ── linalg module ──────────────────────────────────────────────────────

#[test]
fn linalg_dot_norm_and_cross_are_exact() {
    let vm = run(
        "let d = linalg.dot([1, 2, 3], [4, 5, 6])\nlet n = linalg.norm([3, 4])\nlet c = linalg.cross([1, 0, 0], [0, 1, 0])",
    );
    assert_eq!(owned(&vm, "d").as_number(), Some(32.0));
    assert_eq!(owned(&vm, "n").as_number(), Some(5.0));
    let cross = owned(&vm, "c");
    let items = cross.as_array().expect("cross must return an array");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].as_number(), Some(0.0));
    assert_eq!(items[1].as_number(), Some(0.0));
    assert_eq!(items[2].as_number(), Some(1.0));
}

#[test]
fn linalg_normalize_transpose_determinant_identity_are_exact() {
    let vm = run(
        "let u = linalg.normalize([3, 0, 4])\nlet t = linalg.transpose([[1, 2], [3, 4]])\nlet d2 = linalg.determinant([[3, 8], [4, 6]])\nlet d3 = linalg.determinant([[2, 0, 0], [0, 3, 0], [0, 0, 4]])\nlet i = linalg.identity(2)",
    );
    let u = owned(&vm, "u").as_array().expect("normalize must return an array").clone();
    assert_eq!(u[0].as_number(), Some(0.6));
    assert_eq!(u[1].as_number(), Some(0.0));
    assert_eq!(u[2].as_number(), Some(0.8));

    let transposed = owned(&vm, "t");
    let rows = transposed.as_array().expect("transpose must return a matrix");
    assert_eq!(rows[0].as_array().expect("row 0")[1].as_number(), Some(3.0));
    assert_eq!(rows[1].as_array().expect("row 1")[0].as_number(), Some(2.0));

    assert_eq!(owned(&vm, "d2").as_number(), Some(-14.0));
    assert_eq!(owned(&vm, "d3").as_number(), Some(24.0));

    let ident = owned(&vm, "i");
    let identity = ident.as_array().expect("identity must return a matrix");
    assert_eq!(identity.len(), 2);
    assert_eq!(identity[0].as_array().expect("row 0")[0].as_number(), Some(1.0));
    assert_eq!(identity[0].as_array().expect("row 0")[1].as_number(), Some(0.0));
    assert_eq!(identity[1].as_array().expect("row 1")[1].as_number(), Some(1.0));
}

#[test]
fn linalg_error_paths_report_exact_messages() {
    assert_eq!(run_err("let x = linalg.bogus([1])"), "Unknown linalg method: bogus");
    assert_eq!(run_err("let x = linalg.dot([1, 2])"), "linalg.dot() expects 2 arguments");
    assert_eq!(run_err("let x = linalg.dot([1, 2], [1, 2, 3])"), "linalg.dot: vectors must have same length");
    assert_eq!(run_err("let x = linalg.cross([1, 2], [3, 4])"), "linalg.cross: vectors must be 3D");
    assert_eq!(run_err("let x = linalg.normalize([0, 0])"), "linalg.normalize: zero vector");
    assert_eq!(run_err("let x = linalg.determinant([[1, 2, 3], [4, 5, 6]])"), "linalg.determinant: requires square matrix");
    assert_eq!(
        run_err("let x = linalg.determinant([[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]])"),
        "linalg.determinant: only implemented for 1x1, 2x2, and 3x3 matrices"
    );
}

// ── env lookup ─────────────────────────────────────────────────────────

#[test]
fn env_reads_process_environment_and_missing_keys_are_null() {
    // Unique key: no other test (and no .env file) can define it.
    let key = "HUDHUD_COV_VALUE_PROBE_9187";
    std::env::set_var(key, "deterministic");
    let vm = run(
        "let hit = env(\"HUDHUD_COV_VALUE_PROBE_9187\")\nlet miss = env(\"HUDHUD_COV_VALUE_MISSING_6261\")",
    );
    assert_eq!(owned(&vm, "hit").as_string(), Some("deterministic".to_string()));
    assert!(owned(&vm, "miss").is_null());
    std::env::remove_var(key);
}

#[test]
fn env_validates_argument_shape() {
    err_contains(run_err("let x = env(123)"), "env() requires a string key");
    err_contains(run_err("let x = env()"), "env() expects 1 argument, got 0");
}

// ── type_name_of via method-dispatch errors ────────────────────────────

#[test]
fn method_call_on_number_names_the_type() {
    err_contains(run_err("let n = 5\nlet x = n.bogus()"), "Cannot call method 'bogus' on number");
}

#[test]
fn method_call_on_boolean_and_null_name_their_types() {
    err_contains(run_err("let b = true\nlet x = b.bogus()"), "Cannot call method 'bogus' on boolean");
    err_contains(run_err("let z = None\nlet x = z.bogus()"), "Cannot call method 'bogus' on null");
}
