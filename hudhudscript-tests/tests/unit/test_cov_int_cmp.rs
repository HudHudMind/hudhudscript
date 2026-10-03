//! Coverage tests for `vm/execute/int_cmp.rs` — comparison, negation and fused
//! return opcode semantics through parser + compiler + VM public API. Every
//! test pins an exact value or an exact error fragment from the VM source.
//! Script shaping: computed locals (`1 + 2`) keep both operands in registers
//! so `IntCmp` survives; `<local> <op> <int literal>` fuses to `IntCmpI`;
//! `<local> % <imm> <op> <imm>` chains to `IntModCmpI`; `return a <op> b;`
//! inside a function fuses to the `Int*Return` opcodes.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::vm::VM;

// ── helpers ──────────────────────────────────────────────────────────────────

fn run_result(source: &str) -> Result<VM, hudhudscript_errors::Error> {
    let ast = parse(source).expect("source must parse");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("source must compile");
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

fn run(source: &str) -> VM {
    run_result(source).unwrap_or_else(|e| panic!("script must run: {}", e.message))
}

fn err_msg(source: &str) -> String {
    match run_result(source) {
        Err(e) => e.message,
        Ok(_) => panic!("script must fail at runtime: {}", source),
    }
}

fn val(vm: &VM, name: &str) -> hudhudscript_bytecode::Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("variable {} must be published", name))
}

fn bool_of(vm: &VM, name: &str) -> bool {
    val(vm, name).as_bool().unwrap_or_else(|| panic!("{} must be a Bool", name))
}

fn int_of(vm: &VM, name: &str) -> i64 {
    val(vm, name).as_int().unwrap_or_else(|| panic!("{} must be an Int", name))
}

fn num_of(vm: &VM, name: &str) -> f64 {
    val(vm, name).as_number().unwrap_or_else(|| panic!("{} must be numeric", name))
}

fn big_of(vm: &VM, name: &str) -> String {
    val(vm, name)
        .as_bigint()
        .unwrap_or_else(|| panic!("{} must be a BigInt", name))
        .to_string()
}

// ── IntCmp — register-register, pure Int operands ────────────────────────────

#[test]
fn int_cmp_int_operands_all_ops() {
    let vm = run("let a = 1 + 2;\nlet b = 3 + 4;\n\
                  let r0 = a < b;\nlet r1 = a <= b;\nlet r2 = a > b;\n\
                  let r3 = a >= b;\nlet r4 = a == b;\nlet r5 = a != b;\n");
    assert!(bool_of(&vm, "r0"), "3 < 7");
    assert!(bool_of(&vm, "r1"), "3 <= 7");
    assert!(!bool_of(&vm, "r2"), "3 > 7");
    assert!(!bool_of(&vm, "r3"), "3 >= 7");
    assert!(!bool_of(&vm, "r4"), "3 == 7");
    assert!(bool_of(&vm, "r5"), "3 != 7");
}

#[test]
fn int_cmp_int_boundary_equal_operands() {
    let vm = run("let b = 3 + 4;\nlet c = 5 + 2;\n\
                  let r0 = b < c;\nlet r1 = b <= c;\nlet r2 = b > c;\n\
                  let r3 = b >= c;\nlet r4 = b == c;\nlet r5 = b != c;\n");
    assert!(!bool_of(&vm, "r0"), "7 < 7");
    assert!(bool_of(&vm, "r1"), "7 <= 7");
    assert!(!bool_of(&vm, "r2"), "7 > 7");
    assert!(bool_of(&vm, "r3"), "7 >= 7");
    assert!(bool_of(&vm, "r4"), "7 == 7");
    assert!(!bool_of(&vm, "r5"), "7 != 7");
}

#[test]
fn int_cmp_mixed_int_float_operands() {
    let vm = run("let i = 2 + 1;\nlet f = 1.5 + 2.0;\nlet lt = i < f;\nlet gt = i > f;\n\
                  let five = 2 + 3;\nlet fh = 4.0 + 1.0;\nlet eq = five == fh;\n");
    assert!(bool_of(&vm, "lt"), "3 < 3.5 across Int/Number");
    assert!(!bool_of(&vm, "gt"), "3 > 3.5 across Int/Number");
    assert!(bool_of(&vm, "eq"), "5 == 5.0 must compare numerically");
}

#[test]
fn int_cmp_float_float_operands() {
    let vm = run("let a = 1.5 + 1.0;\nlet b = 2.0 + 0.75;\n\
                  let lt = a < b;\nlet ge = b >= a;\nlet gt = a > b;\n");
    assert!(bool_of(&vm, "lt"), "2.5 < 2.75");
    assert!(bool_of(&vm, "ge"), "2.75 >= 2.5");
    assert!(!bool_of(&vm, "gt"), "2.5 > 2.75");
}

#[test]
fn int_cmp_nan_is_not_equal_to_itself() {
    let vm = run("let nv = Math.sqrt(-1.0);\nlet eq = nv == nv;\nlet ne = nv != nv;\n\
                  let lt = nv < 5;\nlet ge = nv >= 5;\n");
    assert!(!bool_of(&vm, "eq"), "NaN == NaN must be false");
    assert!(bool_of(&vm, "ne"), "NaN != NaN must be true");
    assert!(!bool_of(&vm, "lt"), "NaN < 5 must be false");
    assert!(!bool_of(&vm, "ge"), "NaN >= 5 must be false");
}

#[test]
fn int_cmp_negative_zero_equals_positive_zero() {
    let vm = run("let z = 1.0 - 1.0;\nlet nz = -z;\nlet eq = nz == z;\n");
    assert_eq!(num_of(&vm, "nz"), 0.0);
    assert!(bool_of(&vm, "eq"), "-0.0 == 0.0 must be true (IEEE 754)");
}

// ── IntCmp — string / bool / null / object operands ──────────────────────────

#[test]
fn int_cmp_strings_compare_lexicographically() {
    let vm = run("let sa = \"abc\";\nlet sb = \"abd\";\n\
                  let lt = sa < sb;\nlet le = sa <= sb;\nlet gt = sa > sb;\n\
                  let ge = sa >= sb;\nlet ne = sa != sb;\n\
                  let sc = \"ab\" + \"c\";\nlet eqc = sa == sc;\n");
    assert!(bool_of(&vm, "lt"), "\"abc\" < \"abd\"");
    assert!(bool_of(&vm, "le"), "\"abc\" <= \"abd\"");
    assert!(!bool_of(&vm, "gt"), "\"abc\" > \"abd\"");
    assert!(!bool_of(&vm, "ge"), "\"abc\" >= \"abd\"");
    assert!(bool_of(&vm, "ne"), "\"abc\" != \"abd\"");
    assert!(bool_of(&vm, "eqc"), "\"abc\" == \"ab\"+\"c\" by content");
}

#[test]
fn int_cmp_bools_equality_and_ordering() {
    // Ordering ops on bools are defined in the handler as
    // op0: !a && b, op1: !a || a == b (see the Bool arm of IntCmp).
    let vm = run("let bt = true;\nlet bf = false;\n\
                  let eq = bt == bf;\nlet ne = bt != bf;\nlet eqs = bt == bt;\n\
                  let lt = bf < bt;\nlet le = bt <= bt;\n");
    assert!(!bool_of(&vm, "eq"), "true == false");
    assert!(bool_of(&vm, "ne"), "true != false");
    assert!(bool_of(&vm, "eqs"), "true == true");
    assert!(bool_of(&vm, "lt"), "false < true is defined as !a && b");
    assert!(bool_of(&vm, "le"), "true <= true is defined as !a || a == b");
}

#[test]
fn int_cmp_null_equality_and_false_ordering() {
    let vm = run("let na = null;\nlet nb = null;\nlet one = 1;\n\
                  let eq = na == nb;\nlet ne = na != nb;\n\
                  let eq2 = na == one;\nlet lt = na < one;\n");
    assert!(bool_of(&vm, "eq"), "null == null");
    assert!(!bool_of(&vm, "ne"), "null != null");
    assert!(!bool_of(&vm, "eq2"), "null == 1 must be false");
    assert!(!bool_of(&vm, "lt"), "null < 1 must be false (non-eq ops on null)");
}

#[test]
fn int_cmp_objects_use_identity_policy() {
    // Default ObjectEquality::Identity: same pointer equal, distinct
    // literals with equal content are not.
    let vm = run("let o1 = { x: 1 };\nlet o2 = { x: 1 };\n\
                  let same = o1 == o1;\nlet diff = o1 == o2;\nlet neq = o1 != o2;\n");
    assert!(bool_of(&vm, "same"), "o1 == o1 identity");
    assert!(!bool_of(&vm, "diff"), "{{x:1}} == {{x:1}} must not be deep-equal");
    assert!(bool_of(&vm, "neq"), "{{x:1}} != {{x:1}}");
}

#[test]
fn int_cmp_bigint_register_operands() {
    let vm = run("let bv = 9223372036854775807 + 1;\nlet bv2 = bv + 1;\nlet n = 2 + 3;\n\
                  let lt = bv < bv2;\nlet gt = bv2 > bv;\nlet eq = bv == bv;\n\
                  let ne = bv != bv2;\nlet gtn = bv > n;\n");
    assert!(bool_of(&vm, "lt"), "2^63 < 2^63+1");
    assert!(bool_of(&vm, "gt"), "2^63+1 > 2^63");
    assert!(bool_of(&vm, "eq"), "BigInt self-equality");
    assert!(bool_of(&vm, "ne"), "2^63 != 2^63+1");
    assert!(bool_of(&vm, "gtn"), "BigInt > Int compares exactly");
}

// ── IntCmpI — immediate comparison ───────────────────────────────────────────

#[test]
fn int_cmp_i_int_immediate_all_ops() {
    let vm = run("let x = 3 + 2;\n\
                  let r0 = x < 10;\nlet r1 = x <= 5;\nlet r2 = x > 10;\n\
                  let r3 = x >= 5;\nlet r4 = x == 10;\nlet r5 = x != 10;\n");
    assert!(bool_of(&vm, "r0"), "5 < 10");
    assert!(bool_of(&vm, "r1"), "5 <= 5 boundary");
    assert!(!bool_of(&vm, "r2"), "5 > 10");
    assert!(bool_of(&vm, "r3"), "5 >= 5 boundary");
    assert!(!bool_of(&vm, "r4"), "5 == 10");
    assert!(bool_of(&vm, "r5"), "5 != 10");
}

#[test]
fn int_cmp_i_number_immediate() {
    let vm = run("let f = 4.5;\nlet lt = f < 5;\nlet ge = f >= 5;\nlet ne = f != 5;\n");
    assert!(bool_of(&vm, "lt"), "4.5 < 5 in the Number arm");
    assert!(!bool_of(&vm, "ge"), "4.5 >= 5 in the Number arm");
    assert!(bool_of(&vm, "ne"), "4.5 != 5 in the Number arm");
}

#[test]
fn int_cmp_i_bigint_immediate() {
    let vm = run("let bv = 9223372036854775807 + 1;\n\
                  let gt = bv > 10;\nlet lt = bv < 10;\nlet eq = bv == 10;\n");
    assert!(bool_of(&vm, "gt"), "2^63 > 10 in the BigInt arm");
    assert!(!bool_of(&vm, "lt"), "2^63 < 10");
    assert!(!bool_of(&vm, "eq"), "2^63 == 10");
}

// ── IntModCmpI — fused modulo-then-compare ───────────────────────────────────

#[test]
fn int_mod_cmp_i_int_all_ops() {
    // 7 % 3 == 1; all six comparisons against that remainder.
    let vm = run("let x = 3 + 4;\n\
                  let r0 = x % 3 < 2;\nlet r1 = x % 3 <= 1;\nlet r2 = x % 3 > 1;\n\
                  let r3 = x % 3 >= 2;\nlet r4 = x % 3 == 1;\nlet r5 = x % 3 != 1;\n");
    assert!(bool_of(&vm, "r0"), "1 < 2");
    assert!(bool_of(&vm, "r1"), "1 <= 1");
    assert!(!bool_of(&vm, "r2"), "1 > 1");
    assert!(!bool_of(&vm, "r3"), "1 >= 2");
    assert!(bool_of(&vm, "r4"), "1 == 1");
    assert!(!bool_of(&vm, "r5"), "1 != 1");
}

#[test]
fn int_mod_cmp_i_negative_remainder_keeps_sign() {
    // -5 % 2 == -1 (Rust remainder semantics), so == 1 fails, == -1 passes.
    let vm = run("let x = 0 - 5;\nlet hit = x % 2 == -1;\nlet miss = x % 2 == 1;\n");
    assert!(bool_of(&vm, "hit"), "-5 % 2 == -1");
    assert!(!bool_of(&vm, "miss"), "-5 % 2 == 1 must be false");
}

#[test]
fn int_mod_cmp_i_even_number() {
    let vm = run("let x = 2 + 4;\nlet even = x % 2 == 0;\n");
    assert!(bool_of(&vm, "even"), "6 % 2 == 0 (collatz even test)");
}

#[test]
fn int_mod_cmp_i_number_operand_takes_float_arm() {
    // Parameters are Unknown-typed, so the fused opcode is emitted and the
    // runtime tag picks the arm: 3.0 % 2 == 1.0 is true, 7.5 % 2 == 1.5 is
    // not 1.
    let vm = run("fn has1(x) { return x % 2 == 1; }\n\
                  let a = has1(3.0);\nlet b = has1(7.5);\nlet c = has1(3);\n");
    assert!(bool_of(&vm, "a"), "3.0 % 2 == 1.0");
    assert!(!bool_of(&vm, "b"), "7.5 % 2 == 1.5 != 1");
    assert!(bool_of(&vm, "c"), "3 % 2 == 1");
}

#[test]
fn int_mod_cmp_i_string_operand_yields_false_without_error() {
    // A string call-site argument types the param as string, so `x % 2 == 1`
    // takes a dynamic path yielding Bool(false) instead of reaching the
    // IntModCmpI "src not numeric" guard (verified by direct execution).
    let vm = run("fn has1(x) { return x % 2 == 1; }\nlet a = has1(\"aaaa\");\n");
    assert!(!bool_of(&vm, "a"), "string % int must yield false without error");
}

// ── Neg — unary minus ────────────────────────────────────────────────────────

#[test]
fn neg_int_and_float() {
    let vm = run("fn do_neg(x) { return -x; }\n\
                  let a = do_neg(7);\nlet b = do_neg(a);\nlet f = 2.5;\nlet g = -f;\n");
    assert_eq!(int_of(&vm, "a"), -7);
    assert_eq!(int_of(&vm, "b"), 7, "-(-7) == 7");
    assert_eq!(num_of(&vm, "g"), -2.5);
}

#[test]
fn neg_i64_min_promotes_to_bigint() {
    // G3.1: negating i64::MIN must promote to BigInt 2^63, not wrap.
    let vm = run("fn do_neg(x) { return -x; }\n\
                  let lo = -9223372036854775807 - 1;\nlet z = do_neg(lo);\n");
    assert_eq!(big_of(&vm, "z"), "9223372036854775808");
}

#[test]
fn neg_bigint_stays_exact() {
    let vm = run("fn do_neg(x) { return -x; }\n\
                  let bv = 9223372036854775807 + 1;\nlet nb = do_neg(bv);\n\
                  let hit = nb == -9223372036854775808;\n");
    assert!(bool_of(&vm, "hit"), "-2^63 must compare exactly");
}

#[test]
fn neg_non_numeric_strings_report_unsupported_type() {
    // Both inline-tagged and heap strings land on the same runtime error
    // ("Neg: unsupported type"); the earlier expectation of two distinct
    // messages was wrong (verified by direct execution).
    let inline = err_msg("fn do_neg(x) { return -x; }\nlet r = do_neg(\"abc\");\n");
    assert!(inline.contains("Neg: unsupported type"), "got: {}", inline);
    let heap = err_msg("fn do_neg(x) { return -x; }\nlet r = do_neg(\"aaaaaaaaaaaaaaaa\");\n");
    assert!(heap.contains("Neg: unsupported type"), "got: {}", heap);
}

// ── Not — logical not via truthiness ─────────────────────────────────────────

#[test]
fn not_flips_truthiness_per_tag() {
    let vm = run("let zi = 1 - 1;\nlet n0 = !zi;\nlet seven = 3 + 4;\nlet n7 = !seven;\n\
                  let es = \"\";\nlet ne = !es;\nlet nl = null;\nlet nn = !nl;\n\
                  let bt = true;\nlet nb = !bt;\n\
                  let zf = 1.0 - 1.0;\nlet nzf = !zf;\nlet hf = 1.5;\nlet nhf = !hf;\n");
    assert!(bool_of(&vm, "n0"), "!0");
    assert!(!bool_of(&vm, "n7"), "!7");
    assert!(bool_of(&vm, "ne"), "!\"\" (empty string is falsy)");
    assert!(bool_of(&vm, "nn"), "!null");
    assert!(!bool_of(&vm, "nb"), "!true");
    assert!(bool_of(&vm, "nzf"), "!0.0");
    assert!(!bool_of(&vm, "nhf"), "!1.5");
}

// ── IntAddReturn / IntSubReturn / IntMulReturn / IntDivReturn ────────────────

#[test]
fn int_add_return_values_and_overflow_promotion() {
    let vm = run("fn add2(a, b) { return a + b; }\nlet s = add2(2, 3);\n\
                  let big = add2(9223372036854775807, 1);\n");
    assert_eq!(int_of(&vm, "s"), 5);
    assert_eq!(big_of(&vm, "big"), "9223372036854775808");
}

#[test]
fn int_sub_return_values_and_underflow_promotion() {
    let vm = run("fn sub2(a, b) { return a - b; }\nlet s = sub2(7, 3);\n\
                  let lo = -9223372036854775807 - 1;\nlet u = sub2(lo, 1);\n");
    assert_eq!(int_of(&vm, "s"), 4);
    assert_eq!(big_of(&vm, "u"), "-9223372036854775809");
}

#[test]
fn int_mul_return_values_and_overflow_promotion() {
    let vm = run("fn mul2(a, b) { return a * b; }\nlet s = mul2(6, 7);\n\
                  let big = mul2(4611686018427387904, 4);\n");
    assert_eq!(int_of(&vm, "s"), 42);
    assert_eq!(big_of(&vm, "big"), "18446744073709551616");
}

#[test]
fn int_div_return_truncates_and_supports_float_and_bigint() {
    let vm = run("fn div2(a, b) { return a / b; }\n\
                  let q = div2(7, 2);\nlet nq = div2(-7, 2);\nlet fq = div2(7.5, 2.5);\n\
                  let bv = 9223372036854775807 + 1;\nlet bq = div2(bv, 2);\n\
                  let lo = -9223372036854775807 - 1;\nlet mz = 0 - 1;\nlet oq = div2(lo, mz);\n");
    assert_eq!(int_of(&vm, "q"), 3, "7 / 2 truncates");
    assert_eq!(int_of(&vm, "nq"), -3, "-7 / 2 truncates toward zero");
    assert_eq!(num_of(&vm, "fq"), 3.0, "float operands divide as floats");
    // BigInt 2^63 / 2 == 2^62, which fits i64, so the result comes back as
    // Int (verified by direct execution).
    assert_eq!(int_of(&vm, "bq"), 4611686018427387904, "BigInt / Int demotes to Int when it fits");
    assert_eq!(big_of(&vm, "oq"), "9223372036854775808", "i64::MIN / -1 promotes");
}

#[test]
fn int_div_return_zero_divisor_errors() {
    // bigint_arith::int_div signals ErrorCode(310) for a zero divisor; the
    // handler renders it with Display, i.e. the E0310 catalog code.
    let msg = err_msg("fn div2(a, b) { return a / b; }\nlet q = div2(5, 0);\n");
    assert!(msg.contains("[E0310]"), "got: {}", msg);
}

// ── IntCmpIReturn — fused compare-and-return ─────────────────────────────────

#[test]
fn int_cmp_i_return_ops() {
    let vm = run("fn lt5(x) { return x < 5; }\nfn eq5(x) { return x == 5; }\n\
                  fn ne5(x) { return x != 5; }\n\
                  let a = lt5(3);\nlet b = lt5(9);\n\
                  let c = eq5(5);\nlet d = eq5(6);\nlet e = ne5(5);\n");
    assert!(bool_of(&vm, "a"), "3 < 5");
    assert!(!bool_of(&vm, "b"), "9 < 5");
    assert!(bool_of(&vm, "c"), "5 == 5");
    assert!(!bool_of(&vm, "d"), "6 == 5");
    assert!(!bool_of(&vm, "e"), "5 != 5");
}

#[test]
fn int_cmp_i_return_number_operand() {
    let vm = run("fn lt5(x) { return x < 5; }\nlet a = lt5(4.5);\nlet b = lt5(5.5);\n");
    assert!(bool_of(&vm, "a"), "4.5 < 5 in the Number arm");
    assert!(!bool_of(&vm, "b"), "5.5 < 5 in the Number arm");
}

#[test]
fn int_cmp_i_return_not_numeric_errors() {
    let msg = err_msg("fn lt5(x) { return x < 5; }\nlet a = lt5(\"abc\");\n");
    assert!(msg.contains("IntCmpIReturn: src not numeric"), "got: {}", msg);
}
