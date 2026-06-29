//! P5-A1: BigInt add conversion regression tests.
//! Lock: BigInt+Int, Int+BigInt, BigInt+BigInt, overflow, Number error.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(src: &str) -> hudhudscript_bytecode::Value16 {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    vm.get_global("r")
        .unwrap_or(hudhudscript_bytecode::Value16::null())
}

fn run_err(src: &str) -> bool {
    let ast = parse(src).unwrap();
    let mut compiler = Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).is_err()
}

#[test]
fn p5_bigint_plus_int() {
    let src = "let a = 9223372036854775807 + 1; let r = a + 5; r;";
    let val = run(src);
    assert_eq!(
        val.as_bigint().map(|b| b.to_string()),
        Some("9223372036854775813".to_string())
    );
}

#[test]
fn p5_int_plus_bigint() {
    let src = "let a = 9223372036854775807 + 1; let r = 5 + a; r;";
    let val = run(src);
    assert_eq!(
        val.as_bigint().map(|b| b.to_string()),
        Some("9223372036854775813".to_string())
    );
}

#[test]
fn p5_bigint_plus_bigint() {
    let src = "let a = 9223372036854775807 + 1; let b = 9223372036854775807 + 2; let r = a + b; r;";
    let val = run(src);
    assert_eq!(
        val.as_bigint().map(|b| b.to_string()),
        Some("18446744073709551617".to_string())
    );
}

#[test]
fn p5_int_overflow_bigint() {
    let src = "let r = 9223372036854775807 + 1; r;";
    let val = run(src);
    assert_eq!(
        val.as_bigint().map(|b| b.to_string()),
        Some("9223372036854775808".to_string())
    );
}

#[test]
fn p5_bigint_plus_number_error() {
    let src = "let a = 9223372036854775807 + 1; let r = a + 0.5;";
    assert!(run_err(src), "BigInt + Number should error");
}
