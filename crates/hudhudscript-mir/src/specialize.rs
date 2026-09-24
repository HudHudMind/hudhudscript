//! Higher-order call specialization (monomorphization) for typed HIR modules.

use std::collections::{BTreeMap, HashSet};
use hudhudscript_types::{HirExpr, HirFunction, HirModule, HirStmt};

/// Specialize any call passing a top-level function as an argument.
pub fn specialize_module(module: &mut HirModule) {
    for _ in 0..5 {
        let mut new_funcs: BTreeMap<String, HirFunction> = BTreeMap::new();
        let func_names: HashSet<String> = module.functions.keys().cloned().collect();
        let targets = module.functions.clone();

        for func in module.functions.values_mut() {
            specialize_stmts(&mut func.body, &func_names, &targets, &mut new_funcs);
        }

        if new_funcs.is_empty() {
            break;
        }
        for (name, f) in new_funcs {
            module.functions.insert(name, f);
        }
    }
    prune_unreachable(module);
}

fn specialize_stmts(
    stmts: &mut [HirStmt],
    func_names: &HashSet<String>,
    all_funcs: &BTreeMap<String, HirFunction>,
    new_funcs: &mut BTreeMap<String, HirFunction>,
) {
    for s in stmts {
        match s {
            HirStmt::Let { value, .. } | HirStmt::Assign { value, .. } | HirStmt::Expr(value) => {
                specialize_expr(value, func_names, all_funcs, new_funcs);
            }
            HirStmt::Return(Some(e)) => specialize_expr(e, func_names, all_funcs, new_funcs),
            HirStmt::Return(None) => {}
            HirStmt::If { cond, then_branch, else_branch } => {
                specialize_expr(cond, func_names, all_funcs, new_funcs);
                specialize_stmts(then_branch, func_names, all_funcs, new_funcs);
                specialize_stmts(else_branch, func_names, all_funcs, new_funcs);
            }
            HirStmt::While { cond, body } => {
                specialize_expr(cond, func_names, all_funcs, new_funcs);
                specialize_stmts(body, func_names, all_funcs, new_funcs);
            }
            HirStmt::ArrayStore { array, index, value } => {
                specialize_expr(array, func_names, all_funcs, new_funcs);
                specialize_expr(index, func_names, all_funcs, new_funcs);
                specialize_expr(value, func_names, all_funcs, new_funcs);
            }
            HirStmt::PropertySet { object, value, .. } => {
                specialize_expr(object, func_names, all_funcs, new_funcs);
                specialize_expr(value, func_names, all_funcs, new_funcs);
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Throw(e) => specialize_expr(e, func_names, all_funcs, new_funcs),
            HirStmt::Try { try_body, catch_body, finally_body, .. } => {
                specialize_stmts(try_body, func_names, all_funcs, new_funcs);
                specialize_stmts(catch_body, func_names, all_funcs, new_funcs);
                specialize_stmts(finally_body, func_names, all_funcs, new_funcs);
            }
        }
    }
}

fn specialize_expr(
    expr: &mut HirExpr,
    func_names: &HashSet<String>,
    all_funcs: &BTreeMap<String, HirFunction>,
    new_funcs: &mut BTreeMap<String, HirFunction>,
) {
    match expr {
        HirExpr::Call { callee, args, .. } => {
            for a in args.iter_mut() {
                specialize_expr(a, func_names, all_funcs, new_funcs);
            }
            if let Some(target) = all_funcs.get(callee) {
                let mut substitutions: Vec<(usize, String, String)> = Vec::new();
                for (idx, arg) in args.iter().enumerate() {
                    if let HirExpr::Local { name, .. } = arg {
                        if func_names.contains(name) && idx < target.params.len() {
                            let param_name = target.params[idx].name.clone();
                            substitutions.push((idx, param_name, name.clone()));
                        }
                    }
                }
                if !substitutions.is_empty() {
                    let mut spec_name = callee.clone();
                    for (_, _, fn_name) in &substitutions {
                        spec_name = format!("{spec_name}${fn_name}");
                    }
                    if !all_funcs.contains_key(&spec_name) && !new_funcs.contains_key(&spec_name) {
                        let mut specialized = target.clone();
                        specialized.name = spec_name.clone();
                        for (_, param_name, fn_name) in &substitutions {
                            substitute_calls_in_stmts(&mut specialized.body, param_name, fn_name);
                        }
                        new_funcs.insert(spec_name.clone(), specialized);
                    }
                    for (idx, _, _) in &substitutions {
                        args[*idx] = HirExpr::IntLit(0);
                    }
                    *callee = spec_name;
                }
            }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            specialize_expr(lhs, func_names, all_funcs, new_funcs);
            specialize_expr(rhs, func_names, all_funcs, new_funcs);
        }
        HirExpr::Unary { operand, .. } => {
            specialize_expr(operand, func_names, all_funcs, new_funcs);
        }
        HirExpr::ArrayLit { elements, .. } => {
            for e in elements {
                specialize_expr(e, func_names, all_funcs, new_funcs);
            }
        }
        HirExpr::ArrayIndex { array, index, .. } => {
            specialize_expr(array, func_names, all_funcs, new_funcs);
            specialize_expr(index, func_names, all_funcs, new_funcs);
        }
        HirExpr::ArrayMethod { array, args, .. } => {
            specialize_expr(array, func_names, all_funcs, new_funcs);
            for a in args {
                specialize_expr(a, func_names, all_funcs, new_funcs);
            }
        }
        HirExpr::ObjectLit { properties, .. } => {
            for (_, v) in properties {
                specialize_expr(v, func_names, all_funcs, new_funcs);
            }
        }
        HirExpr::PropertyGet { object, .. } => {
            specialize_expr(object, func_names, all_funcs, new_funcs);
        }
        HirExpr::Ternary { condition, true_expr, false_expr, .. } => {
            specialize_expr(condition, func_names, all_funcs, new_funcs);
            specialize_expr(true_expr, func_names, all_funcs, new_funcs);
            specialize_expr(false_expr, func_names, all_funcs, new_funcs);
        }
        _ => {}
    }
}

fn substitute_calls_in_stmts(stmts: &mut [HirStmt], from_param: &str, to_func: &str) {
    for s in stmts {
        match s {
            HirStmt::Let { value, .. } | HirStmt::Assign { value, .. } | HirStmt::Expr(value) => {
                substitute_calls_in_expr(value, from_param, to_func);
            }
            HirStmt::Return(Some(e)) => substitute_calls_in_expr(e, from_param, to_func),
            HirStmt::Return(None) => {}
            HirStmt::If { cond, then_branch, else_branch } => {
                substitute_calls_in_expr(cond, from_param, to_func);
                substitute_calls_in_stmts(then_branch, from_param, to_func);
                substitute_calls_in_stmts(else_branch, from_param, to_func);
            }
            HirStmt::While { cond, body } => {
                substitute_calls_in_expr(cond, from_param, to_func);
                substitute_calls_in_stmts(body, from_param, to_func);
            }
            HirStmt::ArrayStore { array, index, value } => {
                substitute_calls_in_expr(array, from_param, to_func);
                substitute_calls_in_expr(index, from_param, to_func);
                substitute_calls_in_expr(value, from_param, to_func);
            }
            HirStmt::PropertySet { object, value, .. } => {
                substitute_calls_in_expr(object, from_param, to_func);
                substitute_calls_in_expr(value, from_param, to_func);
            }
            HirStmt::Break | HirStmt::Continue => {}
            HirStmt::Throw(e) => substitute_calls_in_expr(e, from_param, to_func),
            HirStmt::Try { try_body, catch_body, finally_body, .. } => {
                substitute_calls_in_stmts(try_body, from_param, to_func);
                substitute_calls_in_stmts(catch_body, from_param, to_func);
                substitute_calls_in_stmts(finally_body, from_param, to_func);
            }
        }
    }
}

fn substitute_calls_in_expr(expr: &mut HirExpr, from_param: &str, to_func: &str) {
    match expr {
        HirExpr::Call { callee, args, .. } => {
            if callee == from_param {
                *callee = to_func.to_string();
            }
            for a in args {
                substitute_calls_in_expr(a, from_param, to_func);
            }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            substitute_calls_in_expr(lhs, from_param, to_func);
            substitute_calls_in_expr(rhs, from_param, to_func);
        }
        HirExpr::Unary { operand, .. } => {
            substitute_calls_in_expr(operand, from_param, to_func);
        }
        HirExpr::ArrayLit { elements, .. } => {
            for e in elements {
                substitute_calls_in_expr(e, from_param, to_func);
            }
        }
        HirExpr::ArrayIndex { array, index, .. } => {
            substitute_calls_in_expr(array, from_param, to_func);
            substitute_calls_in_expr(index, from_param, to_func);
        }
        HirExpr::ArrayMethod { array, args, .. } => {
            substitute_calls_in_expr(array, from_param, to_func);
            for a in args {
                substitute_calls_in_expr(a, from_param, to_func);
            }
        }
        HirExpr::ObjectLit { properties, .. } => {
            for (_, v) in properties {
                substitute_calls_in_expr(v, from_param, to_func);
            }
        }
        HirExpr::PropertyGet { object, .. } => {
            substitute_calls_in_expr(object, from_param, to_func);
        }
        HirExpr::Ternary { condition, true_expr, false_expr, .. } => {
            substitute_calls_in_expr(condition, from_param, to_func);
            substitute_calls_in_expr(true_expr, from_param, to_func);
            substitute_calls_in_expr(false_expr, from_param, to_func);
        }
        _ => {}
    }
}

fn prune_unreachable(module: &mut HirModule) {
    let mut worklist = Vec::new();
    let mut reachable = HashSet::new();

    if module.functions.contains_key("_hudhud_init") {
        worklist.push("_hudhud_init".to_string());
        reachable.insert("_hudhud_init".to_string());
    }
    if module.functions.contains_key("main") {
        worklist.push("main".to_string());
        reachable.insert("main".to_string());
    }

    if worklist.is_empty() {
        return;
    }

    while let Some(fname) = worklist.pop() {
        if let Some(f) = module.functions.get(&fname) {
            let mut called = HashSet::new();
            collect_called_in_stmts(&f.body, &mut called);
            for c in called {
                if module.functions.contains_key(&c) && reachable.insert(c.clone()) {
                    worklist.push(c);
                }
            }
        }
    }

    module.functions.retain(|k, _| reachable.contains(k));
}

fn collect_called_in_stmts(stmts: &[HirStmt], called: &mut HashSet<String>) {
    for s in stmts {
        match s {
            HirStmt::Let { value, .. } | HirStmt::Assign { value, .. } | HirStmt::Expr(value) => {
                collect_called_in_expr(value, called);
            }
            HirStmt::Return(Some(e)) => collect_called_in_expr(e, called),
            HirStmt::Return(None) | HirStmt::Break | HirStmt::Continue => {}
            HirStmt::If { cond, then_branch, else_branch } => {
                collect_called_in_expr(cond, called);
                collect_called_in_stmts(then_branch, called);
                collect_called_in_stmts(else_branch, called);
            }
            HirStmt::While { cond, body } => {
                collect_called_in_expr(cond, called);
                collect_called_in_stmts(body, called);
            }
            HirStmt::ArrayStore { array, index, value } => {
                collect_called_in_expr(array, called);
                collect_called_in_expr(index, called);
                collect_called_in_expr(value, called);
            }
            HirStmt::PropertySet { object, value, .. } => {
                collect_called_in_expr(object, called);
                collect_called_in_expr(value, called);
            }
            HirStmt::Throw(e) => collect_called_in_expr(e, called),
            HirStmt::Try { try_body, catch_body, finally_body, .. } => {
                collect_called_in_stmts(try_body, called);
                collect_called_in_stmts(catch_body, called);
                collect_called_in_stmts(finally_body, called);
            }
        }
    }
}

fn collect_called_in_expr(expr: &HirExpr, called: &mut HashSet<String>) {
    match expr {
        HirExpr::Call { callee, args, .. } => {
            called.insert(callee.clone());
            for a in args {
                collect_called_in_expr(a, called);
            }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            collect_called_in_expr(lhs, called);
            collect_called_in_expr(rhs, called);
        }
        HirExpr::Unary { operand, .. } => collect_called_in_expr(operand, called),
        HirExpr::ArrayLit { elements, .. } => {
            for e in elements {
                collect_called_in_expr(e, called);
            }
        }
        HirExpr::ArrayIndex { array, index, .. } => {
            collect_called_in_expr(array, called);
            collect_called_in_expr(index, called);
        }
        HirExpr::ArrayMethod { array, args, .. } => {
            collect_called_in_expr(array, called);
            for a in args {
                collect_called_in_expr(a, called);
            }
        }
        HirExpr::ObjectLit { properties, .. } => {
            for (_, v) in properties {
                collect_called_in_expr(v, called);
            }
        }
        HirExpr::PropertyGet { object, .. } => collect_called_in_expr(object, called),
        HirExpr::Ternary { condition, true_expr, false_expr, .. } => {
            collect_called_in_expr(condition, called);
            collect_called_in_expr(true_expr, called);
            collect_called_in_expr(false_expr, called);
        }
        _ => {}
    }
}

