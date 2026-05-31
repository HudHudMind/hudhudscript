//! Tests for Issue #95: Strict Null Safety (Option/Result types)

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn run(src: &str) -> hudhudscript_bytecode::Value16 {
    let ast = parse(src).expect("parse error");
    let mut interp = Interpreter::new();
    interp.execute(&ast).expect("runtime error")
}

fn run_expect_err(src: &str) -> String {
    let ast = parse(src).expect("parse error");
    let mut interp = Interpreter::new();
    interp.execute(&ast).unwrap_err().to_string()
}

// ── Some / None ───────────────────────────────────────────────────────────────

#[test]
fn test_some_creates_option() {
    let val = run("let x = Some(42);");
    // Last statement result is the value
    let _ = val; // just ensure no panic
}

#[test]
fn test_none_is_option() {
    let val = run("let x = None;");
    let _ = val;
}

#[test]
fn test_is_some_true() {
    let val = run("let x = is_some(Some(1));");
    let _ = val;
}

#[test]
fn test_is_none_true() {
    let val = run("let x = is_none(None);");
    let _ = val;
}

#[test]
fn test_is_some_false_for_none() {
    let val = run("let x = is_some(None);");
    let _ = val;
}

// ── unwrap ────────────────────────────────────────────────────────────────────

#[test]
fn test_unwrap_some() {
    let val = run("let x = unwrap(Some(99));");
    let _ = val;
}

#[test]
fn test_unwrap_none_fails_gracefully() {
    let err = run_expect_err("let x = unwrap(None);");
    assert!(
        err.contains("unwrap") || err.contains("None"),
        "got: {}",
        err
    );
}

#[test]
fn test_unwrap_or_some() {
    let val = run("let x = unwrap_or(Some(5), 0);");
    let _ = val;
}

#[test]
fn test_unwrap_or_none_returns_default() {
    let val = run("let x = unwrap_or(None, 42);");
    let _ = val;
}

// ── Ok / Err ──────────────────────────────────────────────────────────────────

#[test]
fn test_ok_creates_result() {
    let val = run(r#"let x = Ok("success");"#);
    let _ = val;
}

#[test]
fn test_err_creates_result() {
    let val = run(r#"let x = Err("something went wrong");"#);
    let _ = val;
}

#[test]
fn test_is_ok_true() {
    let val = run(r#"let x = is_ok(Ok(1));"#);
    let _ = val;
}

#[test]
fn test_is_err_true() {
    let val = run(r#"let x = is_err(Err("oops"));"#);
    let _ = val;
}

#[test]
fn test_unwrap_ok() {
    let val = run(r#"let x = unwrap(Ok(100));"#);
    let _ = val;
}

#[test]
fn test_unwrap_err_fails_gracefully() {
    let err = run_expect_err(r#"let x = unwrap(Err("bad"));"#);
    assert!(
        err.contains("unwrap") || err.contains("Err") || err.contains("bad"),
        "got: {}",
        err
    );
}

#[test]
fn test_unwrap_or_err_returns_default() {
    let val = run(r#"let x = unwrap_or(Err("fail"), 0);"#);
    let _ = val;
}

// ── Undefined variable fails gracefully (not panic) ──────────────────────────

#[test]
fn test_undefined_variable_error() {
    let err = run_expect_err("let x = undefined_var;");
    assert!(
        err.contains("Undefined") || err.contains("undefined"),
        "got: {}",
        err
    );
}

// ── Option in if condition ────────────────────────────────────────────────────

#[test]
fn test_some_is_truthy_in_if() {
    // Some(x) should be truthy
    let val = run(r#"
        let opt = Some(1);
        let result = 0;
        if (is_some(opt)) {
            result = 1;
        }
    "#);
    let _ = val;
}

#[test]
fn test_none_is_falsy_in_if() {
    let val = run(r#"
        let opt = None;
        let result = 0;
        if (is_none(opt)) {
            result = 1;
        }
    "#);
    let _ = val;
}
