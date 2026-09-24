//! Uniform native-entry ABI + checked i64 Add lane
//! (JIT_AOT_ARCHITECTURE §6.2/§18) — including the lane-split parity
//! against the VM oracle: JIT signals Overflow where the VM promotes to
//! BigInt (both are the specified §18 behaviors; the replay bridge
//! arrives with the trampoline lane).

use hudhudscript_codegen::backend::{CodegenContext, NativeBackend, OptGoal, OptLevel};
use hudhudscript_codegen_cranelift::CraneliftBackend;
use hudhudscript_mir::builder::{BinOp, MirFunctionBuilder};
use hudhudscript_mir::MirFunction;
use hudhudscript_mir::MirType;
use hudhudscript_native_abi::{JitExit, JIT_EXIT_OVERFLOW, JIT_EXIT_RETURNED};
use hudhudscript_target::TargetSpec;

type NativeEntry = unsafe extern "C" fn(u32, *const i64, *mut JitExit);

unsafe fn call(addr: usize, args: &[i64]) -> JitExit {
    let f: NativeEntry = std::mem::transmute(addr);
    let mut out = JitExit::returned(0);
    f(args.len() as u32, args.as_ptr(), &mut out);
    out
}

fn ctx<'a>(target: &'a TargetSpec) -> CodegenContext<'a> {
    CodegenContext { target, opt: OptLevel::O2, opt_goal: OptGoal::Speed, debug_info: false, abi_version: 1 }
}

fn five() -> MirFunction {
    let mut b = MirFunctionBuilder::new("five", vec![], MirType::I64);
    let e = b.entry();
    let v = b.const_i64(e, 5);
    b.ret(e, v);
    b.finish()
}

fn add_of(a: i64, rhs: i64) -> MirFunction {
    let mut b = MirFunctionBuilder::new("addc", vec![], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, a);
    let y = b.const_i64(e, rhs);
    let s = b.bin(e, BinOp::Add, MirType::I64, x, y);
    b.ret(e, s);
    b.finish()
}

#[test]
fn five_through_uniform_entry() {
    let mut backend = CraneliftBackend::new();
    let target = TargetSpec::host();
    let compiled = backend.compile_function(&five(), &ctx(&target)).expect("compile");
    let out = unsafe { call(compiled.address.unwrap(), &[]) };
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(out.value, 5);
}

#[test]
fn checked_add_within_range() {
    for (a, b) in [(2i64, 3), (-7, 7), (0, 0), (i64::MAX - 1, 1), (i64::MIN + 1, -1)] {
        let mut backend = CraneliftBackend::new();
        let target = TargetSpec::host();
        let compiled = backend.compile_function(&add_of(a, b), &ctx(&target)).expect("compile");
        let out = unsafe { call(compiled.address.unwrap(), &[]) };
        assert_eq!(out.status, JIT_EXIT_RETURNED, "{a}+{b}");
        assert_eq!(out.value, a.checked_add(b).unwrap(), "{a}+{b}");
    }
}

#[test]
fn checked_add_overflow_signals_exit_lane() {
    let mut backend = CraneliftBackend::new();
    let target = TargetSpec::host();
    let compiled = backend.compile_function(&add_of(i64::MAX, 1), &ctx(&target)).expect("compile");
    let out = unsafe { call(compiled.address.unwrap(), &[]) };
    // F13: promote edilen ta\u015fma \u00c7\u00d6Z\u00dcL\u00dcR \u2014 BigInt de\u011fer + RETURNED (VM paritesi;
    // eski OVERFLOW status F06 default'unda sahte s\u00fcre\u00e7 hatas\u00fc \u00fcretiyordu)
    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_typeof(out.value)) }
            .to_str().unwrap(),
        "bigint"
    );
}

#[test]
fn overflow_lane_split_matches_vm_oracle() {
    // VM (oracle): i64::MAX + 1 → BigInt'e yükselir (i64 DEĞİLDİR).
    use hudhudscript_compiler::Compiler;
    use hudhudscript_parser::parse;
    use hudhudscript_vm::VM;

    let source = "let x = 9223372036854775807 + 1";
    let ast = parse(source).expect("parse");
    let bc = Compiler::new().compile(&ast).expect("compile");
    let mut vm = VM::new();
    vm.execute(&bc).expect("execute");
    let as_i64 = vm.get_variable_owned("x").and_then(|v| v.as_int());

    // JIT tarafı: Overflow çıkışı (replay köprüsü trambulin şeridinde)
    let mut backend = CraneliftBackend::new();
    let target = TargetSpec::host();
    let compiled = backend.compile_function(&add_of(i64::MAX, 1), &ctx(&target)).expect("compile");
    let out = unsafe { call(compiled.address.unwrap(), &[]) };

    assert_eq!(out.status, JIT_EXIT_RETURNED);
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(hudhudscript_native_abi::hudhud_typeof(out.value)) }
            .to_str().unwrap(),
        "bigint"
    );
    assert!(as_i64.is_none(), "VM must promote to BigInt (non-i64), got {as_i64:?}");
}

#[test]
fn load_lane_still_rejected_cleanly() {
    use hudhudscript_mir::{LocalId, MirInst, ValueId};
    let mut b = MirFunctionBuilder::new("loader", vec![], MirType::I64);
    let e = b.entry();
    let x = b.const_i64(e, 0);
    // Load komutunu elle ekle (builder'da load API'si yok — IR seviyesinde reddi test ediyoruz)
    let mut f = b.finish();
    f.blocks[0].insts.push(MirInst::Load {
        dst: ValueId(99),
        ty: MirType::I64,
        local: LocalId(0),
    });
    f.blocks[0].terminator = Some(hudhudscript_mir::MirTerminator::Return(x));

    let mut backend = CraneliftBackend::new();
    let target = TargetSpec::host();
    let err = backend.compile_function(&f, &ctx(&target)).unwrap_err();
    assert_eq!(err.code, "UNSUPPORTED_MIR");
    assert!(err.message.contains("Load"), "must name Load: {err}");
}
