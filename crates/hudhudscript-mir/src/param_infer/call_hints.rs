//! Call site argument inference and collection across a module.

use std::collections::{HashMap, HashSet};
use hudhudscript_types::{HirBinOp, HirExpr, HirModule, HirStmt};
use crate::mir::{MirType, RefKind};

pub(super) type CallHints = HashMap<String, HashMap<usize, HashSet<MirType>>>;

pub(super) fn collect_call_hints(
    module: &HirModule,
    globals: &HashMap<String, (u32, MirType)>,
    return_tys: &HashMap<String, MirType>,
    known_params: &HashMap<String, HashMap<String, MirType>>,
) -> CallHints {
    let mut hints: CallHints = HashMap::new();
    for (name, f) in &module.functions {
        let mut locals = HashMap::new();
        if let Some(params) = known_params.get(name) {
            locals.extend(params.clone());
        }
        collect_calls_in_stmts(&f.body, &mut locals, globals, return_tys, &mut hints);
    }
    hints
}

fn collect_calls_in_stmts(
    stmts: &[HirStmt],
    locals: &mut HashMap<String, MirType>,
    globals: &HashMap<String, (u32, MirType)>,
    return_tys: &HashMap<String, MirType>,
    hints: &mut CallHints,
) {
    for s in stmts {
        match s {
            HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } => {
                if let Some(t) = guess_expr_ty(value, locals, globals, return_tys) {
                    locals.insert(name.clone(), t);
                }
                walk_expr_for_calls(value, locals, globals, return_tys, hints);
            }
            HirStmt::Expr(e) | HirStmt::Return(Some(e)) => {
                walk_expr_for_calls(e, locals, globals, return_tys, hints);
            }
            HirStmt::If { cond, then_branch, else_branch } => {
                walk_expr_for_calls(cond, locals, globals, return_tys, hints);
                collect_calls_in_stmts(then_branch, locals, globals, return_tys, hints);
                collect_calls_in_stmts(else_branch, locals, globals, return_tys, hints);
            }
            HirStmt::While { cond, body } => {
                walk_expr_for_calls(cond, locals, globals, return_tys, hints);
                collect_calls_in_stmts(body, locals, globals, return_tys, hints);
            }
            HirStmt::ArrayStore { array, index, value } => {
                if let HirExpr::Local { name, .. } = array {
                    locals.insert(name.clone(), MirType::Ref(RefKind::Array));
                }
                walk_expr_for_calls(array, locals, globals, return_tys, hints);
                walk_expr_for_calls(index, locals, globals, return_tys, hints);
                walk_expr_for_calls(value, locals, globals, return_tys, hints);
            }
            _ => {}
        }
    }
}

fn walk_expr_for_calls(
    e: &HirExpr,
    locals: &mut HashMap<String, MirType>,
    globals: &HashMap<String, (u32, MirType)>,
    return_tys: &HashMap<String, MirType>,
    hints: &mut CallHints,
) {
    match e {
        HirExpr::Call { callee, args, .. } => {
            for (idx, a) in args.iter().enumerate() {
                if let Some(t) = guess_expr_ty(a, locals, globals, return_tys) {
                    hints.entry(callee.clone()).or_default().entry(idx).or_default().insert(t);
                }
                walk_expr_for_calls(a, locals, globals, return_tys, hints);
            }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            walk_expr_for_calls(lhs, locals, globals, return_tys, hints);
            walk_expr_for_calls(rhs, locals, globals, return_tys, hints);
        }
        HirExpr::Unary { operand, .. } => {
            walk_expr_for_calls(operand, locals, globals, return_tys, hints);
        }
        HirExpr::ArrayIndex { array, index, .. } => {
            walk_expr_for_calls(array, locals, globals, return_tys, hints);
            walk_expr_for_calls(index, locals, globals, return_tys, hints);
        }
        HirExpr::ArrayMethod { array, args, .. } => {
            walk_expr_for_calls(array, locals, globals, return_tys, hints);
            for a in args {
                walk_expr_for_calls(a, locals, globals, return_tys, hints);
            }
        }
        HirExpr::ArrayLit { elements, .. } => {
            for el in elements {
                walk_expr_for_calls(el, locals, globals, return_tys, hints);
            }
        }
        HirExpr::Ternary { condition, true_expr, false_expr, .. } => {
            walk_expr_for_calls(condition, locals, globals, return_tys, hints);
            walk_expr_for_calls(true_expr, locals, globals, return_tys, hints);
            walk_expr_for_calls(false_expr, locals, globals, return_tys, hints);
        }
        _ => {}
    }
}

pub(super) fn guess_expr_ty(
    e: &HirExpr,
    locals: &HashMap<String, MirType>,
    globals: &HashMap<String, (u32, MirType)>,
    return_tys: &HashMap<String, MirType>,
) -> Option<MirType> {
    match e {
        HirExpr::IntLit(..) => Some(MirType::I64),
        HirExpr::FloatLit(..) => Some(MirType::F64),
        HirExpr::BoolLit(..) => Some(MirType::Bool),
        HirExpr::StringLit(..) => Some(MirType::Ref(RefKind::String)),
        HirExpr::ArrayLit { .. } => Some(MirType::Ref(RefKind::Array)),
        HirExpr::ObjectLit { .. } => Some(MirType::Ref(RefKind::Object)),
        HirExpr::Local { name, .. } => {
            locals.get(name).copied().or_else(|| globals.get(name).map(|(_, ty)| *ty))
        }
        HirExpr::ArrayMethod { method, .. } if method == "join" => {
            Some(MirType::Ref(RefKind::String))
        }
        HirExpr::Binary { op, lhs, rhs, .. } => {
            let lt = guess_expr_ty(lhs, locals, globals, return_tys);
            let rt = guess_expr_ty(rhs, locals, globals, return_tys);
            if *op == HirBinOp::Add
                && (lt == Some(MirType::Ref(RefKind::String)) || rt == Some(MirType::Ref(RefKind::String)))
            {
                Some(MirType::Ref(RefKind::String))
            } else if lt == Some(MirType::F64) || rt == Some(MirType::F64) {
                Some(MirType::F64)
            } else if lt == Some(MirType::I64) && rt == Some(MirType::I64) {
                Some(MirType::I64)
            } else {
                None
            }
        }
        HirExpr::Call { callee, .. } => return_tys.get(callee).copied(),
        _ => None,
    }
}
