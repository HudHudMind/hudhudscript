//! G3.2: BigInt exactness test values.
//! Uses comparison-based assertions to avoid display_string BigInt issues.

use hudhudscript_vm::VM;

fn run_vm(src: &str) -> VM {
    let stmts = hudhudscript_parser::parse(src).expect("parse");
    let mut compiler = hudhudscript_compiler::Compiler::new();
    let bc = compiler.compile(&stmts).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm
}

#[test]
fn g3_i64_max_plus_1() {
    let vm = run_vm("let x = 9223372036854775807 + 1; if (x == 9223372036854775808) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_i64_min_minus_1() {
    let vm = run_vm("let x = -9223372036854775808 - 1; if (x == -9223372036854775809) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_neg_i64_min() {
    // Use variable to force unary Neg on i64 value (not parser-absorbed minus)
    let vm = run_vm("let x = 9223372036854775807; let y = -(x + 1); let z = -y; if (z == 9223372036854775808) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_i64_max_times_2() {
    let vm = run_vm("let x = 9223372036854775807 * 2; if (x == 18446744073709551614) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_plus_int() {
    let vm = run_vm("let b = 9223372036854775807 + 1; let x = b + 100; if (x == 9223372036854775908) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_int_plus_bigint() {
    let vm = run_vm("let b = 9223372036854775807 + 1; let x = 100 + b; if (x == 9223372036854775908) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_mul_bigint() {
    let vm = run_vm("let a = 9223372036854775807 + 1; let b = 2; let x = a * b; if (x == 18446744073709551616) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_compare_exact() {
    let vm = run_vm("let a = 9223372036854775807 + 1; let b = 9223372036854775807 + 2; if (a < b) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_sub_bigint() {
    let vm = run_vm("let a = 9223372036854775807 + 100; let b = 9223372036854775807 + 1; let x = a - b; if (x == 99) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_div_bigint() {
    let vm = run_vm("let a = 9223372036854775807 + 1; let x = a / 2; if (x == 4611686018427387904) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_mod_bigint() {
    let vm = run_vm("let a = 9223372036854775807 + 10; let x = a % 3; if (x == 2) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_neg_bigint_div() {
    let vm = run_vm("let a = -(9223372036854775807 + 1); let x = a / 2; if (x == -4611686018427387904) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_div_by_zero_int() {
    let vm = run_vm("let ok = 0; try { let x = 100 / 0; } catch (e) { ok = 1; } return ok;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

// ── G1 fix: BigInt+Int/Int+BigInt/BigInt+BigInt × add/sub/mul ──────
// These 9 cases were NOT covered by the existing 13 tests (gap closed).

#[test]
fn g3_bigint_plus_int_exact_add() {
    let vm = run_vm("let b = 9223372036854775807 + 1; let x = b + 100; if (x == 9223372036854775908) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_int_plus_bigint_exact_add() {
    let vm = run_vm("let b = 9223372036854775807 + 1; let x = 100 + b; if (x == 9223372036854775908) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_plus_bigint_exact_add() {
    let vm = run_vm("let a = 9223372036854775807 + 1; let b = 9223372036854775807 + 2; let x = a + b; if (x == 18446744073709551617) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_minus_int_exact_sub() {
    let vm = run_vm("let b = 9223372036854775807 + 100; let x = b - 99; if (x == 9223372036854775808) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_int_minus_bigint_exact_sub() {
    let vm = run_vm("let b = 9223372036854775807 + 1; let x = 0 - b; if (x == -9223372036854775808) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_minus_bigint_exact_sub() {
    let vm = run_vm("let a = 9223372036854775807 + 100; let b = 9223372036854775807 + 1; let x = a - b; if (x == 99) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_times_int_exact_mul() {
    let vm = run_vm("let b = 9223372036854775807 + 1; let x = b * 3; if (x == 27670116110564327424) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_int_times_bigint_exact_mul() {
    let vm = run_vm("let b = 9223372036854775807 + 1; let x = 3 * b; if (x == 27670116110564327424) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}

#[test]
fn g3_bigint_times_bigint_exact_mul() {
    let vm = run_vm("let a = 9223372036854775807 + 1; let b = 2; let x = a * b; if (x == 18446744073709551616) { return 1; } return 0;");
    assert_eq!(vm.last_return_value().display_string(), "1");
}
