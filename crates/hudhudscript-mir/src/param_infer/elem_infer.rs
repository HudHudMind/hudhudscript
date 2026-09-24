//! Array element type inference across functions in a module.

use std::collections::HashMap;
use hudhudscript_types::{HirExpr, HirModule, HirStmt};
use crate::mir::MirType;

pub fn infer_module_array_elem_types(
    module: &HirModule,
    return_tys: &HashMap<String, MirType>,
) -> HashMap<String, HashMap<String, MirType>> {
    let mut all: HashMap<String, HashMap<String, MirType>> = HashMap::new();
    for _ in 0..5 {
        let mut changed = false;
        for (f_name, f) in &module.functions {
            let mut current = all.get(f_name).cloned().unwrap_or_default();
            let initial_len = current.len();
            let mut src_map = HashMap::new();
            scan_stmts(&f.body, &mut current, &mut src_map, return_tys);
            propagate_call_args(&f.body, &all, &mut current, module);
            if current.len() != initial_len || all.get(f_name) != Some(&current) {
                all.insert(f_name.clone(), current);
                changed = true;
            }
        }
        // Forward propagation: caller args to callee params
        for f in module.functions.values() {
            propagate_to_callees(&f.body, &mut all, module);
        }
        if !changed {
            break;
        }
    }
    all
}

fn scan_stmts(
    stmts: &[HirStmt],
    elem_tys: &mut HashMap<String, MirType>,
    src: &mut HashMap<String, String>,
    return_tys: &HashMap<String, MirType>,
) {
    for s in stmts {
        match s {
            HirStmt::ArrayStore { array, value, .. } => {
                if let HirExpr::Local { name, .. } = array {
                    if is_float(value, return_tys, elem_tys, src) {
                        elem_tys.insert(name.clone(), MirType::F64);
                    } else if is_string(value, return_tys, elem_tys) {
                        elem_tys.insert(name.clone(), MirType::Ref(crate::mir::RefKind::String));
                    }
                }
            }
            HirStmt::Expr(e) | HirStmt::Let { value: e, .. } | HirStmt::Assign { value: e, .. } => {
                if let HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } = s {
                    if let HirExpr::ArrayLit { elements, .. } = value {
                        if let Some(HirExpr::ArrayLit { .. }) = elements.first() {
                            elem_tys.insert(name.clone(), MirType::Ref(crate::mir::RefKind::Array));
                        }
                    }
                    // Provenance: `let v = a[i]` → v'nin kaynağı a (v float
                    // bağlama girerse a'nın eleman tipi F64 işaretlenir)
                    if let HirExpr::ArrayIndex { array, .. } = value {
                        if let HirExpr::Local { name: arr, .. } = array.as_ref() {
                            src.insert(name.clone(), arr.clone());
                        }
                    }
                    // Float yerel işareti: `let dx = <float> ifade>` → dx F64.
                    // (ArrayLit/StringLit hariç — onlar handle.)
                    if !matches!(value, HirExpr::ArrayLit { .. } | HirExpr::StringLit(..))
                        && is_float(value, return_tys, elem_tys, src)
                    {
                        // Yalnız YEREL isimleri işaretle: dizi adlarını ezme
                        let is_array_name = matches!(
                            elem_tys.get(name),
                            Some(MirType::Ref(_))
                        );
                        if !is_array_name {
                            elem_tys.insert(name.clone(), MirType::F64);
                        }
                    }
                }
                scan_expr(e, elem_tys, src, return_tys);
            }
            HirStmt::Return(Some(e)) => {
                // return <float ifade> — elem tipini işaretleyebilir
                scan_expr(e, elem_tys, src, return_tys);
            }
            HirStmt::If { cond, then_branch, else_branch } => {
                scan_expr(cond, elem_tys, src, return_tys);
                scan_stmts(then_branch, elem_tys, src, return_tys);
                scan_stmts(else_branch, elem_tys, src, return_tys);
            }
            HirStmt::While { cond, body } => {
                scan_expr(cond, elem_tys, src, return_tys);
                scan_stmts(body, elem_tys, src, return_tys);
            }
            _ => {}
        }
    }
}

fn scan_expr(
    e: &HirExpr,
    elem_tys: &mut HashMap<String, MirType>,
    src: &mut HashMap<String, String>,
    return_tys: &HashMap<String, MirType>,
) {
    match e {
        HirExpr::ArrayMethod { array, method, args, .. } if method == "push" => {
            if let HirExpr::Local { name, .. } = array.as_ref() {
                if let Some(arg) = args.first() {
                    if is_float(arg, return_tys, elem_tys, src) {
                        elem_tys.insert(name.clone(), MirType::F64);
                    } else if is_string(arg, return_tys, elem_tys) {
                        elem_tys.insert(name.clone(), MirType::Ref(crate::mir::RefKind::String));
                    }
                }
            }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            check_context(lhs, rhs, elem_tys, src, return_tys);
            check_context(rhs, lhs, elem_tys, src, return_tys);
            scan_expr(lhs, elem_tys, src, return_tys);
            scan_expr(rhs, elem_tys, src, return_tys);
        }
        _ => {}
    }
}

fn check_context(
    idx_expr: &HirExpr,
    other: &HirExpr,
    elem_tys: &mut HashMap<String, MirType>,
    src: &HashMap<String, String>,
    return_tys: &HashMap<String, MirType>,
) {
    if let HirExpr::ArrayIndex { array, .. } = idx_expr {
        if let HirExpr::Local { name, .. } = array.as_ref() {
            if is_float(other, return_tys, elem_tys, src) {
                elem_tys.insert(name.clone(), MirType::F64);
            }
        }
    } else if let HirExpr::Local { name, .. } = idx_expr {
        // `v * 2.0` ve v ← a[i] ise a'nın elemanı F64'tür (provenance)
        if is_float(other, return_tys, elem_tys, src) {
            if let Some(arr) = src.get(name) {
                if !matches!(elem_tys.get(arr), Some(MirType::Ref(_))) {
                    elem_tys.insert(arr.clone(), MirType::F64);
                }
            }
        }
    }
}

fn is_float(
    e: &HirExpr,
    return_tys: &HashMap<String, MirType>,
    elem_tys: &HashMap<String, MirType>,
    src: &HashMap<String, String>,
) -> bool {
    match e {
        HirExpr::FloatLit(..) => true,
        HirExpr::Call { callee, .. } => return_tys.get(callee) == Some(&MirType::F64),
        HirExpr::ArrayIndex { array, .. } => {
            if let HirExpr::Local { name, .. } = array.as_ref() {
                elem_tys.get(name) == Some(&MirType::F64)
            } else {
                false
            }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            is_float(lhs, return_tys, elem_tys, src) || is_float(rhs, return_tys, elem_tys, src)
        }
        // Float yerel: `let dx = <float>` ile işaretlenmiş (veya kaynağı
        // float dizi elemanı olan) yerel isim
        HirExpr::Local { name, .. } => {
            if elem_tys.get(name) == Some(&MirType::F64) {
                return true;
            }
            src.get(name)
                .and_then(|arr| elem_tys.get(arr))
                .is_some_and(|t| *t == MirType::F64)
        }
        _ => false,
    }
}

fn is_string(
    e: &HirExpr,
    return_tys: &HashMap<String, MirType>,
    elem_tys: &HashMap<String, MirType>,
) -> bool {
    match e {
        HirExpr::StringLit(..) => true,
        HirExpr::Call { callee, .. } => {
            return_tys.get(callee) == Some(&MirType::Ref(crate::mir::RefKind::String))
        }
        HirExpr::ArrayIndex { array, .. } => {
            if let HirExpr::Local { name, .. } = array.as_ref() {
                elem_tys.get(name) == Some(&MirType::Ref(crate::mir::RefKind::String))
            } else {
                false
            }
        }
        HirExpr::Binary { op, lhs, rhs, .. } => {
            if *op == hudhudscript_types::HirBinOp::Add {
                is_string(lhs, return_tys, elem_tys) || is_string(rhs, return_tys, elem_tys)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn propagate_call_args(
    stmts: &[HirStmt],
    all: &HashMap<String, HashMap<String, MirType>>,
    caller_elem_tys: &mut HashMap<String, MirType>,
    module: &HirModule,
) {
    for s in stmts {
        match s {
            HirStmt::Expr(e) | HirStmt::Let { value: e, .. } | HirStmt::Assign { value: e, .. } => {
                if let HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } = s {
                    if let HirExpr::Call { callee, .. } = value {
                        if let Some(callee_fn) = module.functions.get(callee) {
                            if let Some(callee_map) = all.get(callee) {
                                for stmt in &callee_fn.body {
                                    if let HirStmt::Return(Some(HirExpr::Local { name: ret_var, .. })) = stmt {
                                        if let Some(ty) = callee_map.get(ret_var).copied() {
                                            caller_elem_tys.insert(name.clone(), ty);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let HirExpr::Call { callee, args, .. } = e {
                    if let Some(callee_fn) = module.functions.get(callee) {
                        let callee_elem_tys = all.get(callee);
                        for (idx, arg) in args.iter().enumerate() {
                            if let HirExpr::Local { name: arg_name, .. } = arg {
                                if let Some(param) = callee_fn.params.get(idx) {
                                    if let Some(ty) = callee_elem_tys.and_then(|m| m.get(&param.name)).copied() {
                                        caller_elem_tys.insert(arg_name.clone(), ty);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            HirStmt::If { then_branch, else_branch, .. } => {
                propagate_call_args(then_branch, all, caller_elem_tys, module);
                propagate_call_args(else_branch, all, caller_elem_tys, module);
            }
            HirStmt::While { body, .. } => {
                propagate_call_args(body, all, caller_elem_tys, module);
            }
            _ => {}
        }
    }
}

fn propagate_to_callees(
    stmts: &[HirStmt],
    all: &mut HashMap<String, HashMap<String, MirType>>,
    module: &HirModule,
) {
    for s in stmts {
        match s {
            HirStmt::Expr(e) | HirStmt::Let { value: e, .. } | HirStmt::Assign { value: e, .. } => {
                if let HirExpr::Call { callee, args, .. } = e {
                    if let Some(callee_fn) = module.functions.get(callee) {
                        for (idx, arg) in args.iter().enumerate() {
                            if let HirExpr::Local { name: arg_name, .. } = arg {
                                if let Some(arg_elem_ty) = all.values().find_map(|m| m.get(arg_name).copied()) {
                                    if let Some(param) = callee_fn.params.get(idx) {
                                        all.entry(callee.clone()).or_default().insert(param.name.clone(), arg_elem_ty);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            HirStmt::If { then_branch, else_branch, .. } => {
                propagate_to_callees(then_branch, all, module);
                propagate_to_callees(else_branch, all, module);
            }
            HirStmt::While { body, .. } => {
                propagate_to_callees(body, all, module);
            }
            _ => {}
        }
    }
}
