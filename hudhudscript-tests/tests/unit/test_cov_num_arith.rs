//! Coverage tests for `vm/execute/num_arith.rs` — arithmetic opcode semantics
//! through parser + compiler + VM public API. Every test pins an exact value
//! or an exact error fragment from the VM source. Script shaping notes sit on
//! each test: which fusions fire and which packed handler owns the message.

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

fn num(vm: &VM, name: &str) -> f64 {
    val(vm, name).as_number().unwrap_or_else(|| panic!("{} must be numeric", name))
}

fn int(vm: &VM, name: &str) -> i64 {
    val(vm, name).as_int().unwrap_or_else(|| panic!("{} must be an Int", name))
}

fn big(vm: &VM, name: &str) -> String {
    val(vm, name)
        .as_bigint()
        .unwrap_or_else(|| panic!("{} must be a BigInt", name))
        .to_string()
}

// ── NumAddI / NumSubI / NumMulI / NumDivI — immediate float opcodes ─────────

#[test]
fn num_add_i_fuses_immediate_float() {
    // Number-typed local forces NumAdd; LoadNumConst(2.0) fuses into NumAddI.
    let vm = run("let f = 1.5;\nlet s = f + 2.0;\n");
    assert_eq!(num(&vm, "s"), 3.5);
    assert!(val(&vm, "s").is_number(), "NumAddI result must be Number");
}

#[test]
fn num_sub_i_subtracts_immediate() {
    let vm = run("let f = 5.5;\nlet s = f - 2.0;\n");
    assert_eq!(num(&vm, "s"), 3.5);
}

#[test]
fn num_mul_i_multiplies_immediate() {
    let vm = run("let f = 2.5;\nlet p = f * 4.0;\n");
    assert_eq!(num(&vm, "p"), 10.0);
}

#[test]
fn num_div_i_divides_immediate() {
    let vm = run("let f = 9.0;\nlet q = f / 2.0;\n");
    assert_eq!(num(&vm, "q"), 4.5);
}

// ── NumAdd / NumSub / NumMul / NumDiv — register-register float ops ─────────

#[test]
fn num_add_register_floats() {
    // Fractional operands (2.25) cannot fold into an immediate form.
    let vm = run("let a = 1.5;\nlet b = 2.25;\nlet s = a + b;\n");
    assert_eq!(num(&vm, "s"), 3.75);
}

#[test]
fn num_add_mixed_number_int() {
    let vm = run("let f = 1.5;\nlet i = 3;\nlet s = f + i;\n");
    assert_eq!(num(&vm, "s"), 4.5);
}

#[test]
fn num_sub_register_and_mixed() {
    let vm = run("let a = 5.25;\nlet b = 2.5;\nlet d = a - b;\nlet i = 5;\nlet m = i - b;\n");
    assert_eq!(num(&vm, "d"), 2.75);
    assert_eq!(num(&vm, "m"), 2.5);
}

#[test]
fn num_mul_register_and_mixed() {
    let vm = run("let a = 2.5;\nlet b = 4.5;\nlet p = a * b;\nlet i = 3;\nlet m = i * a;\n");
    assert_eq!(num(&vm, "p"), 11.25);
    assert_eq!(num(&vm, "m"), 7.5);
}

#[test]
fn num_div_register_and_mixed() {
    let vm = run("let a = 7.5;\nlet b = 2.5;\nlet q = a / b;\nlet i = 3;\nlet m = a / i;\n");
    assert_eq!(num(&vm, "q"), 3.0);
    assert_eq!(num(&vm, "m"), 2.5);
}

#[test]
fn float_addition_keeps_ieee754_precision() {
    // 0.1 + 0.2 must produce the exact f64 rounding, not a decimal 0.3.
    let vm = run("let a = 0.1;\nlet b = 0.2;\nlet s = a + b;\n");
    assert_eq!(num(&vm, "s"), 0.1 + 0.2);
    assert_ne!(num(&vm, "s"), 0.3);
}

// ── NumDiv error paths ───────────────────────────────────────────────────────

#[test]
fn num_div_by_zero_errors() {
    // Computed zero keeps the divisor in a register: packed D_NUM_DIV_RR.
    let msg = err_msg("let f = 1.5;\nlet z = 1.0 - 1.0;\nlet q = f / z;\n");
    assert!(msg.contains("NumDiv: division by zero"), "got: {}", msg);
}

#[test]
fn num_ops_reject_bigint_operand() {
    // i64::MAX + 1 promotes to BigInt at runtime while the compiler infers
    // Int. `bv <op> f` fuses `f`'s adjacent LoadNumConst into Num*I (Dynamic
    // src -> "src not numeric"); `f / bv` cannot fuse and keeps the RR guard.
    let pre = "let bv = 9223372036854775807 + 1;\nlet f = 2.0;\n";
    for (op, expected) in [
        ("bv + f", "NumAddI: src not numeric"),
        ("bv - f", "NumSubI: src not numeric"),
        ("bv * f", "NumMulI: src not numeric"),
        ("f / bv", "Cannot mix BigInt and Number"),
    ] {
        let msg = err_msg(&format!("{}let q = {};\n", pre, op));
        assert!(msg.contains(expected), "{} must fail with {}, got: {}", op, expected, msg);
    }
}

// ── IntDiv — truncating integer division fast path ───────────────────────────

#[test]
fn int_div_truncates_toward_zero() {
    let vm = run("let a = 5 + 2;\nlet b = 1 + 1;\nlet q1 = a / b;\n\
                  let a2 = 0 - 7;\nlet q2 = a2 / b;\n\
                  let b2 = 0 - 2;\nlet q3 = a / b2;\nlet q4 = a2 / b2;\n");
    assert_eq!(int(&vm, "q1"), 3);
    assert_eq!(int(&vm, "q2"), -3, "-7/2 must truncate toward zero");
    assert_eq!(int(&vm, "q3"), -3, "7/-2 must truncate toward zero");
    assert_eq!(int(&vm, "q4"), 3);
}

#[test]
fn int_div_by_zero_errors() {
    let msg = err_msg("let a = 5 + 2;\nlet z = 1 - 1;\nlet q = a / z;\n");
    assert!(msg.contains("Division by zero"), "got: {}", msg);
}

#[test]
fn int_div_float_operands_fall_to_float_math() {
    // Parameters are Unknown-typed so IntDiv is emitted; runtime Number
    // operands take the as_number_fast branch of the same handler.
    let vm = run("fn divp(a, b) { let q = a / b; let out = q; return out; }\n\
                  let r1 = divp(7.5, 2.5);\nlet r2 = divp(7, 2.5);\nlet r3 = divp(7.5, 2);\n");
    assert_eq!(num(&vm, "r1"), 3.0);
    assert_eq!(num(&vm, "r2"), 2.8);
    assert_eq!(num(&vm, "r3"), 3.75);
}

#[test]
fn int_div_bigint_slow_path_is_exact() {
    // bigint_div wraps 2^63 / 2 in Value16::bigint, which demotes fitting
    // values to Int — int() pins both the exact value and the Int tag.
    let vm = run("let bv = 9223372036854775807 + 1;\nlet two = 1 + 1;\nlet q = bv / two;\n");
    assert_eq!(int(&vm, "q"), 4611686018427387904);
}

#[test]
fn int_div_slow_path_reports_type_error() {
    // Conflicting (Str, Str) vs (Int, Int) call sites keep the params Unknown
    // so IntDiv survives; strings reach bigint_div -> Debug format.
    let msg = err_msg("fn divp(a, b) { let q = a / b; let out = q; return out; }\n\
                       let r = divp(\"aaaa\", \"bbbb\");\nlet r2 = divp(3, 2);\n");
    assert!(msg.contains("Division error: ErrorCode(310)"), "got: {}", msg);
}

#[test]
fn int_div_overflow_promotes_to_bigint() {
    // i64::MIN / -1 overflows checked_div and promotes to BigInt 2^63.
    let vm = run("let lo = -9223372036854775807 - 1;\nlet minus = 0 - 1;\nlet q = lo / minus;\n");
    assert_eq!(big(&vm, "q"), "9223372036854775808");
}

// ── IntMod — remainder semantics ──────────────────────────────────────────────

#[test]
fn int_mod_sign_follows_dividend() {
    let vm = run("let a = 10 + 7;\nlet b = 2 + 3;\nlet r1 = a % b;\n\
                  let a2 = 0 - 17;\nlet r2 = a2 % b;\n\
                  let b2 = 0 - 5;\nlet r3 = a % b2;\nlet r4 = a2 % b2;\n");
    assert_eq!(int(&vm, "r1"), 2);
    assert_eq!(int(&vm, "r2"), -2, "-17 % 5 keeps the dividend sign");
    assert_eq!(int(&vm, "r3"), 2, "17 % -5 keeps the dividend sign");
    assert_eq!(int(&vm, "r4"), -2);
}

#[test]
fn int_mod_by_zero_errors() {
    let msg = err_msg("let a = 10 + 7;\nlet z = 1 - 1;\nlet r = a % z;\n");
    assert!(msg.contains("Modulo by zero"), "got: {}", msg);
}

#[test]
fn int_mod_float_operands_fall_to_float_math() {
    let vm = run("fn modp(a, b) { let r = a % b; let out = r; return out; }\n\
                  let r1 = modp(17.5, 5.0);\nlet r2 = modp(17.5, 5);\n");
    assert_eq!(num(&vm, "r1"), 2.5);
    assert_eq!(num(&vm, "r2"), 2.5);
    assert!(val(&vm, "r1").is_number(), "float modulo yields Number");
}

#[test]
fn int_mod_bigint_slow_path_demotes_to_int() {
    // ...5817 % 3 == 2; bigint_mod's Value16::bigint demotes the fitting
    // remainder to a plain Int.
    let vm = run("let bv = 9223372036854775807 + 10;\nlet three = 1 + 2;\nlet r = bv % three;\n");
    assert_eq!(int(&vm, "r"), 2);
}

#[test]
fn int_mod_slow_path_reports_type_error() {
    // Conflicting call sites keep params Unknown so IntMod survives; strings
    // reach bigint_mod -> Debug format.
    let msg = err_msg("fn modp(a, b) { let r = a % b; let out = r; return out; }\n\
                       let r = modp(\"aaaa\", \"bbbb\");\nlet r2 = modp(3, 2);\n");
    assert!(msg.contains("Modulo error: ErrorCode(310)"), "got: {}", msg);
}

// ── NumMod — Number-domain remainder ─────────────────────────────────────────

#[test]
fn num_mod_number_by_int() {
    let vm = run("let f = 17.5;\nlet i = 5;\nlet r = f % i;\n");
    assert_eq!(num(&vm, "r"), 2.5);
}

#[test]
fn num_mod_number_by_number() {
    let vm = run("let f = 17.5;\nlet g = 4.0;\nlet r = f % g;\n");
    assert_eq!(num(&vm, "r"), 1.5);
}

#[test]
fn num_mod_by_one_takes_fract_path() {
    // b == 1.0 branch: 17.75 % 1.0 == fract(17.75) == 0.75.
    let vm = run("let f = 17.75;\nlet one = 2.0 - 1.0;\nlet r = f % one;\n");
    assert_eq!(num(&vm, "r"), 0.75);
}

#[test]
fn num_mod_by_zero_errors_for_int_and_float_divisors() {
    let msg_int = err_msg("let f = 1.5;\nlet z = 1 - 1;\nlet r = f % z;\n");
    assert!(msg_int.contains("Modulo by zero"), "got: {}", msg_int);
    let msg_flt = err_msg("let f = 1.5;\nlet z = 1.0 - 1.0;\nlet r = f % z;\n");
    assert!(msg_flt.contains("Modulo by zero"), "got: {}", msg_flt);
}

// ── NumMulAddAssign — fused Horner self-accumulation ─────────────────────────

#[test]
fn num_mul_add_assign_float_accumulates() {
    // acc = acc * m + a -> 2.0 * 3.0 + 1.5.
    let vm = run("fn fma2(m, a) { let acc = 2.0; acc = acc * m + a; return acc; }\n\
                  let r = fma2(3.0, 1.5);\n");
    assert_eq!(num(&vm, "r"), 7.5);
}

#[test]
fn num_mul_add_assign_keeps_int_result() {
    // Packed D_NUM_MUL_ADD_ASSIGN takes the all-Int fast path
    // (checked_mul + checked_add): the accumulator stays a plain Int.
    let vm = run("fn fmai(m, a) { let acc = 2; acc = acc * m + a; return acc; }\n\
                  let r = fmai(3, 1);\n");
    assert_eq!(int(&vm, "r"), 7);
}

#[test]
fn num_mul_add_assign_rejects_bigint() {
    // Non-numeric mixes route through bigint_arith::int_mul whose
    // ErrorCode(310) renders via the catalog.
    let msg = err_msg("fn fma2(m, a) { let acc = 2.0; acc = acc * m + a; return acc; }\n\
                       let bv = 9223372036854775807 + 1;\nlet r = fma2(bv, 1);\n");
    assert!(msg.contains("[E0310] BigInt and Number cannot be mixed"), "got: {}", msg);
}

// ── NumMulAddIndexed — Horner loop over an array ─────────────────────────────

#[test]
fn num_mul_add_indexed_horner_evaluates_polynomial() {
    // coeffs [1,2,3] at x=10: 3*10+2 = 32, 32*10+1 = 321.
    let vm = run("fn horner(coeffs, x) { let result = coeffs[2]; let i = 1;\n\
                  while (i >= 0) { result = result * x + coeffs[i]; i = i - 1; }\n\
                  return result; }\n\
                  let r = horner([1, 2, 3], 10);\n");
    assert_eq!(num(&vm, "r"), 321.0);
}

// ── IntMulMod / IntMulModI — fused multiply-modulo ───────────────────────────

#[test]
fn int_mul_mod_multiplies_then_reduces() {
    // a * b then p % m with both operands in registers fuses to IntMulMod.
    let vm = run("fn mulmod(a, b, m) { let p = a * b; let r = p % m; return r; }\n\
                  let r = mulmod(7, 6, 5);\n");
    assert_eq!(int(&vm, "r"), 2, "42 % 5 must be 2");
}

#[test]
fn int_mul_mod_overflow_keeps_exact_modulus() {
    // 2^62 * 4 = 2^64 promotes to BigInt; 2^64 % 1000 == 616 exactly, then
    // demotes to a plain Int.
    let vm = run("fn mulmod(a, b, m) { let p = a * b; let r = p % m; return r; }\n\
                  let r = mulmod(4611686018427387904, 4, 1000);\n");
    assert_eq!(int(&vm, "r"), 616);
}

#[test]
fn int_mul_mod_zero_modulus_errors() {
    let msg = err_msg("fn mulmod(a, b, m) { let p = a * b; let r = p % m; return r; }\n\
                       let r = mulmod(4, 5, 0);\n");
    assert!(msg.contains("Modulo by zero"), "got: {}", msg);
}

#[test]
fn mulmod_string_operands_error() {
    // Str call-site typing makes `a * b` emit IntMul (packed D_INT_MUL_RR);
    // string operands fail int_mul with ErrorCode(310) -> catalog rendering.
    let msg = err_msg("fn mulmod(a, b, m) { let p = a * b; let r = p % m; return r; }\n\
                       let r = mulmod(\"aaaa\", \"bbbb\", 5);\n");
    assert!(msg.contains("[E0310] BigInt and Number cannot be mixed"), "got: {}", msg);
}

#[test]
fn int_mul_mod_i_uses_immediate_modulus() {
    // p % 5 fuses the constant into IntModI, then IntMul+IntModI -> IntMulModI.
    let vm = run("fn mulmodi(a, b) { let p = a * b; let r = p % 5; return r; }\n\
                  let r = mulmodi(7, 6);\n");
    assert_eq!(int(&vm, "r"), 2);
}

// ── NumSqrt / NumSin / NumCos — Math intrinsics ──────────────────────────────

#[test]
fn num_sqrt_exact_values() {
    let vm = run("let r16 = Math.sqrt(16.0);\nlet r2 = Math.sqrt(2.0);\n");
    assert_eq!(num(&vm, "r16"), 4.0);
    assert_eq!(num(&vm, "r2"), 2.0f64.sqrt());
}

#[test]
fn num_sqrt_negative_is_nan() {
    let vm = run("let r = Math.sqrt(-1.0);\n");
    assert!(num(&vm, "r").is_nan(), "sqrt(-1) must be NaN");
}

#[test]
fn num_sqrt_not_numeric_errors() {
    let msg = err_msg("let r = Math.sqrt(\"aaaa\");\n");
    assert!(msg.contains("NumSqrt: src not numeric"), "got: {}", msg);
}

#[test]
fn num_sin_cos_exact_values() {
    let vm = run("let s0 = Math.sin(0.0);\nlet c0 = Math.cos(0.0);\n\
                  let sh = Math.sin(0.5);\nlet ch = Math.cos(0.5);\n");
    assert_eq!(num(&vm, "s0"), 0.0);
    assert_eq!(num(&vm, "c0"), 1.0);
    assert_eq!(num(&vm, "sh"), 0.5f64.sin());
    assert_eq!(num(&vm, "ch"), 0.5f64.cos());
}

#[test]
fn num_sin_cos_not_numeric_errors() {
    let sin_msg = err_msg("let r = Math.sin(\"aaaa\");\n");
    assert!(sin_msg.contains("NumSin: src not numeric"), "got: {}", sin_msg);
    let cos_msg = err_msg("let r = Math.cos(\"aaaa\");\n");
    assert!(cos_msg.contains("NumCos: src not numeric"), "got: {}", cos_msg);
}
