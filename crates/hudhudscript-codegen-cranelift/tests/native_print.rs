//! Native print: JIT-compiled code calls hudhud_print_int and produces
//! real stdout output.

use std::collections::HashMap;

use hudhudscript_codegen::backend::{CodegenContext, NativeBackend, OptGoal, OptLevel};
use hudhudscript_codegen_cranelift::CraneliftBackend;
use hudhudscript_mir::{lower_function_typed, MirFunction, MirType};
use hudhudscript_native_abi::{JitExit, JIT_EXIT_RETURNED};
use hudhudscript_parser::parse;
use hudhudscript_types::lower_module;
use hudhudscript_target::TargetSpec;

type NativeEntry = unsafe extern "C" fn(u32, *const i64, *mut JitExit);

unsafe fn call(addr: usize, args: &[i64]) -> JitExit {
    let f: NativeEntry = std::mem::transmute(addr);
    let mut out = JitExit::returned(0);
    f(args.len() as u32, args.as_ptr(), &mut out);
    out
}

fn compile_function_source(src: &str, fn_name: &str, param_names: &[&str]) -> MirFunction {
    let ast = parse(src).expect("parse");
    let module = lower_module(&ast).expect("AST→HIR");
    let hir = module.functions.get(fn_name).unwrap_or_else(|| panic!("{fn_name} not found"));
    let types: HashMap<String, MirType> = param_names
        .iter()
        .map(|n| (n.to_string(), MirType::I64))
        .collect();
    lower_function_typed(hir, &types).expect("HIR→MIR")
}

fn compile(mir: &MirFunction) -> usize {
    let mut backend = CraneliftBackend::new();
    let target = TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O2,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    backend.compile_function(mir, &ctx).expect("compile").address.expect("addr")
}

#[test]
fn native_print_hello() {
    // print(42) — en basit native print
    let src = "function main() { print(42) }";
    let mir = compile_function_source(src, "main", &[]);
    let addr = compile(&mir);
    let out = unsafe { call(addr, &[]) };
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    // print değersiz → JitExit.value = 0 (null)
    assert_eq!(out.value, 0);
}

#[test]
fn native_print_computed() {
    // print(6 * 7) — hesaplanmış değer
    let src = "function main(a, b) { print(a * b) }";
    let mir = compile_function_source(src, "main", &["a", "b"]);
    let addr = compile(&mir);
    let out = unsafe { call(addr, &[6, 7]) };
    assert_eq!(out.status, JIT_EXIT_RETURNED);
}

#[test]
fn native_print_and_return() {
    // print + return birlikte
    let src = "function compute(a) { print(a + 100); return a * 2 }";
    let mir = compile_function_source(src, "compute", &["a"]);
    let addr = compile(&mir);
    let out = unsafe { call(addr, &[21]) };
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 42); // 21 * 2
}

#[test]
fn native_print_in_loop() {
    // While döngüsü içinde print
    let src = r#"
function countdown(n) {
    let i = n
    while (i > 0) { print(i); i = i - 1 }
    return 0
}
"#;
    let mir = compile_function_source(src, "countdown", &["n"]);
    let addr = compile(&mir);
    let out = unsafe { call(addr, &[3]) };
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 0);
}
