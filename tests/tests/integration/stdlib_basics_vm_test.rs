//! VM parity tests for v0.4.38 Batch 2: Stdlib Basics

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::{Compiler, VM};
use hudhudscript_parser::parse;

fn vm_run_and_get(code: &str, var: &str) -> hudhudscript_bytecode::Value16 {
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

fn assert_string(val: hudhudscript_bytecode::Value16, expected: &str) {
    if let Some(s) = val.as_str() {
        assert_eq!(s, expected, "Expected '{}', got '{}'", expected, s);
    } else {
        panic!("Expected String(\"{}\"), got {:?}", expected, val);
    }
}

fn assert_bool(val: hudhudscript_bytecode::Value16, expected: bool) {
    if let Some(b) = val.as_bool() {
        assert_eq!(b, expected, "Expected {}, got {}", expected, b);
    } else {
        panic!("Expected Boolean({}), got {:?}", expected, val);
    }
}

fn assert_number(val: hudhudscript_bytecode::Value16, expected: f64) {
    if let Some(n) = val.as_number() {
        assert!(
            (n - expected).abs() < 0.001,
            "Expected {}, got {}",
            expected,
            n
        );
    } else {
        panic!("Expected Number({}), got {:?}", expected, val);
    }
}

// ─── sleep/delay (#655) ─────────────────────────────────────────────────────

#[test]
fn test_vm_sleep_zero() {
    let val = vm_run_and_get("var x = sleep(0);", "x");
    assert!(val.is_null());
}

#[test]
fn test_vm_delay_zero() {
    let val = vm_run_and_get("var x = delay(0);", "x");
    assert!(val.is_null());
}

// ─── Path (#663) ────────────────────────────────────────────────────────────

#[test]
fn test_vm_path_join() {
    assert_string(
        vm_run_and_get(r#"var x = Path.join("/home", "user", "file.txt");"#, "x"),
        "/home/user/file.txt",
    );
}

#[test]
fn test_vm_path_parent() {
    assert_string(
        vm_run_and_get(r#"var x = Path.parent("/home/user/file.txt");"#, "x"),
        "/home/user",
    );
}

#[test]
fn test_vm_path_filename() {
    assert_string(
        vm_run_and_get(r#"var x = Path.filename("/home/user/file.txt");"#, "x"),
        "file.txt",
    );
}

#[test]
fn test_vm_path_extension() {
    assert_string(
        vm_run_and_get(r#"var x = Path.extension("/home/user/file.txt");"#, "x"),
        "txt",
    );
}

#[test]
fn test_vm_path_is_absolute() {
    assert_bool(
        vm_run_and_get(r#"var x = Path.isAbsolute("/home/user");"#, "x"),
        true,
    );
    assert_bool(
        vm_run_and_get(r#"var x = Path.isAbsolute("relative/path");"#, "x"),
        false,
    );
}

// ─── Temp (#664) ────────────────────────────────────────────────────────────

#[test]
fn test_vm_temp_file() {
    let val = vm_run_and_get("var f = Temp.file(); var x = f.path;", "x");
    if let Some(s) = val.as_str() {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string path");
    }
}

#[test]
fn test_vm_temp_dir() {
    let val = vm_run_and_get("var d = Temp.dir(); var x = d.path;", "x");
    if let Some(s) = val.as_str() {
        assert!(!s.is_empty());
    } else {
        panic!("Expected string path");
    }
}

// ─── URLParser (#665) ───────────────────────────────────────────────────────

#[test]
fn test_vm_url_parser_parse_scheme() {
    assert_string(
        vm_run_and_get(
            r#"var u = URLParser.parse("https://example.com:8080/path?q=1#frag"); var x = u.scheme;"#,
            "x",
        ),
        "https",
    );
}

#[test]
fn test_vm_url_parser_parse_host() {
    assert_string(
        vm_run_and_get(
            r#"var u = URLParser.parse("https://example.com:8080/path"); var x = u.host;"#,
            "x",
        ),
        "example.com",
    );
}

#[test]
fn test_vm_url_parser_parse_port() {
    assert_number(
        vm_run_and_get(
            r#"var u = URLParser.parse("https://example.com:8080/path"); var x = u.port;"#,
            "x",
        ),
        8080.0,
    );
}

// ─── Glob (#666) ────────────────────────────────────────────────────────────

#[test]
fn test_vm_glob_match_true() {
    assert_bool(
        vm_run_and_get(r#"var x = Glob.match("hello.txt", "*.txt");"#, "x"),
        true,
    );
}

#[test]
fn test_vm_glob_match_false() {
    assert_bool(
        vm_run_and_get(r#"var x = Glob.match("hello.rs", "*.txt");"#, "x"),
        false,
    );
}

// ─── String.padStart/padEnd (#670) ──────────────────────────────────────────

#[test]
fn test_vm_pad_start_default() {
    assert_string(vm_run_and_get(r#"var x = "42".padStart(5);"#, "x"), "   42");
}

#[test]
fn test_vm_pad_start_custom_char() {
    assert_string(
        vm_run_and_get(r#"var x = "42".padStart(5, "0");"#, "x"),
        "00042",
    );
}

#[test]
fn test_vm_pad_end_default() {
    assert_string(vm_run_and_get(r#"var x = "hi".padEnd(5);"#, "x"), "hi   ");
}

#[test]
fn test_vm_pad_end_custom_char() {
    assert_string(
        vm_run_and_get(r#"var x = "hi".padEnd(5, ".");"#, "x"),
        "hi...",
    );
}

#[test]
fn test_vm_pad_start_no_change() {
    assert_string(
        vm_run_and_get(r#"var x = "hello".padStart(3);"#, "x"),
        "hello",
    );
}

#[test]
fn test_vm_pad_end_no_change() {
    assert_string(
        vm_run_and_get(r#"var x = "hello".padEnd(3);"#, "x"),
        "hello",
    );
}
