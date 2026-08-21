//! G12 — f-loop analiz yarısı (emit yarısı `floop.rs`).
//!
//! `analyze`: while-gövdesinde float-kanıtlı, kaçışsız yerelleri kanıtlar;
//! kanıtlanamazsa None döner (döngü eski yoldan derlenir — doğruluk asla
//! riske girmez).

use super::floop::{FloopPlan, FE};
use super::*;
use crate::compiler::expr::ExprType;
use hudhudscript_ast::UnaryOp;

/// f-ifade sınıflandırması — emit tarafının kabul ettiği şekillerle birebir
/// aynı küme. `cands` içindeki her isim Number-kanıtlı sayılır.
fn f_class(expr: &Expr, cands: &HashSet<String>, math_shadowed: bool) -> FE {
    match expr {
        Expr::Identifier(name, _) => {
            if cands.contains(name.as_str()) {
                FE::Float
            } else {
                FE::No
            }
        }
        Expr::Literal(Literal::Number(_, is_float), _) => {
            if *is_float {
                FE::Float
            } else {
                FE::Int
            }
        }
        Expr::Literal(Literal::Int(_), _) => FE::Int,
        Expr::Binary {
            left, op, right, ..
        } => {
            let l = f_class(left, cands, math_shadowed);
            let r = f_class(right, cands, math_shadowed);
            match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul => combine(l, r),
                // Int/Int bölme truncation'dır (IntDiv); f-domain f64 böler —
                // sessiz anlam değişimine izin yok.
                BinaryOp::Div => match (l, r) {
                    (FE::No, _) | (_, FE::No) => FE::No,
                    (FE::Int, FE::Int) => FE::No,
                    _ => FE::Float,
                },
                _ => FE::No,
            }
        }
        Expr::Unary {
            op: UnaryOp::Neg | UnaryOp::Plus,
            expr: inner,
            ..
        } => f_class(inner, cands, math_shadowed),
        Expr::Call { args, .. } => {
            if !math_shadowed && super::floop::math_intrinsic(expr).is_some() {
                return match f_class(&args[0], cands, math_shadowed) {
                    FE::No => FE::No,
                    _ => FE::Float,
                };
            }
            FE::No
        }
        _ => FE::No,
    }
}

fn combine(l: FE, r: FE) -> FE {
    match (l, r) {
        (FE::No, _) | (_, FE::No) => FE::No,
        (FE::Float, _) | (_, FE::Float) => FE::Float,
        _ => FE::Int,
    }
}

/// "Güvenli ifade": çağrı/index/property/closure/await vs. İÇERMEZ — yani
/// ne gizli kaçış yaratır ne de unwinding (epilogu atlatabilecek istisna)
/// mümkündür. Karşılaştırmalar ve And/Or dahildir (IntCmp/JumpIfFalse hata
/// üretmez).
fn expr_safe(expr: &Expr) -> bool {
    match expr {
        Expr::Literal(..) | Expr::Identifier(..) => true,
        Expr::Binary { left, right, .. } => expr_safe(left) && expr_safe(right),
        Expr::Unary { expr: inner, .. } => expr_safe(inner),
        Expr::Ternary {
            condition,
            true_expr,
            false_expr,
            ..
        } => expr_safe(condition) && expr_safe(true_expr) && expr_safe(false_expr),
        _ => false,
    }
}

/// İfadedeki TÜM identifier'ları toplar (derin yürüyüş). Kapamayla
/// karşılaşırsa `poison` kurulur — kapama gövdesi ayrı kapsamda derlenir;
/// içindeki olası aday okumalarını kanıtlayamayız, çağıran döngüyü
/// reddetmek zorundadır.
fn collect_idents(expr: &Expr, out: &mut Vec<String>, poison: &mut bool) {
    match expr {
        Expr::Literal(..) | Expr::This(_) => {}
        Expr::Identifier(name, _) => out.push(name.clone()),
        Expr::Binary { left, right, .. } => {
            collect_idents(left, out, poison);
            collect_idents(right, out, poison);
        }
        Expr::Unary { expr: inner, .. } => collect_idents(inner, out, poison),
        Expr::Call { callee, args, .. } => {
            collect_idents(callee, out, poison);
            for a in args {
                collect_idents(a, out, poison);
            }
        }
        Expr::Perform { action, .. } => collect_idents(action, out, poison),
        Expr::Recall { query, .. } => collect_idents(query, out, poison),
        Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
            collect_idents(object, out, poison)
        }
        Expr::Index { object, index, .. } => {
            collect_idents(object, out, poison);
            collect_idents(index, out, poison);
        }
        Expr::Ternary {
            condition,
            true_expr,
            false_expr,
            ..
        } => {
            collect_idents(condition, out, poison);
            collect_idents(true_expr, out, poison);
            collect_idents(false_expr, out, poison);
        }
        Expr::Array { elements, .. } => {
            for e in elements {
                collect_idents(e, out, poison);
            }
        }
        Expr::Object { properties, .. } => {
            for (_, v) in properties {
                collect_idents(v, out, poison);
            }
        }
        Expr::TemplateString { parts, .. } => {
            for p in parts {
                if let hudhudscript_ast::TemplateStringPart::Interpolation(e) = p {
                    collect_idents(e, out, poison);
                }
            }
        }
        Expr::ArrowFunction { .. } => *poison = true,
        Expr::Await { expr: inner, .. } | Expr::Spread { expr: inner, .. } => {
            collect_idents(inner, out, poison)
        }
        Expr::New { args, .. } => {
            for a in args {
                collect_idents(a, out, poison);
            }
        }
        Expr::Yield { value, .. } => {
            if let Some(v) = value {
                collect_idents(v, out, poison);
            }
        }
        Expr::Spawn { args, .. } => {
            for a in args {
                collect_idents(a, out, poison);
            }
        }
        Expr::ViewAs { instance, .. } => collect_idents(instance, out, poison),
    }
}

/// Statement ağacındaki tüm identifier'ları toplar (seed adaylığı için).
/// Kapama gövdelerine GİRMEZ (ayrı kapsam); kapama içeren döngü validate
/// aşamasında zaten reddedilir.
fn collect_stmts_idents(stmt: &Stmt, out: &mut Vec<String>) {
    fn w(e: &Expr, out: &mut Vec<String>) {
        collect_idents(e, out, &mut false);
    }
    match stmt {
        Stmt::Let { value, .. } | Stmt::Const { value, .. } => w(value, out),
        Stmt::Assignment { target, value, .. } => {
            w(target, out);
            w(value, out);
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            w(condition, out);
            collect_stmts_idents(then_branch, out);
            if let Some(e) = else_branch {
                collect_stmts_idents(e, out);
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            w(condition, out);
            collect_stmts_idents(body, out);
        }
        Stmt::For { iterable, body, .. } => {
            w(iterable, out);
            collect_stmts_idents(body, out);
        }
        Stmt::ForCStyle {
            init,
            condition,
            update,
            body,
            ..
        } => {
            if let Some(i) = init {
                collect_stmts_idents(i, out);
            }
            if let Some(c) = condition {
                w(c, out);
            }
            if let Some(u) = update {
                collect_stmts_idents(u, out);
            }
            collect_stmts_idents(body, out);
        }
        Stmt::ForRange {
            start,
            stop,
            step,
            body,
            ..
        } => {
            w(start, out);
            w(stop, out);
            if let Some(s) = step {
                w(s, out);
            }
            collect_stmts_idents(body, out);
        }
        Stmt::Block { statements, .. } => {
            for s in statements {
                collect_stmts_idents(s, out);
            }
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value {
                w(v, out);
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => w(e, out),
        Stmt::Switch {
            value,
            cases,
            default,
            ..
        } => {
            w(value, out);
            for c in cases {
                w(&c.value, out);
                for s in &c.body {
                    collect_stmts_idents(s, out);
                }
            }
            if let Some(d) = default {
                for s in d {
                    collect_stmts_idents(s, out);
                }
            }
        }
        Stmt::Try {
            try_block,
            catch_clause,
            finally_block,
            ..
        } => {
            collect_stmts_idents(try_block, out);
            if let Some(c) = catch_clause {
                collect_stmts_idents(&c.body, out);
            }
            if let Some(f) = finally_block {
                collect_stmts_idents(f, out);
            }
        }
        Stmt::Export { item, .. } => collect_stmts_idents(item, out),
        // Kapama/decl/diğerleri: ayrı kapsam ya da validate'de red — toplama.
        _ => {}
    }
}

/// Gövdedeki `let` isimlerini gez (shadow yasağı için).
fn collect_let_names(stmt: &Stmt, out: &mut Vec<String>) {
    match stmt {
        Stmt::Let { name, .. } => out.push(name.clone()),
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_let_names(then_branch, out);
            if let Some(e) = else_branch {
                collect_let_names(e, out);
            }
        }
        Stmt::Block { statements, .. } => {
            for s in statements {
                collect_let_names(s, out);
            }
        }
        _ => {}
    }
}

/// Gövde-içi `let` tohumlaması: init'i mevcut aday kümesiyle Float-kanıtlı
/// olan, mevcut bir yerelle çakışmayan let'lerin isimlerini toplar.
fn seed_internals(
    stmt: &Stmt,
    cands: &HashSet<String>,
    math_shadowed: bool,
    banned: &HashSet<String>,
    out: &mut Vec<String>,
) {
    match stmt {
        Stmt::Let { name, value, .. } => {
            let cls = f_class(value, cands, math_shadowed);
            if !cands.contains(name.as_str()) && !banned.contains(name.as_str()) && cls == FE::Float
            {
                out.push(name.clone());
            }
        }
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            seed_internals(then_branch, cands, math_shadowed, banned, out);
            if let Some(e) = else_branch {
                seed_internals(e, cands, math_shadowed, banned, out);
            }
        }
        Stmt::Block { statements, .. } => {
            for s in statements {
                seed_internals(s, cands, math_shadowed, banned, out);
            }
        }
        _ => {}
    }
}

/// İfade `cands`'tan isim içeriyorsa o adayları diskalifiye listesine
/// ekle; poison (kapama) görülürse tüm döngüyü reddettir.
fn disqualify_occurrences(
    expr: &Expr,
    cands: &HashSet<String>,
    remove: &mut Vec<String>,
    bail: &mut bool,
) {
    let mut ids = Vec::new();
    let mut poison = false;
    collect_idents(expr, &mut ids, &mut poison);
    if poison {
        *bail = true;
        return;
    }
    for id in ids {
        if cands.contains(&id) {
            remove.push(id);
        }
    }
}

/// Gövde doğrulaması: adayların HER kullanımı f-domain'de mi? Güvensiz
/// yapı varsa `bail` (tüm pass iptal), aksi hâlde ihlal eden adaylar
/// `remove`'a (fixpoint ile diskalifiye).
fn validate_stmt(
    stmt: &Stmt,
    cands: &HashSet<String>,
    math_shadowed: bool,
    bail: &mut bool,
    remove: &mut Vec<String>,
) {
    if *bail {
        return;
    }
    match stmt {
        Stmt::Let { name, value, .. } => {
            let cls = f_class(value, cands, math_shadowed);
            if cands.contains(name.as_str()) {
                if cls != FE::Float {
                    remove.push(name.clone());
                }
            } else {
                disqualify_occurrences(value, cands, remove, bail);
            }
        }
        Stmt::Const { value, .. } => disqualify_occurrences(value, cands, remove, bail),
        Stmt::Assignment { target, value, .. } => {
            if let Expr::Identifier(t, _) = target {
                if cands.contains(t.as_str()) {
                    if f_class(value, cands, math_shadowed) != FE::Float {
                        remove.push(t.clone());
                    }
                } else {
                    disqualify_occurrences(value, cands, remove, bail);
                }
            } else {
                // Member/Index hedefli atama: SetProperty/IndexAssign hem
                // istisna üretebilir hem aday register'ına dokunabilir.
                *bail = true;
            }
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            if !expr_safe(condition) {
                *bail = true;
                return;
            }
            disqualify_occurrences(condition, cands, remove, bail);
            validate_stmt(then_branch, cands, math_shadowed, bail, remove);
            if let Some(e) = else_branch {
                validate_stmt(e, cands, math_shadowed, bail, remove);
            }
        }
        Stmt::Expr(e) => {
            if !expr_safe(e) {
                *bail = true;
                return;
            }
            disqualify_occurrences(e, cands, remove, bail);
        }
        Stmt::Return { value, .. } => {
            if let Some(v) = value {
                if !expr_safe(v) {
                    *bail = true;
                    return;
                }
                disqualify_occurrences(v, cands, remove, bail);
            }
        }
        Stmt::Block { statements, .. } => {
            for s in statements {
                validate_stmt(s, cands, math_shadowed, bail, remove);
            }
        }
        // İç içe döngü, try, switch, match, break/continue, throw, kapama,
        // decl, import/export, destructure, var — v1 kapsamı dışı: red.
        _ => *bail = true,
    }
}

/// Ana analiz: başarılıysa FloopPlan, değilse None (eski yol).
pub(super) fn analyze(
    target: &impl CompileTarget,
    condition: &Expr,
    body: &Stmt,
) -> Option<FloopPlan> {
    // Koşul her iterasyonda register'dan okunur; güvenli olmalı ve aday
    // içermemeli (içeren isimler diskalifiye).
    if !expr_safe(condition) {
        return None;
    }
    let mut cond_ids = Vec::new();
    let mut poison = false;
    collect_idents(condition, &mut cond_ids, &mut poison);
    if poison {
        return None;
    }
    let cond_set: HashSet<String> = cond_ids.into_iter().collect();

    let math_shadowed = target.ct_local_reg("Math").is_some() || target.ct_math_global_written();

    // Shadow yasağı: gövdede `let x` ile yeniden bildirilen isim mevcut bir
    // yerelle çakışıyorsa aday olamaz (hangi `x`'in f-slot'ta yaşadığı
    // belirsizleşir).
    let mut banned: HashSet<String> = HashSet::new();
    let mut let_names = Vec::new();
    collect_let_names(body, &mut let_names);
    for n in let_names {
        if target.ct_local_reg(&n).is_some() {
            banned.insert(n);
        }
    }

    // Ön-tohum: gövdede geçen, Number-kanıtlı, kaçışsız mevcut yereller.
    let mut body_ids = Vec::new();
    collect_stmts_idents(body, &mut body_ids);
    let mut order: Vec<String> = Vec::new();
    let mut cands: HashSet<String> = HashSet::new();
    for name in body_ids {
        if cands.contains(&name) || cond_set.contains(&name) || banned.contains(&name) {
            continue;
        }
        if target.ct_local_reg(&name).is_some()
            && target.ct_local_type(&name) == ExprType::Number
            && !target.ct_is_const_local(&name)
            && !target.ct_floop_captured(&name)
            && !(target.ct_is_top_level() && target.ct_is_shared_top_level(&name))
        {
            cands.insert(name.clone());
            order.push(name);
        }
    }

    // FAZ 1 — BÜYÜME: gövde-içi let tohumlarını kapanışa kadar ekle.
    // (`let k1t = omega; let k2t = omega + hdt*k1o` zinciri seviye seviye
    // aday olur.) Doğrulamadan ÖNCE kapanışa ulaşmak şart: erken doğrulama,
    // henüz aday olmamış iç let'lerin RHS'lerindeki adayları diskalifiye
    // eder ve kümeyi boşaltır.
    loop {
        let mut seeds = Vec::new();
        seed_internals(body, &cands, math_shadowed, &banned, &mut seeds);
        let mut changed = false;
        for name in seeds {
            if !cands.contains(&name) {
                cands.insert(name.clone());
                order.push(name);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // FAZ 2 — KÜÇÜLME: ihlal edenleri düşür, fixpoint'e kadar. `f_class`
    // cands'te monotondur (küme küçülünce Float→No olabilir, tersi olamaz),
    // dolayısıyla shrink sonrası yeniden büyüme gerekmez; kaskad (`k1t =
    // omega` → omega düşünce k1t de düşer) doğrulama turlarıyla yayılır.
    loop {
        let mut bail = false;
        let mut remove: Vec<String> = Vec::new();
        validate_stmt(body, &cands, math_shadowed, &mut bail, &mut remove);
        if bail {
            return None;
        }
        let mut changed = false;
        for r in remove {
            if cands.remove(&r) {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    if cands.is_empty() {
        return None;
    }
    let order: Vec<String> = order.into_iter().filter(|n| cands.contains(n)).collect();
    // 64 slot: adaylar + en az 16 geçici.
    if order.len() > 48 {
        return None;
    }

    let mut pre = Vec::new();
    let mut slots = Vec::new();
    for (i, name) in order.iter().enumerate() {
        let fslot = i as u8;
        slots.push((name.clone(), fslot));
        if let Some(reg) = target.ct_local_reg(name) {
            pre.push((name.clone(), fslot, reg));
        }
    }
    Some(FloopPlan::new(pre, slots, order.len() as u8))
}
