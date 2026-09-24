//! MIR aritmetik/karşılaştırma kolları — BigInt promote desenli.
//!
//! Cranelift `emit_promoted_op` denklemi: LLVM'de with-overflow
//! intrinsic'leri (llvm.sadd/ssub/smul.with.overflow.i64) sarmalanmış değeri
//! ve i1 taşma bayrağını tek instruction'da üretir;
//! `need_slow = taşma || operand(lar)da BigInt tag'i (0xB161<<48)` →
//! slow block `hudhud_num_add/sub/mul/div/rem/cmp` helper'ını çağırır.
//! Terfi eden taşma ÇÖZÜLMÜŞTÜR — §18 ov_acc'ye yazılmaz (F13: sahte
//! OVERFLOW exit). Fast bloğu önce yaratılır (H.6 düzeni: sıcak yol önde).

use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{CmpOp, MirInst, ValueId};
use inkwell::values::{BasicValue, FunctionValue, IntValue};
use inkwell::{FloatPredicate, IntPredicate};

use super::{ext_i64_fn, FnCx};

/// BigInt handle tag'i — native-abi HUD_BIGINT_TAG (0xB161<<48) ile aynı.
const TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
const TAG_BIGINT: u64 = 0xB161_0000_0000_0000;

/// llvm.s{add,sub,mul}.with.overflow.i64 → (sarmalanmış i64, i1 taşma).
fn checked_intrinsic<'ctx, 'm>(
    cx: &FnCx<'ctx, 'm>,
    name: &str,
    l: IntValue<'ctx>,
    r: IntValue<'ctx>,
    tag: &str,
) -> (IntValue<'ctx>, IntValue<'ctx>) {
    let f = if let Some(f) = cx.module.get_function(name) {
        f
    } else {
        let st = cx
            .ctx
            .struct_type(&[cx.i64t.into(), cx.ctx.bool_type().into()], false);
        let fty = st.fn_type(&[cx.i64t.into(), cx.i64t.into()], false);
        cx.module.add_function(name, fty, None)
    };
    let call = cx.b.build_call(f, &[l.into(), r.into()], tag);
    let st = call
        .try_as_basic_value()
        .left()
        .expect("with.overflow returns struct")
        .into_struct_value();
    let wrapped = cx
        .b
        .build_extract_value(st, 0, "ow")
        .expect("with.overflow wrapped")
        .into_int_value();
    let ov = cx
        .b
        .build_extract_value(st, 1, "ovf")
        .expect("with.overflow flag")
        .into_int_value();
    (wrapped, ov)
}

/// BigInt adayı mı: üst 16 bit == 0xB161 (i1 döner).
fn is_bigint_candidate<'ctx, 'm>(cx: &FnCx<'ctx, 'm>, v: IntValue<'ctx>) -> IntValue<'ctx> {
    let mask = cx.i64t.const_int(TAG_MASK, false);
    let tag = cx.i64t.const_int(TAG_BIGINT, false);
    let masked = cx.b.build_and(v, mask, "bim");
    cx.b.build_int_compare(IntPredicate::EQ, masked, tag, "bib")
}

/// Slow-path işlem türü: direkt num helper veya num_cmp + 0 karşılaştırma.
enum SlowOp {
    Direct(&'static str),
    Compare(CmpOp),
}

/// Fast/slow/merge üçlüsü: koşul → slow (helper), aksi halde fast; merge'de
/// phi birleştirir. Builder merge'ye konumlanır — kalan instruction'lar ve
/// terminator oraya yazılır (term.rs phi incoming'leri güncel bloktan alır).
fn emit_promoted<'ctx, 'm>(
    cx: &mut FnCx<'ctx, 'm>,
    func: FunctionValue<'ctx>,
    dst: ValueId,
    l: IntValue<'ctx>,
    r: IntValue<'ctx>,
    fast_val: IntValue<'ctx>,
    need_slow: IntValue<'ctx>,
    kind: SlowOp,
) {
    let fast = cx.ctx.append_basic_block(func, "pfast");
    let slow = cx.ctx.append_basic_block(func, "pslow");
    let merge = cx.ctx.append_basic_block(func, "pmerge");
    cx.b.build_conditional_branch(need_slow, slow, fast);
    cx.b.position_at_end(fast);
    cx.b.build_unconditional_branch(merge);
    cx.b.position_at_end(slow);
    let slow_res = match kind {
        SlowOp::Direct(helper) => {
            let f = ext_i64_fn(cx, helper, 2);
            cx.b
                .build_call(f, &[l.into(), r.into()], "big")
                .try_as_basic_value()
                .left()
                .expect("num helper returns i64")
                .into_int_value()
        }
        SlowOp::Compare(op) => {
            let f = ext_i64_fn(cx, "hudhud_num_cmp", 2);
            let c = cx
                .b
                .build_call(f, &[l.into(), r.into()], "ncmp")
                .try_as_basic_value()
                .left()
                .expect("num_cmp returns i64")
                .into_int_value();
            let pred = match op {
                CmpOp::Eq => IntPredicate::EQ,
                CmpOp::Ne => IntPredicate::NE,
                CmpOp::Lt => IntPredicate::SLT,
                CmpOp::Le => IntPredicate::SLE,
                CmpOp::Gt => IntPredicate::SGT,
                CmpOp::Ge => IntPredicate::SGE,
            };
            let b = cx
                .b
                .build_int_compare(pred, c, cx.i64t.const_zero(), "ncmpb");
            cx.b.build_int_z_extend(b, cx.i64t, "ncmp64")
        }
    };
    cx.b.build_unconditional_branch(merge);
    cx.b.position_at_end(merge);
    let phi = cx.b.build_phi(cx.i64t, "pphi");
    phi.add_incoming(&[(&fast_val, fast), (&slow_res, slow)]);
    cx.env.insert(dst.0, phi.as_basic_value());
}

/// Aritmetik/karşılaştırma kollarını çevirir; kol aritmetik değilse
/// Ok(false) döner ve insts.rs'teki ana match işi devralır.
pub(crate) fn try_translate_arith<'ctx, 'm>(
    cx: &mut FnCx<'ctx, 'm>,
    func: FunctionValue<'ctx>,
    inst: &MirInst,
) -> Result<bool, BackendError> {
    use hudhudscript_mir::MirInst as I;
    match inst {
        I::Add { dst, lhs, rhs, .. } | I::Sub { dst, lhs, rhs, .. } => {
            let is_add = matches!(inst, I::Add { .. });
            if cx.is_f64(*lhs) || cx.is_f64(*rhs) {
                let (lf, rf) = (cx.fval(*lhs)?, cx.fval(*rhs)?);
                let v = if is_add {
                    cx.b.build_float_add(lf, rf, "fadd")
                } else {
                    cx.b.build_float_sub(lf, rf, "fsub")
                };
                cx.env.insert(dst.0, v.as_basic_value_enum());
                return Ok(true);
            }
            let (l, r) = (cx.ival(*lhs)?, cx.ival(*rhs)?);
            let (iname, helper) = if is_add {
                ("llvm.sadd.with.overflow.i64", "hudhud_num_add")
            } else {
                ("llvm.ssub.with.overflow.i64", "hudhud_num_sub")
            };
            let (wrapped, ov) = checked_intrinsic(cx, iname, l, r, "adc");
            let need = cx.b.build_or(
                ov,
                cx.b.build_or(
                    is_bigint_candidate(cx, l),
                    is_bigint_candidate(cx, r),
                    "bo2",
                ),
                "bo3",
            );
            emit_promoted(cx, func, *dst, l, r, wrapped, need, SlowOp::Direct(helper));
            Ok(true)
        }
        I::Mul { dst, lhs, rhs, .. } => {
            if cx.is_f64(*lhs) || cx.is_f64(*rhs) {
                let (lf, rf) = (cx.fval(*lhs)?, cx.fval(*rhs)?);
                cx.env.insert(
                    dst.0,
                    cx.b.build_float_mul(lf, rf, "fmul").as_basic_value_enum(),
                );
                return Ok(true);
            }
            let (l, r) = (cx.ival(*lhs)?, cx.ival(*rhs)?);
            let (wrapped, ov) =
                checked_intrinsic(cx, "llvm.smul.with.overflow.i64", l, r, "mulc");
            let need = cx.b.build_or(
                ov,
                cx.b.build_or(
                    is_bigint_candidate(cx, l),
                    is_bigint_candidate(cx, r),
                    "bo2",
                ),
                "bo3",
            );
            emit_promoted(
                cx,
                func,
                *dst,
                l,
                r,
                wrapped,
                need,
                SlowOp::Direct("hudhud_num_mul"),
            );
            Ok(true)
        }
        I::Div { dst, lhs, rhs, .. } | I::Rem { dst, lhs, rhs, .. } => {
            let is_div = matches!(inst, I::Div { .. });
            if cx.is_f64(*lhs) || cx.is_f64(*rhs) {
                let (lf, rf) = (cx.fval(*lhs)?, cx.fval(*rhs)?);
                let v = if is_div {
                    cx.b
                        .build_float_div(lf, rf, "fdiv")
                        .as_basic_value_enum()
                } else {
                    cx.b
                        .build_float_rem(lf, rf, "frem")
                        .as_basic_value_enum()
                };
                cx.env.insert(dst.0, v);
                return Ok(true);
            }
            let (l, r) = (cx.ival(*lhs)?, cx.ival(*rhs)?);
            let i64t = cx.i64t;
            // §18 bölme-sıfır bayrağı dallanmasız birikir; SIGFPE select'le
            // engellenir. F13/VM paritesi: MIN/-1 promote edilir (num_div →
            // BigInt 2^63) — OVERFLOW bayrağına YAZILMAZ, slow-path çözer.
            let is_zero = cx
                .b
                .build_int_compare(IntPredicate::EQ, r, i64t.const_zero(), "isdz");
            cx.or_flag(is_zero, cx.dz_acc);
            let neg1 = i64t.const_int((-1i64) as u64, true);
            let minv = i64t.const_int(i64::MIN as u64, true);
            let is_min = cx.b.build_int_compare(IntPredicate::EQ, l, minv, "ismin");
            let is_neg1 = cx.b.build_int_compare(IntPredicate::EQ, r, neg1, "isneg1");
            let min_neg1 = cx.b.build_and(is_min, is_neg1, "minneg1");
            let one = i64t.const_int(1, false);
            let sel1 = cx
                .b
                .build_select(is_zero, one, r, "safe1")
                .into_int_value();
            let safe_r = cx
                .b
                .build_select(min_neg1, one, sel1, "safer")
                .into_int_value();
            let fast_val = if is_div {
                cx.b.build_int_signed_div(l, safe_r, "sdiv")
            } else {
                cx.b.build_int_signed_rem(l, safe_r, "srem")
            };
            let need = cx.b.build_or(
                min_neg1,
                cx.b.build_or(
                    is_bigint_candidate(cx, l),
                    is_bigint_candidate(cx, r),
                    "bd",
                ),
                "bd2",
            );
            let helper = if is_div {
                "hudhud_num_div"
            } else {
                "hudhud_num_rem"
            };
            emit_promoted(cx, func, *dst, l, r, fast_val, need, SlowOp::Direct(helper));
            Ok(true)
        }
        I::Cmp { dst, op, lhs, rhs, .. } => {
            if cx.is_f64(*lhs) || cx.is_f64(*rhs) {
                let (lf, rf) = (cx.fval(*lhs)?, cx.fval(*rhs)?);
                let fpred = match op {
                    CmpOp::Eq => FloatPredicate::OEQ,
                    CmpOp::Ne => FloatPredicate::ONE,
                    CmpOp::Lt => FloatPredicate::OLT,
                    CmpOp::Le => FloatPredicate::OLE,
                    CmpOp::Gt => FloatPredicate::OGT,
                    CmpOp::Ge => FloatPredicate::OGE,
                };
                let c = cx.b.build_float_compare(fpred, lf, rf, "fcmp");
                cx.env.insert(
                    dst.0,
                    cx.b
                        .build_int_z_extend(c, cx.i64t, "cmp64")
                        .as_basic_value_enum(),
                );
                return Ok(true);
            }
            let (l, r) = (cx.ival(*lhs)?, cx.ival(*rhs)?);
            let pred = match op {
                CmpOp::Eq => IntPredicate::EQ,
                CmpOp::Ne => IntPredicate::NE,
                CmpOp::Lt => IntPredicate::SLT,
                CmpOp::Le => IntPredicate::SLE,
                CmpOp::Gt => IntPredicate::SGT,
                CmpOp::Ge => IntPredicate::SGE,
            };
            let fast_val = cx.b.build_int_z_extend(
                cx.b.build_int_compare(pred, l, r, "cmp"),
                cx.i64t,
                "cmp64",
            );
            let need = cx.b.build_or(
                is_bigint_candidate(cx, l),
                is_bigint_candidate(cx, r),
                "bc",
            );
            emit_promoted(cx, func, *dst, l, r, fast_val, need, SlowOp::Compare(*op));
            Ok(true)
        }
        _ => Ok(false),
    }
}
