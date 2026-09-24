//! f64 lane: floating-point arithmetic with VM-oracle parity.

use std::collections::HashMap;

use hudhudscript_codegen::backend::{CodegenContext, NativeBackend, OptGoal, OptLevel};
use hudhudscript_codegen_cranelift::CraneliftBackend;
use hudhudscript_mir::{lower_function_typed, MirFunction, MirType};
use hudhudscript_native_abi::{JitExit, JIT_EXIT_RETURNED};
use hudhudscript_parser::parse;
use hudhudscript_types::lower_module;
use hudhudscript_target::TargetSpec;

type NativeEntry = unsafe extern "C" fn(u32, *const i64, *mut JitExit);

unsafe fn call_f64(addr: usize, args: &[f64]) -> JitExit {
    let f: NativeEntry = std::mem::transmute(addr);
    let i64_args: Vec<i64> = args.iter().map(|v| v.to_bits() as i64).collect();
    let mut out = JitExit::returned(0);
    f(i64_args.len() as u32, i64_args.as_ptr(), &mut out);
    out
}

fn compile_fn(src: &str, fn_name: &str, param_types: &[(&str, MirType)]) -> usize {
    let ast = parse(src).expect("parse");
    let module = lower_module(&ast).expect("AST→HIR");
    let hir = module.functions.get(fn_name).unwrap_or_else(|| panic!("{fn_name} not found"));
    let types: HashMap<String, MirType> = param_types.iter()
        .map(|(n, t)| (n.to_string(), *t))
        .collect();
    let mir = lower_function_typed(hir, &types).expect("HIR→MIR");

    let mut backend = CraneliftBackend::new();
    let target = TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O2,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    backend.compile_function(&mir, &ctx).expect("compile").address.expect("addr")
}

fn result_as_f64(out: &JitExit) -> f64 {
    f64::from_bits(out.value as u64)
}

#[test]
fn f64_add_matches_vm() {
    let src = "function addf(a, b) { return a + b }";
    let addr = compile_fn(src, "addf", &[("a", MirType::F64), ("b", MirType::F64)]);
    for (a, b) in [(1.5f64, 2.5), (0.0, 0.0), (-1.5, 1.5), (3.14159, 2.71828)] {
        let out = unsafe { call_f64(addr, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED);
        let native = result_as_f64(&out);
        assert!((native - (a + b)).abs() < 1e-12, "a={a} b={b}: native={native}");
    }
}

#[test]
fn f64_mul_sub_div_match() {
    let mul = compile_fn(
        "function mulf(a, b) { return a * b }", "mulf",
        &[("a", MirType::F64), ("b", MirType::F64)]);
    let out = unsafe { call_f64(mul, &[2.5, 4.0]) };
    assert!((result_as_f64(&out) - 10.0).abs() < 1e-12);

    let sub = compile_fn(
        "function subf(a, b) { return a - b }", "subf",
        &[("a", MirType::F64), ("b", MirType::F64)]);
    let out = unsafe { call_f64(sub, &[10.5, 3.5]) };
    assert!((result_as_f64(&out) - 7.0).abs() < 1e-12);

    let div = compile_fn(
        "function divf(a, b) { return a / b }", "divf",
        &[("a", MirType::F64), ("b", MirType::F64)]);
    let out = unsafe { call_f64(div, &[10.0, 4.0]) };
    assert!((result_as_f64(&out) - 2.5).abs() < 1e-12);
}

#[test]
fn f64_mixed_int_float() {
    // int literal + float param → float sonucu
    let src = "function mixed(a) { return a + 10 }";
    let addr = compile_fn(src, "mixed", &[("a", MirType::F64)]);
    let out = unsafe { call_f64(addr, &[1.5]) };
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    let native = result_as_f64(&out);
    assert!((native - 11.5).abs() < 1e-12, "1.5 + 10 = 11.5, got {native}");
}

#[test]
fn f64_comparison_in_if() {
    let src = "function compare(a, b) { if (a > b) { return 1.0 } return 0.0 }";
    let addr = compile_fn(src, "compare", &[("a", MirType::F64), ("b", MirType::F64)]);
    let out1 = unsafe { call_f64(addr, &[2.5, 1.5]) };
    assert!((result_as_f64(&out1) - 1.0).abs() < 1e-12);
    let out2 = unsafe { call_f64(addr, &[1.5, 2.5]) };
    assert!((result_as_f64(&out2) - 0.0).abs() < 1e-12);
}

#[test]
fn f64_chain_arithmetic() {
    let src = "function chain(a) { let x = a * 2.0; let y = x + 1.5; return y / 3.0 }";
    let addr = compile_fn(src, "chain", &[("a", MirType::F64)]);
    let out = unsafe { call_f64(addr, &[3.0]) };
    // (3.0 * 2.0 + 1.5) / 3.0 = 7.5 / 3.0 = 2.5
    assert!((result_as_f64(&out) - 2.5).abs() < 1e-12, "got {}", result_as_f64(&out));
}
