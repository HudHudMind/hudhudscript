//! Phi and control flow type promotion helpers for typed lowering.

use std::collections::HashMap;
use hudhudscript_types::{HirExpr, HirStmt};
use crate::mir::{BlockId, MirType, ValueId};
use super::cx::FnCx;

/// Gövdede yeniden bağlanan (Assign/Let ile mevcut değişkene yeni değer
/// atayan) değişken adlarını topla — While phi'si için.
pub(super) fn collect_reassigned(body: &[HirStmt], outer: &HashMap<String, ValueId>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    fn scan(stmts: &[HirStmt], outer: &HashMap<String, ValueId>, out: &mut Vec<String>) {
        for s in stmts {
            match s {
                HirStmt::Assign { name, .. } | HirStmt::Let { name, .. } => {
                    if outer.contains_key(name) && !out.contains(name) {
                        out.push(name.clone());
                    }
                }
                HirStmt::If { then_branch, else_branch, .. } => {
                    scan(then_branch, outer, out);
                    scan(else_branch, outer, out);
                }
                HirStmt::While { body, .. } => {
                    scan(body, outer, out);
                }
                HirStmt::Try { try_body, catch_body, finally_body, .. } => {
                    scan(try_body, outer, out);
                    scan(catch_body, outer, out);
                    scan(finally_body, outer, out);
                }
                _ => {}
            }
        }
    }
    scan(body, outer, &mut out);
    out
}

pub(super) fn promote_to_phi(
    cx: &mut FnCx,
    block: BlockId,
    val: ValueId,
    target_ty: MirType,
) -> ValueId {
    if target_ty == MirType::F64 && cx.ty_of(val) != Some(MirType::F64) {
        let f = cx.builder.int_to_float(block, val);
        cx.set_ty(f, MirType::F64);
        f
    } else {
        val
    }
}

pub(super) fn body_assigns_float(name: &str, stmts: &[HirStmt], cx: &FnCx) -> bool {
    for s in stmts {
        match s {
            HirStmt::Assign { name: n, value } | HirStmt::Let { name: n, value, .. } if n == name => {
                if expr_produces_float(value, cx) {
                    return true;
                }
            }
            HirStmt::If { then_branch, else_branch, .. } => {
                if body_assigns_float(name, then_branch, cx) || body_assigns_float(name, else_branch, cx) {
                    return true;
                }
            }
            HirStmt::While { body, .. } if body_assigns_float(name, body, cx) => return true,
            HirStmt::Try { try_body, catch_body, finally_body, .. } => {
                if body_assigns_float(name, try_body, cx)
                    || body_assigns_float(name, catch_body, cx)
                    || body_assigns_float(name, finally_body, cx)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

pub(super) fn expr_produces_float(e: &HirExpr, cx: &FnCx) -> bool {
    match e {
        HirExpr::FloatLit(..) => true,
        HirExpr::Local { name, .. } => {
            cx.bindings.get(name).and_then(|v| cx.ty_of(*v)) == Some(MirType::F64)
        }
        HirExpr::Call { callee, .. } => {
            cx.module_functions.get(callee).map(|(_, _, ret)| *ret) == Some(MirType::F64)
        }
        HirExpr::ArrayIndex { array, .. } => {
            if let HirExpr::Local { name, .. } = array.as_ref() {
                cx.array_elem_tys.get(name) == Some(&MirType::F64)
            } else {
                false
            }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            expr_produces_float(lhs, cx) || expr_produces_float(rhs, cx)
        }
        _ => false,
    }
}

pub(super) fn track_assigned_array_types(cx: &mut FnCx, name: &str, value: &HirExpr, v: ValueId) {
    match value {
        HirExpr::ArrayLit { elements, .. } => {
            if let Some(HirExpr::ArrayLit { elements: inner, .. }) = elements.first() {
                cx.array_elem_tys.insert(name.to_string(), MirType::Ref(crate::mir::RefKind::Array));
                if inner.iter().any(|e| matches!(e, HirExpr::FloatLit(..))) {
                    cx.array_inner_elem_tys.insert(name.to_string(), MirType::F64);
                }
            } else if elements.iter().any(|e| matches!(e, HirExpr::FloatLit(..))) {
                cx.array_elem_tys.insert(name.to_string(), MirType::F64);
            }
        }
        HirExpr::ArrayIndex { array: inner_arr, .. } => {
            if let HirExpr::Local { name: inner_name, .. } = inner_arr.as_ref() {
                if cx.array_elem_tys.get(inner_name) == Some(&MirType::Ref(crate::mir::RefKind::Array)) {
                    cx.set_ty(v, MirType::Ref(crate::mir::RefKind::Array));
                    let inner_elem = cx.array_inner_elem_tys.get(inner_name).copied().unwrap_or(MirType::F64);
                    cx.array_elem_tys.insert(name.to_string(), inner_elem);
                }
            }
        }
        HirExpr::ArrayMethod { method, .. } if method == "split" => {
            cx.array_elem_tys.insert(name.to_string(), MirType::Ref(crate::mir::RefKind::String));
        }
        _ => {}
    }
}
