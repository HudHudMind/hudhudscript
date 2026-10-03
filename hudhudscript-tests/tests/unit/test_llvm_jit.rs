//! Tests for hudhudscript-codegen-llvm — MCJIT in-process execution
//! (moved from crates/hudhudscript-codegen-llvm/src/lib.rs `mod jit_tests`).
//!
//! Needs LLVM 14 on the host (llvm-14-dev; `LLVM_SYS_140_PREFIX`); gated
//! to Unix like the original `#[cfg(all(test, unix))]` inline module.
//! ALSO gated behind the suite's `llvm` feature: MCJIT resolves host
//! helper symbols (hudhud_select_i64, …) from the process dynsym, which
//! requires the test binary to be linked with --export-dynamic; the
//! suite's build.rs adds that flag only when the feature is on, so the
//! default `cargo test --workspace` gate stays hermetic.

#![cfg(all(unix, feature = "llvm"))]

use std::collections::HashMap;

use hudhudscript_codegen::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_codegen_llvm::compile_jit;
use hudhudscript_mir::{lower_module_typed, MirType};
use hudhudscript_parser::parse;
use hudhudscript_types::lower_module_with_init;

#[test]
fn llvm_jit_runs_entry() {
    // top-level print: init çalışır, değer üretmez (value=0), status RETURNED.
    // (eski sürüm 42 bekliyordu — print(2+3) hiçbir zaman 42 üretemez, bayat test)
    let src = "print(2 + 3)";
    let ast = parse(src).unwrap();
    let hir = lower_module_with_init(&ast).unwrap();
    let mut ptys: HashMap<String, HashMap<String, MirType>> = HashMap::new();
    for (name, f) in &hir.functions {
        let m: HashMap<String, MirType> = f.params.iter().map(|p| (p.name.clone(), MirType::I64)).collect();
        ptys.insert(name.clone(), m);
    }
    let mir = lower_module_typed(&hir, &ptys).unwrap();
    let target = hudhudscript_target::TargetSpec::host();
    let ctx = CodegenContext { target: &target, opt: OptLevel::O2, opt_goal: OptGoal::Speed, debug_info: false, abi_version: 1 };
    let fns = compile_jit(&mir, &ctx).expect("jit");
    let addr = fns.iter().find(|(s, _)| s == "hudhud__hudhud_init").expect("entry").1;
    let f: unsafe extern "C" fn(u32, *const i64, *mut hudhudscript_native_abi::JitExit) = unsafe { std::mem::transmute(addr) };
    let mut out = hudhudscript_native_abi::JitExit::returned(0);
    unsafe { f(0, std::ptr::null(), &mut out) };
    assert_eq!(out.status, hudhudscript_native_abi::JIT_EXIT_RETURNED);
}
