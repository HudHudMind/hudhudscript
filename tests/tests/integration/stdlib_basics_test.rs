//! Integration tests for v0.4.38 Batch 2: Stdlib Basics
//!
//! Tests sleep/delay, Path, Temp, URLParser, Glob, and String.padStart/padEnd.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(code: &str) -> Interpreter {
    let ast = parse(code).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.eval_program(&ast).expect("execution failed");
    interp
}

fn run_and_get(code: &str, var: &str) -> hudhudscript_bytecode::Value16 {
    let interp = run(code);
    interp.get_variable(var).expect("variable not found")
}

// ─── sleep/delay (#655) ─────────────────────────────────────────────────────

#[test]
fn test_sleep_zero() {
    let val = run_and_get("var x = sleep(0);", "x");
    assert_eq!(val, hudhudscript_bytecode::Value16::null());
}

#[test]
fn test_delay_zero() {
    let val = run_and_get("var x = delay(0);", "x");
    assert_eq!(val, hudhudscript_bytecode::Value16::null());
}

#[test]
fn test_sleep_small_duration() {
    let start = std::time::Instant::now();
    run("sleep(10);");
    assert!(start.elapsed().as_millis() >= 9);
}

// ─── Path (#663) ────────────────────────────────────────────────────────────

#[test]
fn test_path_join() {
    let val = run_and_get(r#"var p = Path.join("/home", "user", "file.txt");"#, "p");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("/home/user/file.txt".to_string())
    );
}

#[test]
fn test_path_parent() {
    let val = run_and_get(r#"var p = Path.parent("/home/user/file.txt");"#, "p");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("/home/user".to_string())
    );
}

#[test]
fn test_path_dirname_alias() {
    let val = run_and_get(r#"var p = Path.dirname("/home/user/file.txt");"#, "p");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("/home/user".to_string())
    );
}

#[test]
fn test_path_filename() {
    let val = run_and_get(r#"var f = Path.filename("/home/user/file.txt");"#, "f");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("file.txt".to_string())
    );
}

#[test]
fn test_path_basename_alias() {
    let val = run_and_get(r#"var f = Path.basename("/home/user/file.txt");"#, "f");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("file.txt".to_string())
    );
}

#[test]
fn test_path_extension() {
    let val = run_and_get(r#"var e = Path.extension("/home/user/file.txt");"#, "e");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("txt".to_string())
    );
}

#[test]
fn test_path_is_absolute() {
    let interp = run(r#"
        var a = Path.isAbsolute("/home/user");
        var b = Path.isAbsolute("relative/path");
    "#);
    assert_eq!(
        interp.get_variable("a").unwrap(),
        hudhudscript_bytecode::Value16::boolean(true)
    );
    assert_eq!(
        interp.get_variable("b").unwrap(),
        hudhudscript_bytecode::Value16::boolean(false)
    );
}

// ─── Temp (#664) ────────────────────────────────────────────────────────────

#[test]
fn test_temp_file() {
    let val = run_and_get("var f = Temp.file(); var p = f.path;", "p");
    if let Some(s) = val.as_str() {
        assert!(!s.is_empty(), "Temp file path should not be empty");
    } else {
        panic!("Expected string path");
    }
}

#[test]
fn test_temp_dir() {
    let val = run_and_get("var d = Temp.dir(); var p = d.path;", "p");
    if let Some(s) = val.as_str() {
        assert!(
            std::path::Path::new(&s).exists(),
            "Temp dir should exist: {}",
            s
        );
    } else {
        panic!("Expected string path");
    }
}

#[test]
fn test_temp_path() {
    let val = run_and_get(r#"var p = Temp.path("test");"#, "p");
    if let Some(s) = val.as_str() {
        assert!(s.contains("test"), "Temp path should contain prefix");
    } else {
        panic!("Expected string");
    }
}

// ─── URLParser (#665) ───────────────────────────────────────────────────────

#[test]
fn test_url_parser_parse() {
    let code = r#"
        var u = URLParser.parse("https://example.com:8080/path?q=1#frag");
        var scheme = u.scheme;
        var host = u.host;
        var port = u.port;
        var path = u.path;
        var query = u.query;
        var fragment = u.fragment;
    "#;
    let interp = run(code);
    assert_eq!(
        interp.get_variable("scheme").unwrap(),
        hudhudscript_bytecode::Value16::string("https".to_string())
    );
    assert_eq!(
        interp.get_variable("host").unwrap(),
        hudhudscript_bytecode::Value16::string("example.com".to_string())
    );
    assert_eq!(
        interp.get_variable("port").unwrap(),
        hudhudscript_bytecode::Value16::number(8080.0)
    );
    assert_eq!(
        interp.get_variable("path").unwrap(),
        hudhudscript_bytecode::Value16::string("/path".to_string())
    );
    assert_eq!(
        interp.get_variable("query").unwrap(),
        hudhudscript_bytecode::Value16::string("q=1".to_string())
    );
    assert_eq!(
        interp.get_variable("fragment").unwrap(),
        hudhudscript_bytecode::Value16::string("frag".to_string())
    );
}

#[test]
fn test_url_parser_format() {
    let code = r#"
        var u = URLParser.parse("https://example.com/api");
        var s = URLParser.format(u);
    "#;
    let val = run_and_get(code, "s");
    if let Some(s) = val.as_str() {
        assert!(s.contains("https://"));
        assert!(s.contains("example.com"));
    } else {
        panic!("Expected string");
    }
}

// ─── Glob (#666) ────────────────────────────────────────────────────────────

#[test]
fn test_glob_match_true() {
    let val = run_and_get(r#"var m = Glob.match("hello.txt", "*.txt");"#, "m");
    assert_eq!(val, hudhudscript_bytecode::Value16::boolean(true));
}

#[test]
fn test_glob_match_false() {
    let val = run_and_get(r#"var m = Glob.match("hello.rs", "*.txt");"#, "m");
    assert_eq!(val, hudhudscript_bytecode::Value16::boolean(false));
}

#[test]
fn test_glob_find_returns_array() {
    let val = run_and_get(r#"var files = Glob.find("Cargo.toml");"#, "files");
    if let Some(_) = val.as_array() {
        // ok — array returned
    } else {
        panic!("Expected array");
    }
}

// ─── String.padStart/padEnd (#670) ──────────────────────────────────────────

#[test]
fn test_pad_start_default() {
    let val = run_and_get(r#"var s = "42".padStart(5);"#, "s");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("   42".to_string())
    );
}

#[test]
fn test_pad_start_custom_char() {
    let val = run_and_get(r#"var s = "42".padStart(5, "0");"#, "s");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("00042".to_string())
    );
}

#[test]
fn test_pad_end_default() {
    let val = run_and_get(r#"var s = "hi".padEnd(5);"#, "s");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("hi   ".to_string())
    );
}

#[test]
fn test_pad_end_custom_char() {
    let val = run_and_get(r#"var s = "hi".padEnd(5, ".");"#, "s");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("hi...".to_string())
    );
}

#[test]
fn test_pad_start_no_change_when_longer() {
    let val = run_and_get(r#"var s = "hello".padStart(3);"#, "s");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("hello".to_string())
    );
}

#[test]
fn test_pad_end_no_change_when_longer() {
    let val = run_and_get(r#"var s = "hello".padEnd(3);"#, "s");
    assert_eq!(
        val,
        hudhudscript_bytecode::Value16::string("hello".to_string())
    );
}
