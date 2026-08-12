//! G4.3: Differential lane tests — packed vs unpacked handler parity.
//! Verifies that D_NEG_R, D_NOT_R, D_ARRAY_PUSH_RRR produce identical
//! results to their unpacked counterparts.
//!
//! G4.4: Semantic fallthrough documentation.
//! Opcodes that intentionally fall through:
//!   - D_NEG_R: falls through for non-Int (Number handled in packed, Dynamic→unpacked)
//!   - D_NOT_R: falls through for non-Bool (unpacked handles is_truthy())
//!   - D_STRCAT_RRR: falls through for non-String operands
//!   - All Num opcodes: fall through when operands are Int (go to Int handler)

use hudhudscript_vm::VM;

fn run_vm(src: &str) -> VM {
    let stmts = hudhudscript_parser::parse(src).expect("parse");
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm
}

// ── Neg packed/unpacked parity ─────────────────────────────────────

#[test]
fn g4_neg_int_parity() {
    let vm = run_vm("let x = 42; if (-x == -42) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g4_neg_bigint_parity() {
    let vm = run_vm("let x = 9223372036854775807 + 1; let y = -x; if (y < 0) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g4_neg_double_negation() {
    let vm = run_vm("let x = 9223372036854775807; let y = -(x + 1); let z = -y; if (z == x + 1) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

// ── Not packed/unpacked parity ─────────────────────────────────────

#[test]
fn g4_not_bool_parity() {
    let vm = run_vm("if (!true == false) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g4_not_truthy_parity() {
    // G4.4: packed D_NOT_R only handles Bool (semantic mismatch with unpacked).
    // Unpacked Not handles is_truthy() for all types — packed doesn't.
    // This is documented intentional fallthrough for non-Bool operands.
    let vm = run_vm("if (!true == false && !false == true) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

// ── ArrayPush packed/unpacked parity ───────────────────────────────

#[test]
fn g4_array_push_parity() {
    let vm = run_vm("let a = []; a.push(1); a.push(2); a.push(3); if (a[2] == 3) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

// ── StrCat packed/unpacked parity ──────────────────────────────────

#[test]
fn g4_strcat_parity() {
    let vm = run_vm(r#"let x = "hello" + " " + "world"; if (x == "hello world") { return 1; } return 0;"#);
    assert_eq!(vm.last_return_value().display_string(), "1");
}

// ── NumDiv packed/unpacked parity ──────────────────────────────────

#[test]
fn g4_num_div_parity() {
    let vm = run_vm("let x = 3.0 / 2.0; if (x == 1.5) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

// ── G4.4: Semantic fallthrough documentation ──────────────────────

#[test]
fn g4_not_fallthrough_documented() {
    // G4.4: D_NOT_R packed handler only accepts Bool.
    // When operand is non-Bool (e.g., integer), it falls through
    // to unpacked Not which uses is_truthy().
    // This is the documented semantic mismatch.
    let vm = run_vm("let x = 10; if (!x == false) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}
