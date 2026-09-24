//! CLIF emit yardımcıları: checked aritmetik, çıkış yazımı, operand/env erişimi.

use cranelift::prelude::codegen::ir::condcodes::IntCC;
use cranelift::prelude::types::{F64, I32, I64};
use cranelift::prelude::{FunctionBuilder, InstBuilder, MemFlags, Value};

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{MirFunction, MirInst, MirType};
use hudhudscript_native_abi::{JIT_EXIT_DIV_ZERO, JIT_EXIT_OVERFLOW, JIT_EXIT_RETURNED};

/// JitExit { status: i32 @0, value: i64 @8 }
pub(super) const OFF_STATUS: i32 = 0;
pub(super) const OFF_VALUE: i32 = 8;

/// Return çıkışı: bayrak select zinciri + status/value store + return
pub(super) fn emit_return_exit(
    builder: &mut FunctionBuilder,
    out_ptr: Value,
    val: Value,
    ty: MirType,
    func: &MirFunction,
    ov_acc: Option<Value>,
    dz_acc: Option<Value>,
) -> Result<(), BackendError> {
    let as_i64 = match ty {
        // Bool şeridi i64 0|1 olarak yaşar — uextend gerekmez (ve geçersiz)
        MirType::I64 | MirType::Bool | MirType::Generic | MirType::Ref(_) => val,
        MirType::F64 => bitcast_f64_to_i64(builder, val),
        other => return Err(reject(func, "Return", &format!("type {other} cannot cross the i64 exit yet"))),
    };

    let st_returned = builder.ins().iconst(I32, JIT_EXIT_RETURNED as i64);
    let st_overflow = builder.ins().iconst(I32, JIT_EXIT_OVERFLOW as i64);
    let st_divzero = builder.ins().iconst(I32, JIT_EXIT_DIV_ZERO as i64);

    let status = match (ov_acc, dz_acc) {
        (Some(ov), Some(dz)) => {
            let not_dz = builder.ins().select(ov, st_overflow, st_returned);
            builder.ins().select(dz, st_divzero, not_dz)
        }
        (Some(ov), None) => builder.ins().select(ov, st_overflow, st_returned),
        (None, Some(dz)) => builder.ins().select(dz, st_divzero, st_returned),
        (None, None) => st_returned,
    };

    builder.ins().store(MemFlags::new(), status, out_ptr, OFF_STATUS);
    builder.ins().store(MemFlags::new(), as_i64, out_ptr, OFF_VALUE);
    builder.ins().return_(&[]);
    Ok(())
}

pub(super) enum OverflowKind { Add, Sub, Mul }

/// §18 checked signed op (branch-free): taşma XOR/işaret analiziyle
/// tespit edilir, (sonuç, overflow_bool) döner. Bayraklar birikir;
/// çıkışta select ile status seçilir. Sonuç asla wrap etmez.
pub(super) fn emit_checked(
    builder: &mut FunctionBuilder,
    l: Value,
    r: Value,
    kind: OverflowKind,
) -> (Value, Value) {
    let zero_c = builder.ins().iconst(I64, 0);
    match kind {
        OverflowKind::Add => {
            let sum = builder.ins().iadd(l, r);
            let ax = builder.ins().bxor(l, sum);
            let bx = builder.ins().bxor(r, sum);
            let ov_bits = builder.ins().band(ax, bx);
            let ov = builder.ins().icmp(IntCC::SignedLessThan, ov_bits, zero_c);
            (sum, ov)
        }
        OverflowKind::Sub => {
            let diff = builder.ins().isub(l, r);
            let ab = builder.ins().bxor(l, r);
            let ad = builder.ins().bxor(l, diff);
            let ov_bits = builder.ins().band(ab, ad);
            let ov = builder.ins().icmp(IntCC::SignedLessThan, ov_bits, zero_c);
            (diff, ov)
        }
        OverflowKind::Mul => {
            let lo = builder.ins().imul(l, r);
            let hi = builder.ins().smulhi(l, r);
            let sign = builder.ins().sshr_imm(lo, 63);
            let ov = builder.ins().icmp(IntCC::NotEqual, hi, sign);
            (lo, ov)
        }
    }
}

pub(super) fn record(env: &mut Vec<Option<(Value, MirType)>>, id: hudhudscript_mir::ValueId, v: Value, ty: MirType) {
    let idx = id.0 as usize;
    while env.len() <= idx {
        env.push(None);
    }
    env[idx] = Some((v, ty));
}

pub(super) fn operand(
    env: &[Option<(Value, MirType)>],
    v: hudhudscript_mir::ValueId,
    func: &MirFunction,
) -> Result<(Value, MirType), BackendError> {
    env.get(v.0 as usize).copied().flatten().ok_or_else(|| {
        reject(func, "operand", &format!("v{} not defined before use", v.0))
    })
}

/// i64 bit pattern → f64 (stack üzerinden taşınabilir bit reinterpretasyonu)
pub(super) fn bitcast_i64_to_f64(builder: &mut FunctionBuilder, v: Value) -> Value {
    let ss = builder.create_sized_stack_slot(
        cranelift::prelude::codegen::ir::StackSlotData::new(
            cranelift::prelude::codegen::ir::StackSlotKind::ExplicitSlot,
            8, 3,
        ),
    );
    builder.ins().stack_store(v, ss, 0);
    builder.ins().stack_load(F64, ss, 0)
}

/// f64 → i64 bit pattern
pub(super) fn bitcast_f64_to_i64(builder: &mut FunctionBuilder, v: Value) -> Value {
    let ss = builder.create_sized_stack_slot(
        cranelift::prelude::codegen::ir::StackSlotData::new(
            cranelift::prelude::codegen::ir::StackSlotKind::ExplicitSlot,
            8, 3,
        ),
    );
    builder.ins().stack_store(v, ss, 0);
    builder.ins().stack_load(I64, ss, 0)
}

/// Operand F64 değilse bitcast ile F64'e çevir (i64 → f64).
pub(super) fn ensure_f64(builder: &mut FunctionBuilder, v: Value, ty: MirType) -> Value {
    if ty == MirType::F64 {
        v
    } else {
        builder.ins().fcvt_from_sint(F64, v)
    }
}

pub(super) fn require_i64(lt: MirType, rt: MirType, what: &str, func: &MirFunction) -> Result<(), BackendError> {
    // Bool = i64 0|1 (uniform şerit) — aritmetikte eşdeğer kabul
    if (lt == MirType::I64 || lt == MirType::Bool) && (rt == MirType::I64 || rt == MirType::Bool) {
        Ok(())
    } else {
        Err(reject(func, what, &format!("needs (i64, i64), got ({lt}, {rt})")))
    }
}

pub(super) fn reject(func: &MirFunction, what: &str, why: &str) -> BackendError {
    BackendError::new("UNSUPPORTED_MIR", format!("{what}: {why}"))
        .in_function(func.name.to_string())
}

pub(super) fn discriminator(i: &MirInst) -> &'static str {
    use MirInst::*;
    match i {
        Add { .. } => "Add",
        Sub { .. } => "Sub",
        Mul { .. } => "Mul",
        Div { .. } => "Div",
        Rem { .. } => "Rem",
        Neg { .. } => "Neg",
        Not { .. } => "Not",
        Cmp { .. } => "Cmp",
        ConstFloat { .. } => "ConstFloat",
        ConstBool { .. } => "ConstBool",
        ConstNull { .. } => "ConstNull",
        Load { .. } => "Load",
        Store { .. } => "Store",
        CallStatic { .. } => "CallStatic",
        CallNative { .. } => "CallNative",
        GcSafepoint => "GcSafepoint",
        Trap { .. } => "Trap",
        Unreachable => "Unreachable",
        ConstInt { .. } => "ConstInt",
        Param { .. } => "Param",
        ConstString { .. } => "ConstString",
        StringConcat { .. } => "StringConcat",
        StringLen { .. } => "StringLen",
        StringEq { .. } => "StringEq",
        IntToString { .. } => "IntToString",
        FloatToString { .. } => "FloatToString",
        LogicalAnd { .. } => "LogicalAnd",
        LogicalOr { .. } => "LogicalOr",
        LogicalNot { .. } => "LogicalNot",
        UnaryNeg { .. } => "UnaryNeg",
        ArrayNew { .. } => "ArrayNew",
        ArrayPush { .. } => "ArrayPush",
        ArrayGet { .. } => "ArrayGet",
        ArraySet { .. } => "ArraySet",
        ArrayLen { .. } => "ArrayLen",
        ArrayPop { .. } => "ArrayPop",
        StringCharAt { .. } => "StringCharAt",
        ObjectNew { .. } => "ObjectNew",
        ObjectSet { .. } => "ObjectSet",
        ObjectGet { .. } => "ObjectGet",
        ObjectHas { .. } => "ObjectHas",
        ObjectLen { .. } => "ObjectLen",
        IntToFloat { .. } => "IntToFloat",
        StringSubstring { .. } => "StringSubstring",
        ArrayJoin { .. } => "ArrayJoin",
    }
}
