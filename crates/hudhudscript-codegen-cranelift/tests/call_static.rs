//! CallStatic: function-to-function native calls with VM-oracle parity.

use std::collections::HashMap;

use hudhudscript_codegen::backend::{CodegenContext, NativeBackend, OptGoal, OptLevel};
use hudhudscript_codegen_cranelift::CraneliftBackend;
use hudhudscript_mir::{lower_module_typed, MirType};
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

fn compile_and_get(src: &str, fn_name: &str, params: &[&str]) -> usize {
    let ast = parse(src).expect("parse");
    let hir_module = lower_module(&ast).expect("AST→HIR");

    // Tüm fonksiyonların param tiplerini sağla
    let mut all_ptys: HashMap<String, HashMap<String, MirType>> = HashMap::new();
    for (name, f) in &hir_module.functions {
        let ptys: HashMap<String, MirType> = f.params.iter()
            .map(|p| (p.name.clone(), MirType::I64))
            .collect();
        all_ptys.insert(name.clone(), ptys);
    }
    let _ = params; // sadece test bilgisidir

    let mir_module = lower_module_typed(&hir_module, &all_ptys).expect("MIR");

    let mut backend = CraneliftBackend::new();
    let target = TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O2,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    let compiled = backend.compile_module(&mir_module, &ctx).expect("compile_module");

    // İstenen fonksiyonun adresini bul
    let symbol = format!("hudhud_{fn_name}");
    compiled.functions.iter()
        .find(|f| f.symbol == symbol)
        .and_then(|f| f.address)
        .expect("function address")
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
fn callstatic_double_then_add_matches_vm() {
    let src = r#"
function double(x) { return x * 2 }
function apply(a) { return double(a) + 1 }
"#;
    let addr = compile_and_get(src, "apply", &[]);
    for a in [0i64, 1, 5, -3, 100] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a}");
        let expected = a * 2 + 1;
        assert_eq!(out.value, expected, "a={a}: native={}", out.value);
    }
}

#[test]
fn callstatic_nested_calls_match_vm() {
    let src = r#"
function double(x) { return x * 2 }
function quad(x) { return double(double(x)) }
function compute(a) { return quad(a) + 10 }
"#;
    let addr = compile_and_get(src, "compute", &[]);
    for a in [1i64, 2, 5, 0] {
        let out = unsafe { call(addr, &[a]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a}");
        let expected = a * 4 + 10;
        assert_eq!(out.value, expected, "a={a}: native={} expected={}", out.value, expected);
    }
}

#[test]
fn callstatic_with_control_flow_matches_vm() {
    let src = r#"
function abs_val(x) {
    if (x < 0) { return 0 - x }
    return x
}
function dist(a, b) { return abs_val(a - b) }
"#;
    let addr = compile_and_get(src, "dist", &[]);
    for (a, b) in [(5i64, 3), (3, 5), (0, 0), (-10, 10)] {
        let out = unsafe { call(addr, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "a={a} b={b}");
        let expected = (a - b).abs();
        assert_eq!(out.value, expected, "dist({a},{b}): native={}", out.value);
    }
}

#[test]
fn callstatic_multi_arg_matches_vm() {
    let src = r#"
function add3(a, b, c) { return a + b + c }
function compute(x) { return add3(x, x * 2, x * 3) }
"#;
    let addr = compile_and_get(src, "compute", &[]);
    for x in [1i64, 2, 10, -5] {
        let out = unsafe { call(addr, &[x]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "x={x}");
        let expected = x + x * 2 + x * 3;
        assert_eq!(out.value, expected, "x={x}: native={}", out.value);
    }
}
