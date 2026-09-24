//! Local variable bindings: `let x = a + b; return x` native koşar ve
//! VM oracle'la birebir aynı sonucu üretir.

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
fn let_then_return_matches_vm() {
    let src = "function compute(a) { let x = a + 10; return x }";
    let mir = compile_function_source(src, "compute", &["a"]);
    let addr = compile(&mir);
    for a in [5i64, -3, 0, 100] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a}");
        let vm = vm_i64(&format!(
            "function compute(a) {{ let x = a + 10; return x }}\nlet x = compute({a})"
        ));
        assert_eq!(out.value, vm, "a={a}: native={} vm={}", out.value, vm);
    }
}

#[test]
fn chained_lets_match_vm() {
    let src = "function chain(a) { let x = a + 10; let y = x * 2; let z = y - 5; return z }";
    let mir = compile_function_source(src, "chain", &["a"]);
    let addr = compile(&mir);
    for a in [1i64, 7, -20, 0] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED);
        let vm = vm_i64(&format!(
            "function chain(a) {{ let x = a + 10; let y = x * 2; let z = y - 5; return z }}\nlet x = chain({a})"
        ));
        assert_eq!(out.value, vm, "a={a}");
    }
}

#[test]
fn assign_rebinding_matches_vm() {
    let src = "function rebind(a) { let x = a; x = x + 1; x = x * 3; return x }";
    let mir = compile_function_source(src, "rebind", &["a"]);
    let addr = compile(&mir);
    for a in [0i64, 4, -1] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED);
        let vm = vm_i64(&format!(
            "function rebind(a) {{ let x = a; x = x + 1; x = x * 3; return x }}\nlet x = rebind({a})"
        ));
        assert_eq!(out.value, vm, "a={a}");
    }
}

#[test]
fn mixed_arithmetic_with_locals_matches_vm() {
    let src = r#"
function calc(a, b) {
    let sum = a + b
    let diff = a - b
    let prod = sum * diff
    let result = prod / 2
    return result
}
"#;
    let mir = compile_function_source(src, "calc", &["a", "b"]);
    let addr = compile(&mir);
    for (a, b) in [(10i64, 4), (20, 5), (-6, 3), (100, 7)] {
        let out = unsafe { call(addr, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "{a},{b}");
        let vm = vm_i64(&format!(
            "function calc(a, b) {{ let sum = a + b; let diff = a - b; let prod = sum * diff; let result = prod / 2; return result }}\nlet x = calc({a}, {b})"
        ));
        assert_eq!(out.value, vm, "a={a} b={b}: native={} vm={}", out.value, vm);
    }
}
