//! A1: int_div / int_mod panic fix regression tests.
//! Verifies i64::MIN / -1 and i64::MIN % -1 don't panic.

use hudhudscript_bytecode::Value16;
use hudhudscript_vm::VM;

fn run_and_get(src: &str) -> Value16 {
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    vm.execute(&bc).unwrap();
    vm.last_return_value()
}

fn eval(src: &str) -> Value16 {
    run_and_get(&format!("fn __t() {{ {} }} __t()", src))
}

fn eval_raw(src: &str) -> Value16 {
    run_and_get(src)
}

#[test]
fn test_int_min_div_neg1_no_panic() {
    // Test that the VM doesn't panic on i64::MIN / -1.
    // Use runtime arithmetic to get i64::MIN: let x = 9223372036854775807 + 1 produces 9223372036854775808 (BigInt at add)
    // So we need values that stay Int. Let's test via direct checked_div.
    // The important thing: the div paths use checked_div, no panic.
    let (ta, pa) = (hudhudscript_bytecode::ReprTag::Int, i64::MIN as u64);
    let (tb, pb) = (hudhudscript_bytecode::ReprTag::Int, (-1i64) as u64);
    let a = Value16(hudhudscript_bytecode::Repr::new_inline(ta, pa));
    let b = Value16(hudhudscript_bytecode::Repr::new_inline(tb, pb));
    let result = hudhudscript_vm::vm::bigint_arith::int_div(a, b).unwrap();
    assert!(result.is_bigint(), "i64::MIN / -1 should be BigInt");
    assert_eq!(
        result.as_bigint().unwrap().to_string(),
        "9223372036854775808"
    );
}

#[test]
fn test_int_min_mod_neg1_no_panic() {
    let (ta, pa) = (hudhudscript_bytecode::ReprTag::Int, i64::MIN as u64);
    let (tb, pb) = (hudhudscript_bytecode::ReprTag::Int, (-1i64) as u64);
    let a = Value16(hudhudscript_bytecode::Repr::new_inline(ta, pa));
    let b = Value16(hudhudscript_bytecode::Repr::new_inline(tb, pb));
    let result = hudhudscript_vm::vm::bigint_arith::int_mod(a, b).unwrap();
    assert_eq!(result.as_int(), Some(0), "i64::MIN % -1 should be 0");
}

#[test]
fn test_normal_div_unchanged() {
    let a = Value16::int(10);
    let b = Value16::int(3);
    assert_eq!(
        hudhudscript_vm::vm::bigint_arith::int_div(a, b)
            .unwrap()
            .as_int(),
        Some(3)
    );
    assert_eq!(
        hudhudscript_vm::vm::bigint_arith::int_mod(a, b)
            .unwrap()
            .as_int(),
        Some(1)
    );
    let a = Value16::int(-10);
    assert_eq!(
        hudhudscript_vm::vm::bigint_arith::int_div(a, b)
            .unwrap()
            .as_int(),
        Some(-3)
    );
    assert_eq!(
        hudhudscript_vm::vm::bigint_arith::int_mod(a, b)
            .unwrap()
            .as_int(),
        Some(-1)
    );
}

#[test]
fn test_div_by_zero_still_errors() {
    let src = "let x = 5 / 0";
    let mut vm = VM::new();
    let ast = hudhudscript_parser::parse(src).unwrap();
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&ast).unwrap();
    let result = vm.execute(&bc);
    assert!(result.is_err(), "division by zero should error");
}
