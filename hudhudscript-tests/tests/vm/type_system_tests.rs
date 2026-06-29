//! HudHud VM Type System & Opcode Unit Tests
//!
//! These tests verify the core VM behaviours documented in BUGS.md:
//!   1. Float literals (X.0) must parse as Number, not Int.
//!   2. .length properties must return Int (i64), not Number (f64).
//!   3. Unknown-type variables should not force IntXxx opcodes.
//!   4. IntNe must behave consistently with IntEq/IntGt (accept or reject Number).
//!   5. int / int must return Number and the type system must track it.
//!   6. String concatenation must clone, not mutate shared values.

use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Helper: parse → compile → execute → return VM state.
fn run(source: &str) -> CompileResult<VM> {
    let stmts = parse(source).map_err(|e| {
        hudhudscript_bytecode::error::compile_codes::runtime_error(format!("Parse error: {:?}", e))
    })?;
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&stmts)?;
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Literal parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn int_literal_zero_is_int() {
    let vm = run(r#"let x = 0; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_int(), "0 should be Int, got {:?}", x);
    assert_eq!(x.as_int(), Some(0));
}

#[test]
fn int_literal_positive_is_int() {
    let vm = run(r#"let x = 42; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_int(), "42 should be Int, got {:?}", x);
    assert_eq!(x.as_int(), Some(42));
}

#[test]
fn float_literal_zero_point_zero_is_number() {
    let vm = run(r#"let x = 0.0; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_number(), "0.0 must be Number, not Int. Got {:?}", x);
    assert!((x.as_number().unwrap() - 0.0).abs() < 1e-10);
}

#[test]
fn float_literal_two_point_zero_is_number() {
    let vm = run(r#"let x = 2.0; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_number(), "2.0 must be Number, not Int. Got {:?}", x);
    assert!((x.as_number().unwrap() - 2.0).abs() < 1e-10);
}

#[test]
fn float_literal_one_point_five_is_number() {
    let vm = run(r#"let x = 1.5; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_number(), "1.5 should be Number, got {:?}", x);
    assert!((x.as_number().unwrap() - 1.5).abs() < 1e-10);
}

#[test]
fn number_arithmetic_produces_number() {
    let vm = run(r#"let x = 0.5 - 0.5; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_number(), "0.5 - 0.5 must be Number, got {:?}", x);
    assert!((x.as_number().unwrap() - 0.0).abs() < 1e-10);
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. .length returns Int
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn array_length_returns_int() {
    let vm = run(r#"let arr = [1, 2, 3]; let n = arr.length; n;"#).expect("execution failed");
    let n = vm.get_variable("n").expect("n not found");
    assert!(n.is_int(), "arr.length must return Int, got {:?}", n);
    assert_eq!(n.as_int(), Some(3));
}

#[test]
fn string_length_returns_int() {
    let vm = run(r#"let s = "hello"; let n = s.length; n;"#).expect("execution failed");
    let n = vm.get_variable("n").expect("n not found");
    assert!(n.is_int(), "string.length must return Int, got {:?}", n);
    assert_eq!(n.as_int(), Some(5));
}

#[test]
fn array_length_int_comparison_works() {
    let vm = run(r#"
let arr = [1, 2, 3];
let i = 0;
while (i < arr.length) {
    i = i + 1;
}
i;
"#)
    .expect("execution failed");
    let i = vm.get_variable("i").expect("i not found");
    assert_eq!(i.as_int(), Some(3), "loop should iterate 3 times");
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. String concatenation must clone (not mutate shared references)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn string_concat_clones_does_not_mutate_original() {
    let vm = run(r#"
let first = "hello";
let combined = first + " world";
first;
"#)
    .expect("execution failed");
    let first = vm.get_variable("first").expect("first not found");
    assert_eq!(
        first.as_str(),
        Some("hello"),
        "original variable must NOT be mutated by concat"
    );
}

#[test]
fn string_concat_result_is_correct() {
    let vm = run(r#"
let a = "hello";
let b = " world";
let c = a + b;
c;
"#)
    .expect("execution failed");
    let c = vm.get_variable("c").expect("c not found");
    assert_eq!(c.as_str(), Some("hello world"));
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. int / int = Number (float) behaviour
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn int_div_int_produces_number() {
    let vm = run(r#"let x = 5 / 2; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(
        x.is_number(),
        "5 / 2 must produce Number (float), got {:?}",
        x
    );
    assert!((x.as_number().unwrap() - 2.5).abs() < 1e-10);
}

#[test]
fn int_div_int_chain_produces_number() {
    let vm = run(r#"
let current = 10;
current = current / 2;
current;
"#)
    .expect("execution failed");
    let current = vm.get_variable("current").expect("current not found");
    assert!(
        current.is_number(),
        "10 / 2 must produce Number, got {:?}",
        current
    );
    assert!((current.as_number().unwrap() - 5.0).abs() < 1e-10);
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Comparison operators consistency
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn int_eq_accepts_number() {
    let vm = run(r#"
let a = 1 + 0.5 - 0.5;
let b = 1;
let result = false;
if (a == b) { result = true; }
result;
"#)
    .expect("execution failed");
    let result = vm.get_variable("result").expect("result not found");
    assert!(
        result.as_bool().unwrap_or(false),
        "IntEq should accept Number"
    );
}

#[test]
fn int_gt_accepts_number() {
    let vm = run(r#"
let a = 1 + 0.5 - 0.5;
let b = 1;
let result = false;
if (a > b) { result = true; }
result;
"#)
    .expect("execution failed");
    let result = vm.get_variable("result").expect("result not found");
    assert!(
        !result.as_bool().unwrap_or(true),
        "IntGt with equal values should be false"
    );
}

#[test]
fn int_ne_must_accept_number_consistently() {
    let result = run(r#"
let a = 1 + 0.5 - 0.5;
let b = 1;
let result = false;
if (a != b) { result = true; }
result;
"#);
    let vm = result.expect("IntNe must not panic on Number operands");
    let result = vm.get_variable("result").expect("result not found");
    assert!(
        !result.as_bool().unwrap_or(true),
        "IntNe with equal Number/Int should be false"
    );
}

#[test]
fn number_ne_works_for_different_values() {
    let vm = run(r#"
let a = 1.5;
let b = 2.5;
let result = false;
if (a != b) { result = true; }
result;
"#)
    .expect("execution failed");
    let result = vm.get_variable("result").expect("result not found");
    assert!(
        result.as_bool().unwrap_or(false),
        "NumberNe for different values should be true"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. Unknown variable type — default opcode selection
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn stack_pop_unknown_type_number_comparison() {
    let vm = run(r#"
let stack = [];
stack.push(1000);
let high = stack.pop();
let result = false;
if (high > 0) { result = true; }
result;
"#)
    .expect("execution failed");
    let result = vm.get_variable("result").expect("result not found");
    assert!(
        result.as_bool().unwrap_or(false),
        "unknown-type variable compared with Int should work"
    );
}

#[test]
fn stack_pop_unknown_type_number_arithmetic() {
    let vm = run(r#"
let stack = [];
stack.push(10);
let x = stack.pop();
let y = x + 5;
y;
"#)
    .expect("execution failed");
    let y = vm.get_variable("y").expect("y not found");
    let val = y
        .as_int()
        .map(|v| v as f64)
        .or_else(|| y.as_number())
        .unwrap_or(f64::NAN);
    assert!(
        (val - 15.0).abs() < 1e-10,
        "x + 5 should equal 15, got {:?}",
        y
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. Collatz-like nested loop (Dynamic value regression)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn collatz_like_nested_loop_no_dynamic_error() {
    let vm = run(r#"
let steps = 0;
let current = 5 + 0.5 - 0.5;
while (current / 1 != 1.5 - 0.5) {
    if (current % 2 == 0.5 - 0.5) {
        current = current / 2;
    } else {
        current = current * 3 + 1;
    }
    steps = steps + 1;
}
steps;
"#)
    .expect("execution failed");
    let steps = vm.get_variable("steps").expect("steps not found");
    assert_eq!(steps.as_int(), Some(5), "collatz(5) takes 5 steps");
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Mixed-type arithmetic edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn number_plus_int_produces_number() {
    let vm = run(r#"let x = 0.5 + 2; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_number(), "0.5 + 2 must be Number, got {:?}", x);
    assert!((x.as_number().unwrap() - 2.5).abs() < 1e-10);
}

#[test]
fn int_plus_number_produces_number() {
    let vm = run(r#"let x = 2 + 0.5; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_number(), "2 + 0.5 must be Number, got {:?}", x);
    assert!((x.as_number().unwrap() - 2.5).abs() < 1e-10);
}

#[test]
fn array_length_minus_one_int_arithmetic() {
    let vm = run(r#"
let arr = [10, 20, 30];
let last_idx = arr.length - 1;
let val = arr[last_idx];
val;
"#)
    .expect("execution failed");
    let val = vm.get_variable("val").expect("val not found");
    assert_eq!(val.as_int(), Some(30), "arr[arr.length-1] should be 30");
}

#[test]
fn large_int_literal_stays_int() {
    let vm = run(r#"let x = 100000; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_int(), "100000 should be Int, got {:?}", x);
    assert_eq!(x.as_int(), Some(100000));
}

#[test]
fn large_float_literal_is_number() {
    let vm = run(r#"let x = 100000.0; x;"#).expect("execution failed");
    let x = vm.get_variable("x").expect("x not found");
    assert!(x.is_number(), "100000.0 must be Number, got {:?}", x);
    assert!((x.as_number().unwrap() - 100000.0).abs() < 1e-10);
}
