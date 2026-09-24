//! Parameter + arithmetic lanes with VM-oracle parity
//! (JIT_AOT_ARCHITECTURE §5.3, §18, §22).

use std::collections::HashMap;

use hudhudscript_codegen::backend::{CodegenContext, NativeBackend, OptGoal, OptLevel};
use hudhudscript_codegen_cranelift::CraneliftBackend;
use hudhudscript_mir::{lower_function_typed, MirFunction, MirType};
use hudhudscript_native_abi::{JitExit, JIT_EXIT_DIV_ZERO, JIT_EXIT_OVERFLOW, JIT_EXIT_RETURNED};
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

/// Kaynak → parse → AST→HIR → tipli MIR (paramlar i64) → native derle.
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

/// VM oracle: `let x = <expr_source>` değerini gerçek motordan oku.
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
fn typed_add_e2e_matches_vm() {
    let src = "function add(a, b) { return a + b }";
    let mir = compile_function_source(src, "add", &["a", "b"]);
    let addr = compile(&mir);
    for (a, b) in [(20i64, 22), (-5, 5), (0, 0), (i64::MAX - 1, 1)] {
        let out = unsafe { call(addr, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "{a}+{b}");
        let vm = vm_i64(&format!("function add(a, b) {{ return a + b }}\nlet x = add({a}, {b})"));
        assert_eq!(out.value, vm, "parity broken for {a}+{b}");
    }
}

#[test]
fn typed_add_overflow_lane_split() {
    let src = "function add(a, b) { return a + b }";
    let mir = compile_function_source(src, "add", &["a", "b"]);
    let addr = compile(&mir);
    let out = unsafe { call(addr, &[i64::MAX, 1]) };
    // VM paritesi (F13 düzeltmesi): promote edilen taşma ÇÖZÜLÜR — BigInt
    // değeri status RETURNED ile döner (eski OVERFLOW status'ü aynı programı
    // motor bazında farklı sonuca götürüyordu).
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_typeof(out.value)) }
            .to_str()
            .unwrap(),
        "bigint",
        "değer BigInt handle olmalı"
    );

    // VM aynı girdiyi BigInt'e yükseltir (i64 DEĞİLDİR) — şerit kanıtı.
    use hudhudscript_compiler::Compiler;
    use hudhudscript_vm::VM;
    let src_vm = "function add(a, b) { return a + b }\nlet x = add(9223372036854775807, 1)";
    let ast = parse(src_vm).unwrap();
    let bc = Compiler::new().compile(&ast).unwrap();
    let mut vm = VM::new();
    vm.execute(&bc).unwrap();
    assert!(vm.get_variable_owned("x").and_then(|v| v.as_int()).is_none(), "VM BigInt'e yükseltmeli");
}

#[test]
fn sub_and_mul_lanes_match_vm() {
    let sub = compile_function_source(
        "function sub(a, b) { return a - b }", "sub", &["a", "b"],
    );
    let addr_sub = compile(&sub);
    for (a, b) in [(10i64, 4), (0, 7), (-3, -9)] {
        let out = unsafe { call(addr_sub, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED);
        assert_eq!(out.value, vm_i64(&format!("function sub(a,b){{return a-b}}\nlet x = sub({a}, {b})")));
    }

    let mul = compile_function_source(
        "function mul(a, b) { return a * b }", "mul", &["a", "b"],
    );
    let addr_mul = compile(&mul);
    for (a, b) in [(6i64, 7), (-3, 9), (0, 99)] {
        let out = unsafe { call(addr_mul, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED);
        assert_eq!(out.value, vm_i64(&format!("function mul(a,b){{return a*b}}\nlet x = mul({a}, {b})")));
    }
}

#[test]
fn mul_overflow_lane() {
    let mul = compile_function_source(
        "function mul(a, b) { return a * b }", "mul", &["a", "b"],
    );
    let addr = compile(&mul);
    let big = 3037000500i64; // 3037000500^2 > i64::MAX
    let out = unsafe { call(addr, &[big, big]) };
    // F13: mul promote de ÇÖZÜLÜR — BigInt değer + RETURNED (VM paritesi)
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_typeof(out.value)) }
            .to_str()
            .unwrap(),
        "bigint",
        "değer BigInt handle olmalı"
    );
}

#[test]
fn div_rem_lanes_match_vm() {
    let div = compile_function_source(
        "function mydiv(a, b) { return a / b }", "mydiv", &["a", "b"],
    );
    let addr_div = compile(&div);
    for (a, b) in [(7i64, 2), (-7, 2), (100, 10), (i64::MIN, 1)] {
        let out = unsafe { call(addr_div, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "{a}/{b}");
        assert_eq!(
            out.value,
            vm_i64(&format!("function mydiv(a,b){{return a/b}}\nlet x = mydiv({a}, {b})")),
            "{a}/{b}"
        );
    }

    let rem = compile_function_source(
        "function myrem(a, b) { return a % b }", "myrem", &["a", "b"],
    );
    let addr_rem = compile(&rem);
    for (a, b) in [(7i64, 3), (-7, 3), (10, 5)] {
        let out = unsafe { call(addr_rem, &[a, b]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED);
        assert_eq!(
            out.value,
            vm_i64(&format!("function myrem(a,b){{return a%b}}\nlet x = myrem({a}, {b})"))
        );
    }
}

#[test]
fn div_by_zero_lane_matches_vm_error() {
    let div = compile_function_source(
        "function mydiv(a, b) { return a / b }", "mydiv", &["a", "b"],
    );
    let addr = compile(&div);

    // JIT: DivZero çıkışı
    let out = unsafe { call(addr, &[1, 0]) };
    assert_eq!(out.status, JIT_EXIT_DIV_ZERO);

    // VM: çalışma-zamanı hatası (aynı §18 sınıf)
    use hudhudscript_compiler::Compiler;
    use hudhudscript_vm::VM;
    let src = "function mydiv(a,b){return a/b}\nlet x = mydiv(1, 0)";
    let ast = parse(src).unwrap();
    let bc = Compiler::new().compile(&ast).unwrap();
    let mut vm = VM::new();
    let err = vm.execute(&bc);
    assert!(err.is_err(), "VM div-by-zero hata üretmeli");
}

#[test]
fn min_div_neg1_overflow_lane() {
    // F13/VM paritesi (v0.9.21): MIN/-1 promote edilir — VM oracle
    // BigInt 2^63 üretir, status OVERFLOW DEĞİL RETURNED'dir.
    let div = compile_function_source(
        "function mydiv(a, b) { return a / b }", "mydiv", &["a", "b"],
    );
    let addr = compile(&div);
    let out = unsafe { call(addr, &[i64::MIN, -1]) };
    assert_eq!(out.status, JIT_EXIT_RETURNED, "value={}", out.value);
    let s = unsafe {
        std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_int_to_string(out.value))
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(s, "9223372036854775808");
}
