//! Control flow: if/else with early returns (multi-block CFG) — VM parity.
//! While and If-merge-with-mutation are PHI-gated (cleanly rejected).

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

fn vm_i64(src: &str) -> i64 {
    use hudhudscript_compiler::Compiler;
    use hudhudscript_vm::VM;
    let ast = parse(src).expect("parse");
    let bc = Compiler::new().compile(&ast).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    vm.get_variable_owned("x").and_then(|v| v.as_int()).expect("x as i64")
}

#[test]
fn if_else_early_return_matches_vm() {
    let src = r#"
function classify(a) {
    if (a > 10) { return 1 } else { return 0 }
}
"#;
    let mir = compile_function_source(src, "classify", &["a"]);
    let addr = compile(&mir);
    for a in [5i64, 10, 11, 100, -5] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a}");
        let vm = vm_i64(&format!(
            "function classify(a) {{ if (a > 10) {{ return 1 }} else {{ return 0 }} }}\nlet x = classify({a})"
        ));
        assert_eq!(out.value, vm, "a={a}: native={} vm={}", out.value, vm);
    }
}

#[test]
fn nested_if_early_return_matches_vm() {
    let src = r#"
function grade(a) {
    if (a >= 90) { return 4 }
    if (a >= 80) { return 3 }
    if (a >= 70) { return 2 }
    return 1
}
"#;
    let mir = compile_function_source(src, "grade", &["a"]);
    let addr = compile(&mir);
    for a in [95i64, 85, 75, 65, 90, 80, 70, 0] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a}");
        let vm = vm_i64(&format!(
            "function grade(a) {{ if (a >= 90) {{ return 4 }}
if (a >= 80) {{ return 3 }}
if (a >= 70) {{ return 2 }}
return 1 }}\nlet x = grade({a})"
        ));
        assert_eq!(out.value, vm, "a={a}: native={} vm={}", out.value, vm);
    }
}

#[test]
fn if_no_else_early_return_matches_vm() {
    let src = r#"
function positive(a) {
    if (a > 100) { return a }
    return 0
}
"#;
    let mir = compile_function_source(src, "positive", &["a"]);
    let addr = compile(&mir);
    for a in [50i64, 150, 0, 100, -10] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a}");
        let vm = vm_i64(&format!(
            "function positive(a) {{ if (a > 100) {{ return a }}
return 0 }}\nlet x = positive({a})"
        ));
        assert_eq!(out.value, vm, "a={a}: native={} vm={}", out.value, vm);
    }
}

#[test]
fn if_with_comparison_chain_matches_vm() {
    let src = r#"
function triage(a, b) {
    if (a > b) {
        if (a > 100) { return 3 }
        return 2
    }
    return 1
}
"#;
    let mir = compile_function_source(src, "triage", &["a", "b"]);
    let addr = compile(&mir);
    for (a, b) in [(150i64, 50), (80, 50), (30, 50), (50, 50)] {
        let out = unsafe { call(addr, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a} b={b}");
        let vm = vm_i64(&format!(
            "function triage(a, b) {{ if (a > b) {{ if (a > 100) {{ return 3 }}
return 2 }}
return 1 }}\nlet x = triage({a}, {b})"
        ));
        assert_eq!(out.value, vm, "a={a} b={b}: native={} vm={}", out.value, vm);
    }
}

// ── Phi-gated (dürüst reddetme) ──

#[test]
fn while_loop_phi_matches_vm() {
    let src = r#"
function sumTo(n) {
    let i = 0
    let total = 0
    while (i < n) { total = total + i; i = i + 1 }
    return total
}
"#;
    let mir = compile_function_source(src, "sumTo", &["n"]);
    let addr = compile(&mir);
    for n in [0i64, 1, 5, 10, 100] {
        let out = unsafe { call(addr, &[n]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "n={n}");
        let vm = vm_i64(&format!(
            "function sumTo(n) {{\nlet i = 0\nlet total = 0\nwhile (i < n) {{ total = total + i; i = i + 1 }}\nreturn total\n}}\nlet x = sumTo({n})"
        ));
        assert_eq!(out.value, vm, "n={n}: native={} vm={}", out.value, vm);
    }
}

#[test]
fn if_merge_mutation_phi_matches_vm() {
    let src = r#"
function withDefault(a) {
    let result = 0
    if (a > 0) { result = a * 2 }
    return result
}
"#;
    let mir = compile_function_source(src, "withDefault", &["a"]);
    let addr = compile(&mir);
    for a in [-5i64, 0, 3, 7, 100] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a}");
        let vm = vm_i64(&format!(
            "function withDefault(a) {{\nlet result = 0\nif (a > 0) {{ result = a * 2 }}\nreturn result\n}}\nlet x = withDefault({a})"
        ));
        assert_eq!(out.value, vm, "a={a}: native={} vm={}", out.value, vm);
    }
}
