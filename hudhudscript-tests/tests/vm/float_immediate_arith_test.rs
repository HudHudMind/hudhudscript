// Float operands on the immediate arithmetic opcodes (IntSubI / IntMulI / IntAddI).
//
// `int_math_op_i!` in vm/math_fast_paths.rs took no float operation parameter:
// its slow path hardcoded `x + imm as f64`, so all three ops shared addition.
// `f() - 2` and `f() * 2` on a float both returned `f() + 2` — silently, with no
// error. It only surfaced when the compiler could not prove the operand was a
// float, which is exactly the case when the value arrives from a call:
//
//     fn f() { return 5.5; }
//     let a = f();
//     a - 2      // was 7.5, must be 3.5
//     a * 2      // was 7.5, must be 11
//
// A float *literal* bound to a local took a different opcode and was always
// right, which is why the bug survived: `let b = 5.5; b - 2` == 3.5.
//
// These tests pin every op × operand-provenance combination so one macro arm
// can never again stand in for the other two.
use hudhudscript_vm::VM;

fn eval(src: &str) -> String {
    let stmts = hudhudscript_parser::parse(src).expect("parse failed");
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).expect("compile failed");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute failed");
    vm.last_return_value().display_string()
}

// ======================================================================
// The bug: float from a call, immediate operand
// ======================================================================
#[test]
fn call_returned_float_minus_immediate() {
    let src = r#"
fn f() { return 5.5; }
let a = f();
return a - 2;
"#;
    assert_eq!(
        eval(src),
        "3.5",
        "subtraction must subtract; 7.5 means the op fell back to addition"
    );
}

#[test]
fn call_returned_float_times_immediate() {
    let src = r#"
fn f() { return 5.5; }
let a = f();
return a * 2;
"#;
    assert_eq!(
        eval(src),
        "11",
        "multiplication must multiply; 7.5 means the op fell back to addition"
    );
}

#[test]
fn call_returned_float_plus_immediate() {
    let src = r#"
fn f() { return 5.5; }
let a = f();
return a + 2;
"#;
    assert_eq!(eval(src), "7.5");
}

// ======================================================================
// Literal-bound floats: the path that always worked — keep it working
// ======================================================================
#[test]
fn literal_float_minus_immediate() {
    assert_eq!(eval("let b = 5.5; return b - 2;"), "3.5");
}

#[test]
fn literal_float_times_immediate() {
    assert_eq!(eval("let b = 5.5; return b * 2;"), "11");
}

// ======================================================================
// Negative and zero immediates
// ======================================================================
#[test]
fn call_returned_float_minus_negative_immediate() {
    let src = r#"
fn f() { return 5.5; }
let a = f();
return a - -2;
"#;
    assert_eq!(eval(src), "7.5");
}

#[test]
fn call_returned_float_times_zero_immediate() {
    let src = r#"
fn f() { return 5.5; }
let a = f();
return a * 0;
"#;
    assert_eq!(eval(src), "0");
}

// ======================================================================
// Integers keep the integer path (the fast arm must not have moved)
// ======================================================================
#[test]
fn call_returned_int_arithmetic_unchanged() {
    let src = r#"
fn f() { return 5; }
let a = f();
return (a - 2) * 3 + 1;
"#;
    assert_eq!(eval(src), "10");
}

// ======================================================================
// The cold path still reaches BigInt promotion.
// (Stringified inside the script: `last_return_value().display_string()`
// renders a BigInt as "<dynamic>", so the digits have to come from the VM.)
// ======================================================================
#[test]
fn overflowing_int_still_promotes_to_bigint() {
    let src = r#"
fn f() { return 9223372036854775807; }
let a = f();
return "" + (a + 1);
"#;
    assert_eq!(
        eval(src),
        "9223372036854775808",
        "i64::MAX + 1 must promote instead of wrapping"
    );
}

#[test]
fn overflowing_int_multiply_still_promotes() {
    let src = r#"
fn f() { return 4611686018427387904; }
let a = f();
return "" + (a * 4);
"#;
    assert_eq!(eval(src), "18446744073709551616");
}

// ======================================================================
// Non-immediate (register-register) float ops — the sibling macro
// ======================================================================
#[test]
fn call_returned_floats_register_register_ops() {
    let src = r#"
fn f() { return 5.5; }
fn g() { return 2.0; }
let a = f();
let b = g();
return (a - b) * b;
"#;
    assert_eq!(eval(src), "7");
}

#[test]
fn mixed_int_float_register_ops() {
    let src = r#"
fn f() { return 5.5; }
fn g() { return 2; }
let a = f();
let b = g();
return a - b;
"#;
    assert_eq!(eval(src), "3.5");
}

// ======================================================================
// String concatenation still reaches the cold path it shares with BigInt
// ======================================================================
#[test]
fn string_concat_still_works_through_add() {
    let src = r#"
fn f() { return "ab"; }
let a = f();
return a + "cd";
"#;
    assert_eq!(eval(src), "abcd");
}
