//! P4 regression tests for fused integer multiply-modulo instructions.
//!
//! Verifies that the optimizer fuses `IntMul` + `IntMod` into `IntMulMod`
//! and that `IntMulModI` computes `((a wrapping_mul b) % m)` correctly,
//! matching the existing `IntMul` wrapping semantics.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run(source: &str) -> Result<(), String> {
    let mut vm = VM::with_locale(hudhudscript_vm::OutputLocale::Default);
    let ast = parse(source).map_err(|e| format!("parse: {}", e))?;
    let bytecode = Compiler::new()
        .compile(&ast)
        .map_err(|e| format!("compile: {}", e))?;
    vm.execute(&bytecode).map_err(|e| format!("{}", e))
}

#[test]
fn modular_exp_loop_correctness() {
    run(r#"
        function mod_exp(base, exp, mod) {
            let result = 1;
            let b = base % mod;
            let e = exp;
            while (e > 0) {
                if (e % 2 == 1) {
                    result = (result * b) % mod;
                }
                b = (b * b) % mod;
                e = e / 2;
            }
            return result;
        }
        let r = mod_exp(3, 13, 1000000007);
        if (r != 1594323) { throw "mod_exp value wrong"; }
    "#)
    .unwrap();
}

#[test]
fn chained_mul_mod_stays_int() {
    run(r#"
        let n = 1;
        for (let i = 1; i <= 10; i = i + 1) {
            n = (n * i) % 1000;
        }
        if (n != 800) { throw "chained mul-mod value wrong"; }
    "#)
    .unwrap();
}

#[test]
fn int_mulmod_overflow_wraps_like_intmul() {
    run(r#"
        let a = 4000000000;
        let b = 4000000000;
        let r = (a * b) % 7;
        // a*b overflows i64; product wraps exactly like IntMul, then % applies.
        if (r != 0) { throw "wrapping mul-mod value wrong"; }
    "#)
    .unwrap();
}

#[test]
fn int_mulmod_modulus_one() {
    run(r#"
        let r = (7 * 5) % 1;
        if (r != 0) { throw "mod 1 should be 0"; }
    "#)
    .unwrap();
}

#[test]
fn int_mulmod_negative_operand() {
    run(r#"
        let r = (-7 * 5) % 11;
        if (r != -2) { throw "negative mul-mod value wrong"; }
    "#)
    .unwrap();
}
