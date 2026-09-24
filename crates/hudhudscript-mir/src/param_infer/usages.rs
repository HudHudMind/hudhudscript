//! Parameter usage collection within a function.

use std::collections::{HashMap, HashSet};
use hudhudscript_types::{HirExpr, HirStmt};

#[derive(Default)]
pub(super) struct ParamUsage {
    pub(super) mutated_as_array: bool,
    pub(super) array_method: bool,
    pub(super) string_method: bool,
    pub(super) read_indexed: bool,
    pub(super) used_as_object: bool,
}

pub(super) fn collect_usages_in_stmts(
    stmts: &[HirStmt],
    params: &HashSet<String>,
    usages: &mut HashMap<String, ParamUsage>,
) {
    for s in stmts {
        match s {
            HirStmt::Let { value, .. } | HirStmt::Assign { value, .. } | HirStmt::Expr(value) => {
                collect_usages_in_expr(value, params, usages);
            }
            HirStmt::Return(Some(v)) => collect_usages_in_expr(v, params, usages),
            HirStmt::Return(None) => {}
            HirStmt::If { cond, then_branch, else_branch } => {
                collect_usages_in_expr(cond, params, usages);
                collect_usages_in_stmts(then_branch, params, usages);
                collect_usages_in_stmts(else_branch, params, usages);
            }
            HirStmt::While { cond, body } => {
                collect_usages_in_expr(cond, params, usages);
                collect_usages_in_stmts(body, params, usages);
            }
            HirStmt::ArrayStore { array, index, value } => {
                if let HirExpr::Local { name, .. } = array {
                    if let Some(u) = usages.get_mut(name.as_str()) {
                        u.mutated_as_array = true;
                    }
                }
                collect_usages_in_expr(array, params, usages);
                collect_usages_in_expr(index, params, usages);
                collect_usages_in_expr(value, params, usages);
            }
            HirStmt::PropertySet { object, value, .. } => {
                if let HirExpr::Local { name, .. } = object {
                    if let Some(u) = usages.get_mut(name.as_str()) {
                        u.used_as_object = true;
                    }
                }
                collect_usages_in_expr(object, params, usages);
                collect_usages_in_expr(value, params, usages);
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Throw(e) => collect_usages_in_expr(e, params, usages),
            HirStmt::Try { try_body, catch_body, finally_body, .. } => {
                collect_usages_in_stmts(try_body, params, usages);
                collect_usages_in_stmts(catch_body, params, usages);
                collect_usages_in_stmts(finally_body, params, usages);
            }
        }
    }
}

pub(super) fn collect_usages_in_expr(
    e: &HirExpr,
    params: &HashSet<String>,
    usages: &mut HashMap<String, ParamUsage>,
) {
    match e {
        HirExpr::ArrayIndex { array, index, .. } => {
            if let HirExpr::Local { name, .. } = array.as_ref() {
                if let Some(u) = usages.get_mut(name.as_str()) {
                    u.read_indexed = true;
                }
            }
            collect_usages_in_expr(array, params, usages);
            collect_usages_in_expr(index, params, usages);
        }
        HirExpr::ArrayStore { array, index } => {
            if let HirExpr::Local { name, .. } = array.as_ref() {
                if let Some(u) = usages.get_mut(name.as_str()) {
                    u.mutated_as_array = true;
                }
            }
            collect_usages_in_expr(array, params, usages);
            collect_usages_in_expr(index, params, usages);
        }
        HirExpr::ArrayMethod { array, method, args, .. } => {
            if let HirExpr::Local { name, .. } = array.as_ref() {
                if let Some(u) = usages.get_mut(name.as_str()) {
                    match method.as_str() {
                        "push" | "pop" | "unshift" | "shift" => u.array_method = true,
                        "substring" | "slice" | "indexOf" | "charAt" | "charCodeAt" => {
                            u.string_method = true
                        }
                        _ => {}
                    }
                }
            }
            collect_usages_in_expr(array, params, usages);
            for a in args {
                collect_usages_in_expr(a, params, usages);
            }
        }
        HirExpr::PropertyGet { object, name: prop_name, .. } => {
            if let HirExpr::Local { name, .. } = object.as_ref() {
                if let Some(u) = usages.get_mut(name.as_str()) {
                    if prop_name != "length" && prop_name != "len" {
                        u.used_as_object = true;
                    }
                }
            }
            collect_usages_in_expr(object, params, usages);
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            collect_usages_in_expr(lhs, params, usages);
            collect_usages_in_expr(rhs, params, usages);
        }
        HirExpr::Unary { operand, .. } => collect_usages_in_expr(operand, params, usages),
        HirExpr::Call { args, .. } => {
            for a in args {
                collect_usages_in_expr(a, params, usages);
            }
        }
        HirExpr::ArrayLit { elements, .. } => {
            for el in elements {
                collect_usages_in_expr(el, params, usages);
            }
        }
        HirExpr::ObjectLit { properties, .. } => {
            for (_, v) in properties {
                collect_usages_in_expr(v, params, usages);
            }
        }
        HirExpr::Ternary { condition, true_expr, false_expr, .. } => {
            collect_usages_in_expr(condition, params, usages);
            collect_usages_in_expr(true_expr, params, usages);
            collect_usages_in_expr(false_expr, params, usages);
        }
        _ => {}
    }
}
