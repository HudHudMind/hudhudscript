//! Test unary operators

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;

#[test]
fn test_unary_plus() {
    let mut interp = Interpreter::new();
    let code = "let a = +10;";
    let ast = parse(code).unwrap();
    interp.eval_program(&ast).unwrap();
    let result = interp.get_variable("a").unwrap();
    assert_eq!(result.to_string(), "10");
}

#[test]
fn test_unary_minus() {
    let mut interp = Interpreter::new();
    let code = "let a = -9;";
    let ast = parse(code).unwrap();
    interp.eval_program(&ast).unwrap();
    let result = interp.get_variable("a").unwrap();
    assert_eq!(result.to_string(), "-9");
}

#[test]
fn test_unary_complex() {
    let mut interp = Interpreter::new();
    let code = "let a = (-9) + (+10);";
    let ast = parse(code).unwrap();
    interp.eval_program(&ast).unwrap();
    let result = interp.get_variable("a").unwrap();
    assert_eq!(result.to_string(), "1");
}

#[test]
fn test_double_negative() {
    let mut interp = Interpreter::new();
    let code = "let a = -(-10);";
    let ast = parse(code).unwrap();
    interp.eval_program(&ast).unwrap();
    let result = interp.get_variable("a").unwrap();
    assert_eq!(result.to_string(), "10");
}

#[test]
fn test_nested_unary() {
    let mut interp = Interpreter::new();
    let code = "let a = -(+9) + -(-10);";
    let ast = parse(code).unwrap();
    interp.eval_program(&ast).unwrap();
    let result = interp.get_variable("a").unwrap();
    assert_eq!(result.to_string(), "1");
}
