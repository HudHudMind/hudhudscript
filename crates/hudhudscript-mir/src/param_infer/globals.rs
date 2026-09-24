//! Module globals and return type inference.

use std::collections::{HashMap, HashSet};
use hudhudscript_types::{HirExpr, HirFunction, HirModule, HirStmt, Type};
use crate::mir::{MirType, RefKind};
use super::usages::{collect_usages_in_stmts, ParamUsage};

pub fn infer_module_globals(module: &HirModule) -> HashMap<String, (u32, MirType)> {
    // YALNIZ üst-düzey let'ler modül-globaldir: döngü/if gövdelerindeki
    // `let x` blok-yerelidir (her iterasyonda yeniden bağlanır) — global
    // yapılırsa tip bilgisi kaybolur ve okuma yanlış çevrilir (bkz. g8).
    let candidate_globals = module
        .functions
        .get("_hudhud_init")
        .map(|init| {
            init.body
                .iter()
                .filter_map(|s| match s {
                    HirStmt::Let { name, .. } => Some(name.clone()),
                    _ => None,
                })
                .collect::<HashSet<String>>()
        })
        .unwrap_or_default();
    let used_in_other = collect_free_names_in_other_funcs(module);
    let mut all_names: Vec<String> = candidate_globals
        .into_iter()
        .chain(used_in_other)
        .collect();
    all_names.sort();
    all_names.dedup();
    // Modül fonksiyon dönüş tipleri: `let e1 = energy(bodies)` gibi global
    // başlatıcılarında çağrı sonucunun tipini çözmek için (f64 aksi hâlde
    // I64 sanılıp bit deseni yazdırılıyordu).
    let return_tys = infer_module_return_types(module);
    let mut globals = HashMap::new();
    for (slot, name) in all_names.into_iter().enumerate() {
        globals.insert(name.clone(), (slot as u32, infer_single_global(module, &name, &return_tys)));
    }
    globals
}

fn infer_single_global(module: &HirModule, name: &str, known: &HashMap<String, MirType>) -> MirType {
    if let Some(init) = module.functions.get("_hudhud_init") {
        // Init gövdesini sırayla tara: local tipleri biriktirerek hesaplı
        // ifadelerin (x * x, i * 1.0, ternary, Math.*) tipini çıkar.
        let mut locals: HashMap<String, MirType> = HashMap::new();
        for s in &init.body {
            if let HirStmt::Let { name: n, value, .. } | HirStmt::Assign { name: n, value } = s {
                let ty = infer_expr_ty(value, &locals, known);
                if n == name {
                    match ty {
                        Some(t) => return t,
                        None => {}
                    }
                }
                locals.insert(n.clone(), ty.unwrap_or(MirType::I64));
            }
        }
    }
    for (fn_name, f) in &module.functions {
        if fn_name == "_hudhud_init" {
            continue;
        }
        let mut usages = HashMap::new();
        usages.insert(name.to_string(), ParamUsage::default());
        let mut set = HashSet::new();
        set.insert(name.to_string());
        collect_usages_in_stmts(&f.body, &set, &mut usages);
        let u = usages.get(name).unwrap();
        if u.mutated_as_array || u.array_method {
            return MirType::Ref(RefKind::Array);
        }
        if u.string_method {
            return MirType::Ref(RefKind::String);
        }
        if u.used_as_object {
            return MirType::Ref(RefKind::Object);
        }
    }
    MirType::I64
}

pub fn infer_module_return_types(module: &HirModule) -> HashMap<String, MirType> {
    infer_module_return_types_with_hints(module, &HashMap::new())
}

pub fn infer_module_return_types_with_hints(
    module: &HirModule,
    param_hints: &HashMap<String, HashMap<String, MirType>>,
) -> HashMap<String, MirType> {
    // Fixpoint: `return foo(x)` çağrılarının tipi, foo'nun tipi çözülünce
    // yayılır (tek pas'ta None → I64 varsayımı f64 bit desenini bozuyordu).
    let mut out: HashMap<String, MirType> = HashMap::new();
    for _ in 0..8 {
        let mut changed = false;
        for (name, f) in &module.functions {
            let f_hints = param_hints.get(name);
            let ty = infer_func_return_type(f, f_hints, &out);
            if out.get(name) != Some(&ty) {
                out.insert(name.clone(), ty);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    out
}

fn infer_func_return_type(f: &HirFunction, param_hints: Option<&HashMap<String, MirType>>, known: &HashMap<String, MirType>) -> MirType {
    for s in &f.body {
        if let Some(t) = check_stmt_return(f, s, param_hints, known) {
            return t;
        }
    }
    match f.return_type {
        Type::Number => MirType::I64,
        Type::Boolean => MirType::Bool,
        _ => MirType::Unit,
    }
}

fn check_stmt_return(f: &HirFunction, s: &HirStmt, param_hints: Option<&HashMap<String, MirType>>, known: &HashMap<String, MirType>) -> Option<MirType> {
    let mut visited = HashSet::new();
    check_stmt_return_inner(f, s, &mut visited, param_hints, known)
}

fn check_stmt_return_inner(
    f: &HirFunction,
    s: &HirStmt,
    visited: &mut HashSet<String>,
    param_hints: Option<&HashMap<String, MirType>>,
    known: &HashMap<String, MirType>,
) -> Option<MirType> {
    match s {
        HirStmt::Return(Some(e)) => guess_return_expr_type(f, e, visited, param_hints, known),
        HirStmt::If { then_branch, else_branch, .. } => {
            for st in then_branch.iter().chain(else_branch) {
                if let Some(t) = check_stmt_return_inner(f, st, visited, param_hints, known) {
                    return Some(t);
                }
            }
            None
        }
        HirStmt::While { body, .. } => {
            for st in body {
                if let Some(t) = check_stmt_return_inner(f, st, visited, param_hints, known) {
                    return Some(t);
                }
            }
            None
        }
        HirStmt::Try { try_body, catch_body, finally_body, .. } => {
            for st in try_body.iter().chain(catch_body).chain(finally_body) {
                if let Some(t) = check_stmt_return_inner(f, st, visited, param_hints, known) {
                    return Some(t);
                }
            }
            None
        }
        _ => None,
    }
}

fn guess_return_expr_type(
    f: &HirFunction,
    e: &HirExpr,
    visited: &mut HashSet<String>,
    param_hints: Option<&HashMap<String, MirType>>,
    known: &HashMap<String, MirType>,
) -> Option<MirType> {
    match e {
        HirExpr::ArrayLit { .. } => Some(MirType::Ref(RefKind::Array)),
        HirExpr::StringLit(..) => Some(MirType::Ref(RefKind::String)),
        HirExpr::FloatLit(..) => Some(MirType::F64),
        HirExpr::IntLit(..) => Some(MirType::I64),
        HirExpr::BoolLit(..) => Some(MirType::Bool),
        HirExpr::ArrayMethod { method, .. } if method == "join" => {
            Some(MirType::Ref(RefKind::String))
        }
        HirExpr::Local { name, .. } => {
            if let Some(hints) = param_hints {
                if let Some(ty) = hints.get(name).copied() {
                    if ty == MirType::F64 {
                        return Some(MirType::F64);
                    }
                }
            }
            check_local_return_type(f, name, visited, param_hints, known)
        }
        HirExpr::Binary { op, lhs, rhs, .. } => {
            let lt = guess_return_expr_type(f, lhs, visited, param_hints, known);
            let rt = guess_return_expr_type(f, rhs, visited, param_hints, known);
            if *op == hudhudscript_types::HirBinOp::Add
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
        HirExpr::Unary { operand, .. } => guess_return_expr_type(f, operand, visited, param_hints, known),
        // Çağrı dönüş tipi: bilinen modül fonksiyonu veya builtin
        HirExpr::Call { callee, .. } => match callee.as_str() {
            "Date.to_millis" => Some(MirType::I64),
            "typeof" => Some(MirType::Ref(RefKind::String)),
            c if c.starts_with("Math.") => Some(MirType::F64),
            c => known.get(c).copied().filter(|t| *t != MirType::Unit),
        },
        _ => None,
    }
}

fn check_local_return_type(
    f: &HirFunction,
    local_name: &str,
    visited: &mut HashSet<String>,
    param_hints: Option<&HashMap<String, MirType>>,
    known: &HashMap<String, MirType>,
) -> Option<MirType> {
    if !visited.insert(local_name.to_string()) {
        return None;
    }
    scan_stmts_for_local_type(&f.body, f, local_name, visited, param_hints, known)
}

fn scan_stmts_for_local_type(
    stmts: &[HirStmt],
    f: &HirFunction,
    local_name: &str,
    visited: &mut HashSet<String>,
    param_hints: Option<&HashMap<String, MirType>>,
    known: &HashMap<String, MirType>,
) -> Option<MirType> {
    for s in stmts {
        match s {
            HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } => {
                if name == local_name {
                    match value {
                        HirExpr::ArrayLit { .. } => return Some(MirType::Ref(RefKind::Array)),
                        HirExpr::StringLit(..) => return Some(MirType::Ref(RefKind::String)),
                        HirExpr::FloatLit(..) => return Some(MirType::F64),
                        HirExpr::Binary { .. } => {
                            if let Some(t) = guess_return_expr_type(f, value, visited, param_hints, known) {
                                return Some(t);
                            }
                        }
                        _ => {}
                    }
                }
            }
            HirStmt::If { then_branch, else_branch, .. } => {
                if let Some(t) = scan_stmts_for_local_type(then_branch, f, local_name, visited, param_hints, known) {
                    return Some(t);
                }
                if let Some(t) = scan_stmts_for_local_type(else_branch, f, local_name, visited, param_hints, known) {
                    return Some(t);
                }
            }
            HirStmt::While { body, .. } => {
                if let Some(t) = scan_stmts_for_local_type(body, f, local_name, visited, param_hints, known) {
                    return Some(t);
                }
            }
            _ => {}
        }
    }
    None
}

pub(super) fn collect_lets_deep(stmts: &[HirStmt]) -> HashSet<String> {
    let mut out = HashSet::new();
    fn walk(s: &HirStmt, out: &mut HashSet<String>) {
        match s {
            HirStmt::Let { name, .. } | HirStmt::Assign { name, .. } => { out.insert(name.clone()); }
            HirStmt::If { then_branch, else_branch, .. } => {
                for st in then_branch.iter().chain(else_branch) { walk(st, out); }
            }
            HirStmt::While { body, .. } => {
                for st in body { walk(st, out); }
            }
            HirStmt::Try { try_body, catch_body, finally_body, .. } => {
                for st in try_body.iter().chain(catch_body).chain(finally_body) { walk(st, out); }
            }
            _ => {}
        }
    }
    for s in stmts { walk(s, &mut out); }
    out
}

pub(super) fn collect_free_names_in_other_funcs(module: &HirModule) -> HashSet<String> {
    let mut free = HashSet::new();
    for (name, func) in &module.functions {
        if name == "_hudhud_init" { continue; }
        let mut locals: HashSet<String> = func.params.iter().map(|p| p.name.clone()).collect();
        for s in &func.body {
            walk_free_stmt(s, &mut locals, &mut free);
        }
    }
    free
}

fn walk_free_stmt(s: &HirStmt, locals: &mut HashSet<String>, free: &mut HashSet<String>) {
    match s {
        HirStmt::Let { name, value, .. } => {
            walk_free_expr(value, locals, free);
            locals.insert(name.clone());
        }
        HirStmt::Assign { name, value } => {
            if !locals.contains(name) { free.insert(name.clone()); }
            walk_free_expr(value, locals, free);
        }
        HirStmt::Expr(e) | HirStmt::Return(Some(e)) | HirStmt::Throw(e) => walk_free_expr(e, locals, free),
        HirStmt::Return(None) | HirStmt::Break | HirStmt::Continue => {}
        HirStmt::If { cond, then_branch, else_branch } => {
            walk_free_expr(cond, locals, free);
            for st in then_branch.iter().chain(else_branch) { walk_free_stmt(st, locals, free); }
        }
        HirStmt::While { cond, body } => {
            walk_free_expr(cond, locals, free);
            for st in body { walk_free_stmt(st, locals, free); }
        }
        HirStmt::ArrayStore { array, index, value } => {
            walk_free_expr(array, locals, free);
            walk_free_expr(index, locals, free);
            walk_free_expr(value, locals, free);
        }
        HirStmt::PropertySet { object, value, .. } => {
            walk_free_expr(object, locals, free);
            walk_free_expr(value, locals, free);
        }
        HirStmt::Try { try_body, catch_param, catch_body, finally_body } => {
            for st in try_body { walk_free_stmt(st, locals, free); }
            let mut catch_locals = locals.clone();
            if let Some(p) = catch_param { catch_locals.insert(p.clone()); }
            for st in catch_body { walk_free_stmt(st, &mut catch_locals, free); }
            for st in finally_body { walk_free_stmt(st, locals, free); }
        }
    }
}

fn walk_free_expr(e: &HirExpr, locals: &HashSet<String>, free: &mut HashSet<String>) {
    match e {
        HirExpr::Local { name, .. } => {
            if !locals.contains(name) { free.insert(name.clone()); }
        }
        HirExpr::Binary { lhs, rhs, .. } => {
            walk_free_expr(lhs, locals, free);
            walk_free_expr(rhs, locals, free);
        }
        HirExpr::Unary { operand, .. } => walk_free_expr(operand, locals, free),
        HirExpr::Call { args, .. } => {
            for a in args { walk_free_expr(a, locals, free); }
        }
        HirExpr::ArrayLit { elements, .. } => {
            for el in elements { walk_free_expr(el, locals, free); }
        }
        HirExpr::ArrayIndex { array, index, .. } | HirExpr::ArrayStore { array, index } => {
            walk_free_expr(array, locals, free);
            walk_free_expr(index, locals, free);
        }
        HirExpr::ArrayMethod { array, args, .. } => {
            walk_free_expr(array, locals, free);
            for a in args { walk_free_expr(a, locals, free); }
        }
        HirExpr::PropertyGet { object, .. } => walk_free_expr(object, locals, free),
        HirExpr::Ternary { condition, true_expr, false_expr, .. } => {
            walk_free_expr(condition, locals, free);
            walk_free_expr(true_expr, locals, free);
            walk_free_expr(false_expr, locals, free);
        }
        _ => {}
    }
}


/// Kısmi ifade tipi çıkarımı: f64 üreten hesaplı ifadeleri yakalar.
/// Belirsizlikte `None` (çağıran I64 varsayar — eski davranış korunur).
fn infer_expr_ty(e: &HirExpr, locals: &HashMap<String, MirType>, known: &HashMap<String, MirType>) -> Option<MirType> {
    match e {
        HirExpr::IntLit(_) => Some(MirType::I64),
        HirExpr::FloatLit(_) => Some(MirType::F64),
        HirExpr::BoolLit(_) => Some(MirType::Bool),
        HirExpr::NullLit => Some(MirType::Generic),
        HirExpr::StringLit(_) => Some(MirType::Ref(RefKind::String)),
        HirExpr::ArrayLit { .. } => Some(MirType::Ref(RefKind::Array)),
        HirExpr::ObjectLit { .. } => Some(MirType::Ref(RefKind::Object)),
        HirExpr::Local { name, .. } => locals.get(name).copied(),
        HirExpr::Unary { operand, .. } => infer_expr_ty(operand, locals, known),
        HirExpr::Binary { op, lhs, rhs, .. } => {
            use hudhudscript_types::HirBinOp as Op;
            let lt = infer_expr_ty(lhs, locals, known);
            let rt = infer_expr_ty(rhs, locals, known);
            match op {
                // Karşılaştırma/mantıksal → Bool (sayısal)
                Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge
                | Op::And | Op::Or => Some(MirType::Bool),
                // Aritmetik: bir taraf F64 ise sonuç F64 (VM oracle ile aynı)
                _ => {
                    if lt == Some(MirType::F64) || rt == Some(MirType::F64) {
                        Some(MirType::F64)
                    } else if lt == Some(MirType::Ref(RefKind::String))
                        || rt == Some(MirType::Ref(RefKind::String))
                    {
                        Some(MirType::Ref(RefKind::String))
                    } else {
                        Some(MirType::I64)
                    }
                }
            }
        }
        HirExpr::Ternary { true_expr, false_expr, .. } => {
            let t = infer_expr_ty(true_expr, locals, known)?;
            let f = infer_expr_ty(false_expr, locals, known)?;
            if t == MirType::F64 || f == MirType::F64 {
                Some(MirType::F64)
            } else if t == f {
                Some(t)
            } else {
                None
            }
        }
        HirExpr::Call { callee, .. } => match callee.as_str() {
            "Date.to_millis" => Some(MirType::I64),
            "typeof" => Some(MirType::Ref(RefKind::String)),
            c if c.starts_with("Math.") => Some(MirType::F64),
            "print" => Some(MirType::Generic),
            c => known.get(c).copied().filter(|t| *t != MirType::Unit),
        },
        _ => None,
    }
}
