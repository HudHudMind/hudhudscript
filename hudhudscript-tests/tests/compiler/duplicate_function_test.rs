//! FUNCTION0004: Duplicate function detection tests.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn compile_expect_err(src: &str) -> String {
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    match c.compile(&ast) {
        Err(e) => format!("{}", e),
        Ok(_) => panic!("Expected compilation error"),
    }
}

#[test]
fn test_duplicate_toplevel_function_errors() {
    let err = compile_expect_err("function test() { return 1; }\nfunction test() { return 2; }");
    assert!(err.contains("already defined"), "got: {}", err);
    assert!(
        err.contains("line 1"),
        "should mention original definition line 1, got: {}",
        err
    );
}

#[test]
fn test_unique_functions_ok() {
    let src = "function a() { return 1; }\nfunction b() { return 2; }";
    let ast = parse(src).unwrap();
    let mut c = Compiler::new();
    c.compile(&ast).expect("unique functions should compile");
}

#[test]
fn test_error_message_not_unsupported_feature() {
    // ISSUE-1: error should NOT say "unsupported feature"
    let err = compile_expect_err("function x() {}\nfunction x() {}");
    assert!(
        !err.contains("unsupported feature"),
        "should not say unsupported: {}",
        err
    );
    assert!(
        err.contains("Duplicate") || err.contains("already defined"),
        "got: {}",
        err
    );
}
