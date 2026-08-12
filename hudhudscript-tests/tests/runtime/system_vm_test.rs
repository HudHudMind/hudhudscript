//! Batch 9: Runtime & System VM tests

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn vm_run(code: &str, var: &str) -> Value16 {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("VM execution failed");
    vm.get_variable(var)
        .cloned()
        .map(|v| v)
        .unwrap_or_else(|| panic!("variable '{}' not found", var))
}

fn vm_run_ok(code: &str) {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("VM execution failed");
}

fn assert_string(val: Value16, expected: &str) {
    if let Some(s) = val.as_str() {
        assert_eq!(s, expected)
    } else {
        panic!("Expected String(\"{}\"), got {:?}", expected, val)
    }
}

fn assert_bool(val: Value16, expected: bool) {
    if let Some(b) = val.as_bool() {
        assert_eq!(b, expected)
    } else {
        panic!("Expected Boolean({}), got {:?}", expected, val)
    }
}

fn assert_number(val: Value16, expected: f64) {
    if let Some(n) = val.as_number() {
        assert!(
            (n - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            n
        )
    } else {
        panic!("Expected Number({}), got {:?}", expected, val)
    }
}

fn assert_number_positive(val: Value16) {
    if let Some(n) = val.as_number() {
        assert!(n > 0.0, "Expected positive, got {}", n)
    } else {
        panic!("Expected Number > 0, got {:?}", val)
    }
}

fn assert_obj_has_string(val: &Value16, key: &str, expected: &str) {
    if let Some(obj) = val.as_object() {
        match obj.get(key).and_then(|v| v.as_str()) {
            Some(s) => assert_eq!(
                s, expected,
                "key '{}': expected '{}', got '{}'",
                key, expected, s
            ),
            other => panic!("key '{}': expected String, got {:?}", key, other),
        }
    } else {
        panic!("Expected Object, got {:?}", val)
    }
}

fn assert_obj_has_bool(val: &Value16, key: &str, expected: bool) {
    if let Some(obj) = val.as_object() {
        match obj.get(key).and_then(|v| v.as_bool()) {
            Some(b) => assert_eq!(b, expected),
            other => panic!("key '{}': expected Boolean, got {:?}", key, other),
        }
    } else {
        panic!("Expected Object, got {:?}", val)
    }
}

fn assert_obj_has_number(val: &Value16, key: &str, expected: f64) {
    if let Some(obj) = val.as_object() {
        match obj.get(key).and_then(|v| v.as_number()) {
            Some(n) => assert!(
                (n - expected).abs() < 0.001,
                "key '{}': expected {}, got {}",
                key,
                expected,
                n
            ),
            other => panic!("key '{}': expected Number, got {:?}", key, other),
        }
    } else {
        panic!("Expected Object, got {:?}", val)
    }
}

fn assert_array_len(val: &Value16, expected_len: usize) {
    if let Some(arr) = val.as_array() {
        assert_eq!(
            arr.len(),
            expected_len,
            "Expected array len {}, got {}",
            expected_len,
            arr.len()
        )
    } else {
        panic!("Expected Array, got {:?}", val)
    }
}

// ── Daemon (#596) ──────────────────────────────────────────────────────

#[test]
fn test_vm_daemon_pid() {
    assert_number_positive(vm_run(r#"var r = daemon.pid();"#, "r"));
}

#[test]
fn test_vm_daemon_is_running() {
    assert_bool(
        vm_run(
            r#"var pid = daemon.pid(); var r = daemon.is_running(pid);"#,
            "r",
        ),
        true,
    );
}

// ── Timer/Scheduler (#618) ─────────────────────────────────────────────

#[test]
#[ignore]
fn test_vm_set_interval() {
    let val = vm_run(r#"var r = setInterval(null, 100);"#, "r");
    assert_obj_has_string(&val, "type", "interval");
    assert_obj_has_bool(&val, "active", true);
}

#[test]
fn test_vm_clear_timeout() {
    vm_run_ok(r#"clearTimeout(1);"#);
}

#[test]
fn test_vm_schedule_parse_cron() {
    let val = vm_run(r#"var r = schedule.parse_cron("*/5 * * * *");"#, "r");
    assert_obj_has_string(&val, "minute", "*/5");
    assert_obj_has_string(&val, "hour", "*");
}

#[test]
fn test_vm_schedule_cron() {
    let val = vm_run(r#"var r = schedule.cron("0 9 * * 1-5");"#, "r");
    assert_obj_has_string(&val, "type", "cron");
    assert_obj_has_bool(&val, "active", true);
}

// ── Filesystem (#604) ──────────────────────────────────────────────────

#[test]
fn test_vm_fs_stat() {
    let val = vm_run(r#"var r = fs.stat("/tmp");"#, "r");
    assert_obj_has_bool(&val, "is_dir", true);
}

#[test]
fn test_vm_fs_watch() {
    let val = vm_run(r#"var r = fs.watch("/tmp");"#, "r");
    assert_obj_has_bool(&val, "exists", true);
}

#[test]
fn test_vm_fs_mkdir_p() {
    let dir = tempfile::TempDir::new().unwrap();
    let deep = dir.path().join("vm_a/vm_b/vm_c");
    let src = format!(r#"fs.mkdir_p("{}");"#, deep.display());
    vm_run_ok(&src);
    assert!(deep.exists());
}

// ── Environment & OS (#622) ────────────────────────────────────────────

#[test]
fn test_vm_env_set_get() {
    use hudhudscript_vm::host_access::HostAccessPolicy;
    let ast =
        parse(r#"Env.set("_VM_B9_", "vmval"); var r = Env.get("_VM_B9_");"#).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    let mut policy = HostAccessPolicy::permissive();
    policy.env.allow.insert("_VM_B9_".to_string());
    vm.set_host_access_policy(policy);
    vm.execute(&bytecode).expect("VM execution failed");
    let r = vm
        .get_variable("r")
        .cloned()
        .expect("variable 'r' not found");
    assert_string(r, "vmval");
    std::env::remove_var("_VM_B9_");
}

#[test]
fn test_vm_env_has() {
    assert_bool(vm_run(r#"var r = Env.has("HOME");"#, "r"), true);
}

#[test]
fn test_vm_os_name() {
    assert_string(vm_run(r#"var r = os.name();"#, "r"), "linux");
}

#[test]
fn test_vm_os_pid() {
    assert_number_positive(vm_run(r#"var r = os.pid();"#, "r"));
}

#[test]
fn test_vm_os_cpus() {
    let val = vm_run(r#"var r = os.cpus();"#, "r");
    if let Some(n) = val.as_number() {
        assert!(n >= 1.0);
    } else {
        panic!("Expected number >= 1, got {:?}", val)
    }
}

#[test]
fn test_vm_os_hostname() {
    let val = vm_run(r#"var r = os.hostname();"#, "r");
    if let Some(s) = val.as_str() {
        assert!(!s.is_empty());
    } else {
        panic!("Expected non-empty string, got {:?}", val)
    }
}

// ── Date/Time (#593) ───────────────────────────────────────────────────

#[test]
fn test_vm_date_now() {
    let val = vm_run(r#"var r = Date.now();"#, "r");
    if let Some(n) = val.as_number() {
        assert!(n > 1_700_000_000.0);
    } else {
        panic!("Expected number > 1.7B, got {:?}", val)
    }
}

#[test]
fn test_vm_date_format() {
    assert_string(
        vm_run(r#"var r = Date.format(0, "%Y-%m-%d");"#, "r"),
        "1970-01-01",
    );
}

#[test]
fn test_vm_date_from_timestamp() {
    let val = vm_run(r#"var r = Date.from_timestamp(0);"#, "r");
    assert_obj_has_number(&val, "year", 1970.0);
    assert_obj_has_number(&val, "month", 1.0);
    assert_obj_has_number(&val, "day", 1.0);
}

#[test]
fn test_vm_date_diff() {
    assert_number(vm_run(r#"var r = Date.diff(3600, 0, "hours");"#, "r"), 1.0);
}

#[test]
fn test_vm_date_add() {
    assert_number(vm_run(r#"var r = Date.add(0, 1, "days");"#, "r"), 86400.0);
}

#[test]
fn test_vm_date_iso() {
    let val = vm_run(r#"var r = Date.iso(0);"#, "r");
    if let Some(s) = val.as_str() {
        assert!(
            s.starts_with("1970-01-01"),
            "Expected 1970-01-01..., got {}",
            s
        );
    } else {
        panic!("Expected string, got {:?}", val)
    }
}

#[test]
fn test_vm_duration_seconds() {
    let val = vm_run(r#"var r = Duration.seconds(30);"#, "r");
    assert_obj_has_number(&val, "seconds", 30.0);
    assert_obj_has_number(&val, "millis", 30000.0);
}

#[test]
fn test_vm_duration_days() {
    let val = vm_run(r#"var r = Duration.days(1);"#, "r");
    assert_obj_has_number(&val, "seconds", 86400.0);
}

// ── Regex (#592) ───────────────────────────────────────────────────────

#[test]
fn test_vm_regex_test() {
    assert_bool(
        vm_run(r#"var r = regex.test("\\d+", "abc123");"#, "r"),
        true,
    );
}

#[test]
fn test_vm_regex_test_no_match() {
    assert_bool(
        vm_run(r#"var r = regex.test("\\d+", "abcdef");"#, "r"),
        false,
    );
}

#[test]
fn test_vm_regex_match() {
    let val = vm_run(r#"var r = regex.match("(\\d+)", "abc123def");"#, "r");
    assert_obj_has_bool(&val, "matched", true);
    assert_obj_has_string(&val, "value", "123");
}

#[test]
fn test_vm_regex_find_all() {
    let val = vm_run(r#"var r = regex.find_all("\\d+", "a1b22c333");"#, "r");
    assert_array_len(&val, 3);
}

#[test]
fn test_vm_regex_replace_all() {
    assert_string(
        vm_run(r#"var r = regex.replace_all("\\d+", "a1b2c3", "N");"#, "r"),
        "aNbNcN",
    );
}

#[test]
fn test_vm_regex_split() {
    let val = vm_run(r#"var r = regex.split("[,;]", "a,b;c,d");"#, "r");
    assert_array_len(&val, 4);
}

#[test]
fn test_vm_regex_escape() {
    let val = vm_run(r#"var r = regex.escape("a.b+c");"#, "r");
    if let Some(s) = val.as_str() {
        assert!(s.contains(r"\."));
    } else {
        panic!("Expected string with \\., got {:?}", val)
    }
}

#[test]
fn test_vm_string_match() {
    let val = vm_run(r#"var r = "hello world 42".match("\\d+");"#, "r");
    assert_obj_has_string(&val, "value", "42");
}

#[test]
fn test_vm_string_match_all() {
    let val = vm_run(r#"var r = "a1b2c3".matchAll("\\d");"#, "r");
    assert_array_len(&val, 3);
}
