//! MIR aritmetik/karşılaştırma kolları — BigInt promote desenli.
//!
//! Cranelift `emit_promoted_op` denklemi: fast path sarmalanmış ham i64
//! aritmetiği (u64 şeridinde — C'de taşma tanımlıdır), `need_slow = taşma ||
//! operand(lar)da BigInt tag'i (0xB161<<48)`; slow path
//! `hudhud_num_add/sub/mul/div/rem/cmp` helper'ı doğru BigInt değerini üretir.
//! Terfi eden taşma ÇÖZÜLMÜŞTÜR — §18 ov_acc'ye yazılmaz (F13: sahte
//! OVERFLOW exit). Fast bloğu önce yaratılır (H.6 düzeni: sıcak yol önde).

use gccjit::{BinaryOp, Block, ComparisonOp, RValue, ToRValue, Typeable};
use hudhudscript_codegen::backend::BackendError;
use hudhudscript_mir::{CmpOp, MirInst, MirType, ValueId};

use super::FnCx;

/// BigInt handle tag'i — native-abi HUD_BIGINT_TAG (0xB161<<48) ile aynı.
const TAG_MASK: i64 = 0xFFFF_0000_0000_0000u64 as i64;
const TAG_BIGINT: i64 = 0xB161_0000_0000_0000u64 as i64;

/// bool karşılaştırma sonucunu ll (0/1) şeridine çevirir.
fn bll<'ctx>(cx: &FnCx<'ctx, '_>, b: RValue<'ctx>) -> RValue<'ctx> {
    cx.abi.gcx.new_cast(None, b, cx.abi.ll)
}

fn eq_ll<'ctx>(cx: &FnCx<'ctx, '_>, a: RValue<'ctx>, b: RValue<'ctx>) -> RValue<'ctx> {
    bll(cx, cx.abi.gcx.new_comparison(None, ComparisonOp::Equals, a, b))
}

fn ne_ll<'ctx>(cx: &FnCx<'ctx, '_>, a: RValue<'ctx>, b: RValue<'ctx>) -> RValue<'ctx> {
    bll(cx, cx.abi.gcx.new_comparison(None, ComparisonOp::NotEquals, a, b))
}

fn and_ll<'ctx>(cx: &FnCx<'ctx, '_>, a: RValue<'ctx>, b: RValue<'ctx>) -> RValue<'ctx> {
    cx.abi
        .gcx
        .new_binary_op(None, BinaryOp::BitwiseAnd, cx.abi.ll, a, b)
}

fn or_ll<'ctx>(cx: &FnCx<'ctx, '_>, a: RValue<'ctx>, b: RValue<'ctx>) -> RValue<'ctx> {
    cx.abi
        .gcx
        .new_binary_op(None, BinaryOp::BitwiseOr, cx.abi.ll, a, b)
}

/// ll 0/1 bayrağını tersine çevirir.
fn not_ll<'ctx>(cx: &FnCx<'ctx, '_>, a: RValue<'ctx>) -> RValue<'ctx> {
    cx.abi
        .gcx
        .new_binary_op(None, BinaryOp::BitwiseXor, cx.abi.ll, a, cx.ll(1))
}

/// BigInt adayı: üst 16 bit == 0xB161 (cranelift ushr_imm==0xB161 denklemi).
fn is_bigint_candidate<'ctx>(cx: &FnCx<'ctx, '_>, v: RValue<'ctx>) -> RValue<'ctx> {
    let g = cx.abi.gcx;
    let masked = g.new_binary_op(None, BinaryOp::BitwiseAnd, cx.abi.ll, v, cx.ll(TAG_MASK));
    eq_ll(cx, masked, cx.ll(TAG_BIGINT))
}

/// u64 şeridinde sarmalanmış (C'de tanımlı) işlem → ll'ye geri.
fn wrapped_u<'ctx>(
    cx: &FnCx<'ctx, '_>,
    op: BinaryOp,
    l: RValue<'ctx>,
    r: RValue<'ctx>,
) -> RValue<'ctx> {
    let g = cx.abi.gcx;
    let ull = <u64>::get_type(g);
    let lu = g.new_cast(None, l, ull);
    let ru = g.new_cast(None, r, ull);
    let wu = g.new_binary_op(None, op, ull, lu, ru);
    g.new_cast(None, wu, cx.abi.ll)
}

/// Fast/slow bölme: koşul → slow (helper ifadesi) / fast (sarmalanmış);
/// merge'de lokal birleşir (gccjit'te phi yoktur — C-dili denkimi).
/// *gb promote sonrası merge bloğuna döner; kalan instruction'lar ve
/// terminator oraya yazılır (çağıran gb'yi geri almalıdır).
fn emit_promoted<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: &mut Block<'ctx>,
    dst: ValueId,
    fast_val: RValue<'ctx>,
    slow_val: RValue<'ctx>,
    need_slow_ll: RValue<'ctx>,
) {
    let g = cx.abi.gcx;
    let res = cx.func.new_local(None, cx.abi.ll, format!("v{}_p", dst.0));
    let fast = cx.func.new_block(format!("pf{}", dst.0));
    let slow = cx.func.new_block(format!("ps{}", dst.0));
    let merge = cx.func.new_block(format!("pm{}", dst.0));
    let cond = g.new_comparison(None, ComparisonOp::NotEquals, need_slow_ll, cx.ll(0));
    gb.end_with_conditional(None, cond, slow, fast);
    fast.add_assignment(None, res, fast_val);
    fast.end_with_jump(None, merge);
    slow.add_assignment(None, res, slow_val);
    slow.end_with_jump(None, merge);
    *gb = merge;
    cx.store(*gb, dst, res.to_rvalue(), false);
}

fn emit_addsub<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: &mut Block<'ctx>,
    dst: ValueId,
    l: RValue<'ctx>,
    r: RValue<'ctx>,
    is_add: bool,
) {
    let g = cx.abi.gcx;
    let uop = if is_add { BinaryOp::Plus } else { BinaryOp::Minus };
    let wrapped = wrapped_u(cx, uop, l, r);
    // taşma işaret analizi — sarmalanmış değer üzerinden (tanımlı işlem)
    let ov = if is_add {
        let ax = g.new_binary_op(None, BinaryOp::BitwiseXor, cx.abi.ll, l, wrapped);
        let bx = g.new_binary_op(None, BinaryOp::BitwiseXor, cx.abi.ll, r, wrapped);
        let bits = and_ll(cx, ax, bx);
        bll(cx, g.new_comparison(None, ComparisonOp::LessThan, bits, cx.ll(0)))
    } else {
        let ab = g.new_binary_op(None, BinaryOp::BitwiseXor, cx.abi.ll, l, r);
        let ad = g.new_binary_op(None, BinaryOp::BitwiseXor, cx.abi.ll, l, wrapped);
        let bits = and_ll(cx, ab, ad);
        bll(cx, g.new_comparison(None, ComparisonOp::LessThan, bits, cx.ll(0)))
    };
    let need = or_ll(
        cx,
        ov,
        or_ll(cx, is_bigint_candidate(cx, l), is_bigint_candidate(cx, r)),
    );
    let helper = if is_add { cx.ext.num_add } else { cx.ext.num_sub };
    let slow_val = g.new_call(None, helper, &[l, r]);
    emit_promoted(cx, gb, dst, wrapped, slow_val, need);
}

fn emit_mul<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: &mut Block<'ctx>,
    dst: ValueId,
    l: RValue<'ctx>,
    r: RValue<'ctx>,
) {
    let g = cx.abi.gcx;
    let wrapped = wrapped_u(cx, BinaryOp::Mult, l, r);
    // taşma: tuzaksız böl-ve-kontrol (bölen asla 0 veya -1×MIN tuzağı değil):
    //   safe_l = (l==0 || (l==-1 && wrapped==MIN)) ? 1 : l
    //   ov = l!=0 && (wrapped/safe_l != r) || (l==-1&&r==MIN) || (l==MIN&&r==-1)
    let zero = cx.ll(0);
    let one = cx.ll(1);
    let m1 = cx.ll(-1);
    let minv = cx.ll(i64::MIN as i64);
    let l_eq_0 = eq_ll(cx, l, zero);
    let l_eq_m1 = eq_ll(cx, l, m1);
    let l_eq_min = eq_ll(cx, l, minv);
    let r_eq_m1 = eq_ll(cx, r, m1);
    let r_eq_min = eq_ll(cx, r, minv);
    let w_eq_min = eq_ll(cx, wrapped, minv);
    let guard = or_ll(cx, l_eq_0, and_ll(cx, l_eq_m1, w_eq_min));
    let safe_l = g.new_call(None, cx.ext.select_i64, &[guard, one, l]);
    let quot = g.new_binary_op(None, BinaryOp::Divide, cx.abi.ll, wrapped, safe_l);
    let chk = ne_ll(cx, quot, r);
    let pair_a = and_ll(cx, l_eq_m1, r_eq_min);
    let pair_b = and_ll(cx, l_eq_min, r_eq_m1);
    let ov = or_ll(
        cx,
        and_ll(cx, not_ll(cx, l_eq_0), chk),
        or_ll(cx, pair_a, pair_b),
    );
    let need = or_ll(
        cx,
        ov,
        or_ll(cx, is_bigint_candidate(cx, l), is_bigint_candidate(cx, r)),
    );
    let slow_val = g.new_call(None, cx.ext.num_mul, &[l, r]);
    emit_promoted(cx, gb, dst, wrapped, slow_val, need);
}

fn emit_divrem<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: &mut Block<'ctx>,
    dst: ValueId,
    l: RValue<'ctx>,
    r: RValue<'ctx>,
    is_div: bool,
) {
    let g = cx.abi.gcx;
    // §18 bölme-sıfır bayrağı dallanmasız birikir (exit'te seçilir)
    let zero = cx.ll(0);
    let one = cx.ll(1);
    let m1 = cx.ll(-1);
    let minv = cx.ll(i64::MIN as i64);
    let is_zero = eq_ll(cx, r, zero);
    cx.div_zero_flags_or(*gb, is_zero);
    // F13/VM paritesi: MIN/-1 promote edilir (num_div → BigInt 2^63) —
    // §18 OVERFLOW bayrağına YAZILMAZ, slow-path ile çözülür.
    let min_neg1 = and_ll(cx, eq_ll(cx, l, minv), eq_ll(cx, r, m1));
    // SIGFPE koruması: select helper'larıyla bölen güvenli (cranelift denkimi)
    let sel1 = g.new_call(None, cx.ext.select_i64, &[is_zero, one, r]);
    let safe_r = g.new_call(None, cx.ext.select_i64, &[min_neg1, one, sel1]);
    let op = if is_div { BinaryOp::Divide } else { BinaryOp::Modulo };
    let fast_val = g.new_binary_op(None, op, cx.abi.ll, l, safe_r);
    let need = or_ll(
        cx,
        min_neg1,
        or_ll(cx, is_bigint_candidate(cx, l), is_bigint_candidate(cx, r)),
    );
    let helper = if is_div { cx.ext.num_div } else { cx.ext.num_rem };
    let slow_val = g.new_call(None, helper, &[l, r]);
    emit_promoted(cx, gb, dst, fast_val, slow_val, need);
}

/// Mir karşılaştırma → gccjit karşılaştırma (ComparisonOp Copy değildir —
/// iki kullanım için yeniden üretir).
fn gcc_cmp_op(op: CmpOp) -> ComparisonOp {
    match op {
        CmpOp::Eq => ComparisonOp::Equals,
        CmpOp::Ne => ComparisonOp::NotEquals,
        CmpOp::Lt => ComparisonOp::LessThan,
        CmpOp::Le => ComparisonOp::LessThanEquals,
        CmpOp::Gt => ComparisonOp::GreaterThan,
        CmpOp::Ge => ComparisonOp::GreaterThanEquals,
    }
}

fn emit_cmp<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: &mut Block<'ctx>,
    dst: ValueId,
    op: CmpOp,
    l: RValue<'ctx>,
    r: RValue<'ctx>,
) {
    let g = cx.abi.gcx;
    let fast_val = bll(cx, g.new_comparison(None, gcc_cmp_op(op), l, r));
    // slow: num_cmp işaret döner → 0 ile aynı koşula göre karşılaştır
    let c = g.new_call(None, cx.ext.num_cmp, &[l, r]);
    let slow_val = bll(cx, g.new_comparison(None, gcc_cmp_op(op), c, cx.ll(0)));
    let need = or_ll(cx, is_bigint_candidate(cx, l), is_bigint_candidate(cx, r));
    emit_promoted(cx, gb, dst, fast_val, slow_val, need);
}

/// Aritmetik/karşılaştırma kollarını çevirir; kol aritmetik değilse
/// Ok(false) döner ve insts.rs'teki ana match işi devralır.
pub(crate) fn try_translate_arith<'ctx, 'a>(
    cx: &mut FnCx<'ctx, 'a>,
    gb: &mut Block<'ctx>,
    inst: &MirInst,
) -> Result<bool, BackendError> {
    use hudhudscript_mir::MirInst as I;
    match inst {
        I::Add { dst, lhs, rhs, ty } | I::Sub { dst, lhs, rhs, ty } => {
            let is_add = matches!(inst, I::Add { .. });
            let l = cx.val(*lhs)?;
            let r = cx.val(*rhs)?;
            let lf = cx.float_vals.contains(&lhs.0);
            let rf = cx.float_vals.contains(&rhs.0);
            if *ty == MirType::F64 || lf || rf {
                let lt2 = if lf { MirType::F64 } else { MirType::I64 };
                let rt2 = if rf { MirType::F64 } else { MirType::I64 };
                let op = if is_add { BinaryOp::Plus } else { BinaryOp::Minus };
                let v = cx
                    .abi
                    .gcx
                    .new_binary_op(None, op, cx.abi.f64t, cx.as_f64(l, lt2), cx.as_f64(r, rt2));
                cx.store(*gb, *dst, v, true);
            } else {
                emit_addsub(cx, gb, *dst, l, r, is_add);
            }
            Ok(true)
        }
        I::Mul { dst, lhs, rhs, ty } => {
            let l = cx.val(*lhs)?;
            let r = cx.val(*rhs)?;
            if *ty == MirType::F64
                || cx.float_vals.contains(&lhs.0)
                || cx.float_vals.contains(&rhs.0)
            {
                let v = cx.abi.gcx.new_binary_op(
                    None,
                    BinaryOp::Mult,
                    cx.abi.f64t,
                    cx.as_f64(l, MirType::I64),
                    cx.as_f64(r, MirType::I64),
                );
                cx.store(*gb, *dst, v, true);
            } else {
                emit_mul(cx, gb, *dst, l, r);
            }
            Ok(true)
        }
        I::Div { dst, lhs, rhs, ty } | I::Rem { dst, lhs, rhs, ty } => {
            let is_div = matches!(inst, I::Div { .. });
            let l = cx.val(*lhs)?;
            let r = cx.val(*rhs)?;
            let lf64 = cx.float_vals.contains(&lhs.0);
            let rf64 = cx.float_vals.contains(&rhs.0);
            if lf64 || rf64 || *ty == MirType::F64 {
                let lt = if lf64 { MirType::F64 } else { MirType::I64 };
                let rt = if rf64 { MirType::F64 } else { MirType::I64 };
                let g = cx.abi.gcx;
                if is_div {
                    let v = g.new_binary_op(
                        None,
                        BinaryOp::Divide,
                        cx.abi.f64t,
                        cx.as_f64(l, lt),
                        cx.as_f64(r, rt),
                    );
                    cx.store(*gb, *dst, v, true);
                } else {
                    let v = g.new_call(None, cx.ext.fmod, &[cx.as_f64(l, lt), cx.as_f64(r, rt)]);
                    cx.store(*gb, *dst, v, true);
                }
            } else {
                emit_divrem(cx, gb, *dst, l, r, is_div);
            }
            Ok(true)
        }
        I::Cmp { dst, op, lhs, rhs, .. } => {
            let l = cx.val(*lhs)?;
            let r = cx.val(*rhs)?;
            if cx.float_vals.contains(&lhs.0) || cx.float_vals.contains(&rhs.0) {
                let lt = if cx.float_vals.contains(&lhs.0) { MirType::F64 } else { MirType::I64 };
                let rt = if cx.float_vals.contains(&rhs.0) { MirType::F64 } else { MirType::I64 };
                let fop = match op {
                    CmpOp::Eq => ComparisonOp::Equals,
                    CmpOp::Ne => ComparisonOp::NotEquals,
                    CmpOp::Lt => ComparisonOp::LessThan,
                    CmpOp::Le => ComparisonOp::LessThanEquals,
                    CmpOp::Gt => ComparisonOp::GreaterThan,
                    CmpOp::Ge => ComparisonOp::GreaterThanEquals,
                };
                let b = cx.abi.gcx.new_comparison(
                    None,
                    fop,
                    cx.as_f64(l, lt),
                    cx.as_f64(r, rt),
                );
                cx.store(*gb, *dst, bll(cx, b), false);
            } else {
                emit_cmp(cx, gb, *dst, *op, l, r);
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
