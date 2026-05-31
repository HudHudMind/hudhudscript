//! Batch 9: Runtime & System tests
//! Tests for daemon, timer, fs, env/os, datetime, regex, and process management.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(src: &str) -> hudhudscript_bytecode::Value16 {
    let ast = parse(src).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
    // Return last defined variable
    interp
        .get_variable("result")
        .unwrap_or(hudhudscript_bytecode::Value16::null())
}

fn run_ok(src: &str) {
    let ast = parse(src).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
}

// ── Daemon (#596) ──────────────────────────────────────────────────────

#[test]
fn test_daemon_pid() {
    let src = r#"var result = daemon.pid();"#;
    let val = run(src);
    if let Some(n) = val.as_number() {
        assert!(n > 0.0);
    } else {
        panic!("Expected number, got {:?}", val);
    }
}

#[test]
fn test_daemon_is_running() {
    let src = r#"
        var pid = daemon.pid();
        var result = daemon.is_running(pid);
    "#;
    let val = run(src);
    assert_eq!(val, hudhudscript_bytecode::Value16::boolean(true));
}

// ── Process management (#606) ──────────────────────────────────────────

#[test]
fn test_exec_run() {
    let src = r#"var result = exec.run("echo hello");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("success"),
            Some(&hudhudscript_bytecode::Value16::boolean(true))
        );
        if let Some(stdout) = obj.get("stdout").and_then(|v| v.as_str()) {
            assert!(stdout.trim() == "hello");
        }
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_exec_lines() {
    let src = r#"var result = exec.lines("echo -e 'line1\nline2'");"#;
    let val = run(src);
    if let Some(arr) = val.as_array() {
        assert!(arr.len() >= 1);
    } else {
        panic!("Expected array")
    }
}

// ── Timer/Scheduler (#618) ─────────────────────────────────────────────

#[test]
fn test_set_interval_returns_descriptor() {
    let src = r#"var result = setInterval(null, 100);"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("type"),
            Some(&hudhudscript_bytecode::Value16::string(
                "interval".to_string()
            ))
        );
        assert_eq!(
            obj.get("active"),
            Some(&hudhudscript_bytecode::Value16::boolean(true))
        );
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_clear_timeout() {
    run_ok(r#"clearTimeout(1);"#);
}

#[test]
fn test_schedule_parse_cron() {
    let src = r#"var result = schedule.parse_cron("*/5 * * * *");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("minute"),
            Some(&hudhudscript_bytecode::Value16::string("*/5".to_string()))
        );
    } else {
        panic!("Expected object")
    }
}

// ── Filesystem (#604) ──────────────────────────────────────────────────

#[test]
fn test_fs_stat() {
    let src = r#"var result = fs.stat("/tmp");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("is_dir"),
            Some(&hudhudscript_bytecode::Value16::boolean(true))
        );
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_fs_mkdir_p() {
    let dir = tempfile::TempDir::new().unwrap();
    let deep = dir.path().join("a/b/c");
    let src = format!(r#"fs.mkdir_p("{}");"#, deep.display());
    run_ok(&src);
    assert!(deep.exists());
}

#[test]
fn test_fs_symlink_readlink() {
    let dir = tempfile::TempDir::new().unwrap();
    let target = dir.path().join("target.txt");
    let link = dir.path().join("link.txt");
    std::fs::write(&target, "hello").unwrap();

    let src = format!(
        r#"
        fs.symlink("{}", "{}");
        var result = fs.readlink("{}");
        "#,
        target.display(),
        link.display(),
        link.display()
    );
    let val = run(&src);
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string(target.to_str().unwrap().to_string())
    );
}

#[test]
fn test_fs_watch() {
    let src = r#"var result = fs.watch("/tmp");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("exists"),
            Some(&hudhudscript_bytecode::Value16::boolean(true))
        );
    } else {
        panic!("Expected object")
    }
}

// ── Environment & OS (#622) ────────────────────────────────────────────

#[test]
fn test_env_set_get() {
    let src = r#"
        Env.set("_HUDHUD_B9_TEST_", "batch9value");
        var result = Env.get("_HUDHUD_B9_TEST_");
    "#;
    let val = run(src);
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("batch9value".to_string())
    );
    std::env::remove_var("_HUDHUD_B9_TEST_");
}

#[test]
fn test_env_has() {
    let src = r#"var result = Env.has("HOME");"#;
    let val = run(src);
    assert_eq!(val, hudhudscript_bytecode::Value16::boolean(true));
}

#[test]
fn test_os_name() {
    let src = r#"var result = os.name();"#;
    let val = run(src);
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("linux".to_string())
    );
}

#[test]
fn test_os_arch() {
    let src = r#"var result = os.arch();"#;
    let val = run(src);
    if let Some(s) = val.as_str() {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_os_hostname() {
    let src = r#"var result = os.hostname();"#;
    let val = run(src);
    if let Some(s) = val.as_str() {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_os_pid() {
    let src = r#"var result = os.pid();"#;
    let val = run(src);
    if let Some(n) = val.as_number() {
        assert!(n > 0.0);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_os_cpus() {
    let src = r#"var result = os.cpus();"#;
    let val = run(src);
    if let Some(n) = val.as_number() {
        assert!(n >= 1.0);
    } else {
        panic!("Expected number");
    }
}

// ── Date/Time (#593) ───────────────────────────────────────────────────

#[test]
fn test_date_now() {
    let src = r#"var result = Date.now();"#;
    let val = run(src);
    if let Some(n) = val.as_number() {
        assert!(n > 1_700_000_000.0)
    } else {
        panic!("Expected number")
    }
}

#[test]
fn test_date_parse_iso() {
    let src = r#"var result = Date.parse("2024-01-15T10:30:00+00:00");"#;
    let val = run(src);
    if let Some(n) = val.as_number() {
        assert!(n > 0.0);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_date_format() {
    let src = r#"var result = Date.format(0, "%Y-%m-%d");"#;
    let val = run(src);
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("1970-01-01".to_string())
    );
}

#[test]
fn test_date_from_timestamp() {
    let src = r#"var result = Date.from_timestamp(0);"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("year"),
            Some(&hudhudscript_bytecode::Value16::number(1970.0))
        );
    } else {
        panic!("Expected object")
    }
}

#[test]
fn test_date_diff() {
    let src = r#"var result = Date.diff(3600, 0, "hours");"#;
    let val = run(src);
    assert_eq!(val, hudhudscript_bytecode::Value16::number(1.0));
}

#[test]
fn test_date_add() {
    let src = r#"var result = Date.add(0, 1, "days");"#;
    let val = run(src);
    assert_eq!(val, hudhudscript_bytecode::Value16::number(86400.0));
}

#[test]
fn test_date_iso() {
    let src = r#"var result = Date.iso(0);"#;
    let val = run(src);
    if let Some(s) = val.as_str() {
        assert!(s.starts_with("1970-01-01"));
    } else {
        panic!("Expected string")
    }
}

#[test]
fn test_duration_seconds() {
    let src = r#"var result = Duration.seconds(30);"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("seconds"),
            Some(&hudhudscript_bytecode::Value16::number(30.0))
        );
        assert_eq!(
            obj.get("millis"),
            Some(&hudhudscript_bytecode::Value16::number(30000.0))
        );
    } else {
        panic!("Expected object")
    }
}

// ── Regex (#592) ───────────────────────────────────────────────────────

#[test]
fn test_regex_test() {
    let src = r#"var result = regex.test("\\d+", "abc123");"#;
    let val = run(src);
    assert_eq!(val, hudhudscript_bytecode::Value16::boolean(true));
}

#[test]
fn test_regex_match() {
    let src = r#"var result = regex.match("(\\d+)", "abc123def");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("matched"),
            Some(&hudhudscript_bytecode::Value16::boolean(true))
        );
        assert_eq!(
            obj.get("value"),
            Some(&hudhudscript_bytecode::Value16::string("123".to_string()))
        );
    } else {
        panic!("Expected object, got {:?}", val)
    }
}

#[test]
fn test_regex_find_all() {
    let src = r#"var result = regex.find_all("\\d+", "a1b22c333");"#;
    let val = run(src);
    if let Some(arr) = val.as_array() {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array")
    }
}

#[test]
fn test_regex_replace_all() {
    let src = r#"var result = regex.replace_all("\\d+", "a1b2c3", "N");"#;
    let val = run(src);
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("aNbNcN".to_string())
    );
}

#[test]
fn test_regex_split() {
    let src = r#"var result = regex.split("[,;]", "a,b;c,d");"#;
    let val = run(src);
    if let Some(arr) = val.as_array() {
        assert_eq!(arr.len(), 4);
    } else {
        panic!("Expected array")
    }
}

#[test]
fn test_string_match_method() {
    let src = r#"var result = "hello world 42".match("\\d+");"#;
    let val = run(src);
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("value"),
            Some(&hudhudscript_bytecode::Value16::string("42".to_string()))
        );
    } else {
        panic!("Expected object, got {:?}", val)
    }
}

#[test]
fn test_string_match_all_method() {
    let src = r#"var result = "a1b2c3".matchAll("\\d");"#;
    let val = run(src);
    if let Some(arr) = val.as_array() {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array")
    }
}

#[test]
fn test_regex_escape() {
    let src = r#"var result = regex.escape("a.b+c");"#;
    let val = run(src);
    if let Some(s) = val.as_str() {
        assert!(s.contains(r"\."));
    } else {
        panic!("Expected string")
    }
}
