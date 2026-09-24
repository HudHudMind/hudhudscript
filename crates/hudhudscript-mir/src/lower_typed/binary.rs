//! Binary operation lowering for typed HIR → MIR.

use hudhudscript_types::{HirBinOp, HirExpr, HirFunction};

use crate::builder::BinOp;
use crate::mir::{CmpOp, MirType, ValueId};
use crate::LowerError;

use super::cx::FnCx;
use super::expr::lower_expr;

pub(crate) fn lower_binary(
    hir: &HirFunction,
    cx: &mut FnCx,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
) -> Result<ValueId, LowerError> {
    // F11: && / || KISA-DEVRE — sağ operand yalnız gerekirse değerlenir.
    // (false && effect() → effect ÇAĞRILMAZ; VM oracle ile aynı.)
    if matches!(op, HirBinOp::And | HirBinOp::Or) {
        return lower_short_circuit(hir, cx, op, lhs, rhs);
    }
    let l = lower_expr(hir, cx, lhs)?;
    let r = lower_expr(hir, cx, rhs)?;
    // operandlar ternary olabilir → güncel bloğu taze oku
    let block = cx.current_block;
    // Bool operandlar i64 0|1 olarak yaşar — aritmetikte I64 etiketi
    if cx.ty_of(l) == Some(MirType::Bool) {
        cx.set_ty(l, MirType::I64);
    }
    if cx.ty_of(r) == Some(MirType::Bool) {
        cx.set_ty(r, MirType::I64);
    }
    let (lt, rt) = (cx.ty_of(l), cx.ty_of(r));

    // String terfisi: string + herhangi → StringConcat (fakat null == string pointer karşılaştırması Cmp kalmalı)
    let is_str = |t: Option<MirType>| t == Some(MirType::Ref(crate::mir::RefKind::String));
    let is_null = |t: Option<MirType>| t == Some(MirType::Generic);
    if (is_str(lt) || is_str(rt)) && !(matches!(op, HirBinOp::Eq | HirBinOp::Ne) && (is_null(lt) || is_null(rt))) {
        let ls = if is_str(lt) {
            l
        } else if lt == Some(MirType::F64) {
            cx.builder.float_to_string(block, l)
        } else {
            cx.builder.int_to_string(block, l)
        };
        let rs = if is_str(rt) {
            r
        } else if rt == Some(MirType::F64) {
            cx.builder.float_to_string(block, r)
        } else {
            cx.builder.int_to_string(block, r)
        };
        match op {
            HirBinOp::Add => {
                let v = cx.builder.string_concat(block, ls, rs);
                cx.set_ty(v, MirType::Ref(crate::mir::RefKind::String));
                return Ok(v);
            }
            HirBinOp::Eq => {
                let v = cx.builder.string_eq(block, ls, rs);
                cx.set_ty(v, MirType::Bool);
                return Ok(v);
            }
            HirBinOp::Ne => {
                let eq = cx.builder.string_eq(block, ls, rs);
                cx.set_ty(eq, MirType::Bool);
                let one = cx.builder.const_i64(block, 1);
                let v = cx.builder.bin(block, crate::builder::BinOp::Sub, MirType::I64, one, eq);
                cx.set_ty(v, MirType::Bool);
                return Ok(v);
            }
            HirBinOp::Lt | HirBinOp::Le | HirBinOp::Gt | HirBinOp::Ge => {
                let cmp = cx.builder.call_native(
                    block,
                    MirType::I64,
                    crate::mir::RuntimeHelperId::StringCmp,
                    vec![ls, rs],
                );
                let zero = cx.builder.const_i64(block, 0);
                let cmp_op = match op {
                    HirBinOp::Lt => crate::mir::CmpOp::Lt,
                    HirBinOp::Le => crate::mir::CmpOp::Le,
                    HirBinOp::Gt => crate::mir::CmpOp::Gt,
                    HirBinOp::Ge => crate::mir::CmpOp::Ge,
                    _ => unreachable!(),
                };
                let v = cx.builder.cmp(block, cmp_op, MirType::I64, cmp, zero);
                cx.set_ty(v, MirType::Bool);
                return Ok(v);
            }
            _ => return Err(cx.err("unsupported string binary operation in the typed lane")),
        }
    }


    // Tip terfisi: herhangi biri F64 ise sonuç F64
    let is_f64 = lt == Some(MirType::F64) || rt == Some(MirType::F64);
    let arith_ty = if is_f64 { MirType::F64 } else { MirType::I64 };

    let is_equality = matches!(op, HirBinOp::Eq | HirBinOp::Ne);
    let valid = match (lt, rt) {
        (Some(MirType::I64), Some(MirType::I64)) => true,
        (Some(MirType::F64), Some(MirType::F64)) => true,
        (Some(MirType::I64), Some(MirType::F64)) => true,
        (Some(MirType::F64), Some(MirType::I64)) => true,
        // Bool operands for logical/comparison results
        (Some(MirType::Bool), Some(MirType::Bool)) => true,
        (Some(MirType::I64), Some(MirType::Bool)) => true,
        (Some(MirType::Bool), Some(MirType::I64)) => true,
        // Equality comparison with null (generic) or reference pointers
        _ if is_equality => match (lt, rt) {
            (Some(MirType::Generic), _) | (_, Some(MirType::Generic)) => true,
            (Some(MirType::Ref(_)), _) | (_, Some(MirType::Ref(_))) => true,
            _ => false,
        },
        _ => false,
    };
    if !valid {
        return Err(cx.err(&format!(
            "binary {op} needs numeric operands, got ({}, {})",
            lt.map(|t| t.to_string()).unwrap_or_else(|| "?".into()),
            rt.map(|t| t.to_string()).unwrap_or_else(|| "?".into()),
        )));
    }

    let out = match op {
        HirBinOp::Add => cx.builder.bin(block, BinOp::Add, arith_ty, l, r),
        HirBinOp::Sub => cx.builder.bin(block, BinOp::Sub, arith_ty, l, r),
        HirBinOp::Mul => cx.builder.bin(block, BinOp::Mul, arith_ty, l, r),
        HirBinOp::Div => cx.builder.bin(block, BinOp::Div, arith_ty, l, r),
        HirBinOp::Rem => cx.builder.bin(block, BinOp::Rem, arith_ty, l, r),
        HirBinOp::Eq => cx.builder.cmp(block, CmpOp::Eq, arith_ty, l, r),
        HirBinOp::Ne => cx.builder.cmp(block, CmpOp::Ne, arith_ty, l, r),
        HirBinOp::Lt => cx.builder.cmp(block, CmpOp::Lt, arith_ty, l, r),
        HirBinOp::Le => cx.builder.cmp(block, CmpOp::Le, arith_ty, l, r),
        HirBinOp::Gt => cx.builder.cmp(block, CmpOp::Gt, arith_ty, l, r),
        HirBinOp::Ge => cx.builder.cmp(block, CmpOp::Ge, arith_ty, l, r),
        HirBinOp::And | HirBinOp::Or => return Err(cx.err("logical operators (bool lane)")),
    };
    let out_ty = match op {
        HirBinOp::Eq | HirBinOp::Ne | HirBinOp::Lt
        | HirBinOp::Le | HirBinOp::Gt | HirBinOp::Ge => MirType::Bool,
        _ => arith_ty,
    };
    cx.set_ty(out, out_ty);
    Ok(out)
}


/// F11 kısa-devre lowering:
///   a && b  →  a falsy ise sonuç a; değilse b (JS/Lua seçili-değer semantiği)
///   a || b  →  a truthy ise sonuç a; değilse b
/// Yan etki düzgünlüğü: sağ operand YALNIZ ilgili dalda değerlenir.
fn lower_short_circuit(
    hir: &HirFunction,
    cx: &mut FnCx,
    op: HirBinOp,
    lhs: &HirExpr,
    rhs: &HirExpr,
) -> Result<ValueId, LowerError> {
    let l = lower_expr(hir, cx, lhs)?;
    let l_ty = cx.ty_of(l).unwrap_or(MirType::I64);

    // truthy testi: tip biliniyorsa doğrudan karşılaştır; bool zaten 0|1
    let entry = cx.current_block;
    let zero = match l_ty {
        MirType::F64 => cx.builder.const_f64(entry, 0.0),
        _ => cx.builder.const_i64(entry, 0),
    };
    let is_truthy = cx.builder.cmp(entry, CmpOp::Ne, l_ty, l, zero);
    cx.set_ty(is_truthy, MirType::Bool);

    let rhs_blk = cx.builder.create_block();
    let merge_blk = cx.builder.create_block_with_params(vec![l_ty]);
    let phi = merge_blk.1[0];
    cx.set_ty(phi, l_ty);

    match op {
        HirBinOp::Or => {
            // a truthy → sonuç a (sağ değerlenMEZ); değilse sağa geç
            cx.builder.cond_branch_with_args(
                entry, is_truthy, merge_blk.0, vec![l], rhs_blk, vec![],
            );
        }
        _ => {
            // a falsy → sonuç a; truthy ise sağa geç
            cx.builder.cond_branch_with_args(
                entry, is_truthy, rhs_blk, vec![], merge_blk.0, vec![l],
            );
        }
    }

    // Sağ operand yalnız burada değerlenir
    cx.current_block = rhs_blk;
    let r = lower_expr(hir, cx, rhs)?;
    let r_ty = cx.ty_of(r).unwrap_or(l_ty);
    let rhs_final = cx.current_block;
    // Tipler farklıysa (int vs float) — bool lane: güvenli ortak I64
    let (r_out, phi_ty) = if r_ty != l_ty {
        // nadir karışım: her iki tarafı I64 etiketiyle birleştir (mevcut lane tutarlılığı)
        (r, MirType::I64)
    } else {
        (r, l_ty)
    };
    cx.builder.branch_with_args(rhs_final, merge_blk.0, vec![r_out]);
    let _ = phi_ty;

    cx.current_block = merge_blk.0;
    Ok(phi)
}
