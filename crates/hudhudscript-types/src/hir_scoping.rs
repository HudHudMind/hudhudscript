//! Blok-kapsamı değişken gölgeleme düzeltmesi (M3 ile bulundu, v0.9.36).
//!
//! 15_scope_test.hud tam bunu sınar: `let val = 10; if true { let val = 20 }
//! print(val)` → VM `Val: 10` (blok kapsamı doğru), JIT şeridi iç bloktaki
//! `let`i fonksiyon-düzeyi atamaya indirgediği için `Val: 20` üretiyordu.
//!
//! Çözüm (AST→AST alfa-yeniden adlandırma): dışarıda görünür bir adı
//! gölgeleyen her iç `let`, blok alt ağacında taze ada (`__sh{n}_{ad}`)
//! çevrilir — değer ifadesi bağlanma ÖNCESİNE (dış ada) bakar, sonraki
//! kullanımlar yeni ada. İşlev gövdeleri kendi kapsamıyla özyinelemeli
//! incedenir. Yalnızca gölgeleme OLUŞTUĞUNDA ad değişir; normal kod
//! dokunulmaz.

use hudhudscript_ast::stmt::Stmt;
use hudhudscript_ast::{Expr, Span};
use std::collections::{HashMap, HashSet};

pub(crate) fn resolve_shadows(params: &[String], body: &[Stmt]) -> Vec<Stmt> {
    let mut visible: HashSet<String> = params.iter().cloned().collect();
    let mut counter = 0usize;
    let local = HashMap::new();
    rename_stmts(body, &mut visible, &local, &mut counter)
}

fn fresh(name: &str, counter: &mut usize) -> String {
    *counter += 1;
    format!("__sh{}_{}", counter, name)
}

/// Bir deyim listesini sırayla işler; `renames` o ana kadar bu kapsamda
/// uygulanmış ad değişimlerini taşır (let bağlandıktan sonraki kullanımlar).
fn rename_stmts(
    stmts: &[Stmt],
    visible: &mut HashSet<String>,
    active_local: &HashMap<String, String>,
    c: &mut usize,
) -> Vec<Stmt> {
    // Blok kapsamı: bu listeye özgü görünürlük kopyada tutulur — biterken
    // çağıranın kümesi değişmez (iç let'ler dışarı sızmaz).
    let mut scope: HashSet<String> = visible.clone();
    let mut local: HashMap<String, String> = active_local.clone();
    let mut out = Vec::with_capacity(stmts.len());
    for s in stmts {
        out.push(rename_stmt(s, &mut scope, &mut local, c));
    }
    out
}

fn rename_stmt(
    s: &Stmt,
    scope: &mut HashSet<String>,
    local: &mut HashMap<String, String>,
    c: &mut usize,
) -> Stmt {
    match s {
        Stmt::Let { name, value, span } => {
            let value = rename_expr(value, local);
            if scope.contains(name) {
                // GÖLGELEME: bu noktadan sonra blok içindeki ad taze ada gider
                let nn = fresh(name, c);
                local.insert(name.clone(), nn.clone());
                scope.insert(nn.clone());
                Stmt::Let { name: nn, value, span: *span }
            } else {
                scope.insert(name.clone());
                Stmt::Let { name: name.clone(), value, span: *span }
            }
        }
        Stmt::Assignment { target, value, span } => Stmt::Assignment {
            target: rename_expr(target, local),
            value: rename_expr(value, local),
            span: *span,
        },
        Stmt::If { condition, then_branch, else_branch, span } => Stmt::If {
            condition: rename_expr(condition, local),
            then_branch: Box::new(scoped_block(then_branch, scope, local, c)),
            else_branch: else_branch
                .as_ref()
                .map(|e| Box::new(scoped_block(e, scope, local, c))),
            span: *span,
        },
        Stmt::While { condition, body, span } => Stmt::While {
            condition: rename_expr(condition, local),
            body: Box::new(scoped_block(body, scope, local, c)),
            span: *span,
        },
        Stmt::For { variable, iterable, body, span } => {
            let it = rename_expr(iterable, local);
            let mut inner: HashSet<String> = scope.clone();
            inner.insert(variable.clone());
            let b = scoped_block(body, &inner, local, c);
            Stmt::For { variable: variable.clone(), iterable: it, body: Box::new(b), span: *span }
        }
        Stmt::Block { statements, span } => {
            let mut inner: HashSet<String> = scope.clone();
            let st = rename_stmts(statements, &mut inner, local, c);
            Stmt::Block { statements: st, span: *span }
        }
        Stmt::Expr(e) => Stmt::Expr(rename_expr(e, local)),
        Stmt::Return { value, span } => Stmt::Return {
            value: value.as_ref().map(|v| rename_expr(v, local)),
            span: *span,
        },
        Stmt::Function { name, params, body, is_async, is_generator, type_params, span } => {
            // İşlev kendi kapsamı: gövde yalnızca parametreleri görür
            let mut inner: HashSet<String> = params.iter().cloned().collect();
            let empty = HashMap::new();
            let b = rename_stmts(body, &mut inner, &empty, c);
            Stmt::Function {
                name: name.clone(),
                params: params.clone(),
                body: b,
                is_async: *is_async,
                is_generator: *is_generator,
                type_params: type_params.clone(),
                span: *span,
            }
        }
        other => other.clone(), // Return/Break/Continue/Decl/... — ad taşımaz
    }
}

/// then/else/while gövdesi: tek deyim veya Block — kapsamlı işle.
fn scoped_block(
    s: &Stmt,
    visible: &HashSet<String>,
    local: &HashMap<String, String>,
    c: &mut usize,
) -> Stmt {
    let mut inner = visible.clone();
    match s {
        Stmt::Block { statements, span } => {
            let st = rename_stmts(statements, &mut inner, local, c);
            Stmt::Block { statements: st, span: *span }
        }
        single => {
            let one = vec![single.clone()];
            let st = rename_stmts(&one, &mut inner, local, c);
            st.into_iter().next().unwrap_or_else(|| single.clone())
        }
    }
}

/// Deyimlerdeki ifade ağaçlarında yerel ad değişimlerini uygular.
fn rename_expr(e: &Expr, local: &HashMap<String, String>) -> Expr {
    if local.is_empty() {
        return e.clone();
    }
    match e {
        Expr::Identifier(n, sp) => match local.get(n) {
            Some(nn) => Expr::Identifier(nn.clone(), *sp),
            None => e.clone(),
        },
        Expr::Binary { left, op, right, span } => Expr::Binary {
            left: Box::new(rename_expr(left, local)),
            op: *op,
            right: Box::new(rename_expr(right, local)),
            span: *span,
        },
        Expr::Unary { op, expr, span } => Expr::Unary {
            op: *op,
            expr: Box::new(rename_expr(expr, local)),
            span: *span,
        },
        Expr::Call { callee, args, span } => Expr::Call {
            callee: Box::new(rename_expr(callee, local)),
            args: args.iter().map(|a| rename_expr(a, local)).collect(),
            span: *span,
        },
        Expr::Member { object, property, span } => Expr::Member {
            object: Box::new(rename_expr(object, local)),
            property: property.clone(),
            span: *span,
        },
        Expr::Index { object, index, span } => Expr::Index {
            object: Box::new(rename_expr(object, local)),
            index: Box::new(rename_expr(index, local)),
            span: *span,
        },
        Expr::Array { elements, span } => Expr::Array {
            elements: elements.iter().map(|x| rename_expr(x, local)).collect(),
            span: *span,
        },
        Expr::Object { properties, span } => Expr::Object {
            properties: properties
                .iter()
                .map(|(k, v)| (k.clone(), rename_expr(v, local)))
                .collect(),
            span: *span,
        },
        other => other.clone(),
    }
}

/// Hızlı kontrol: fonksiyon gövdesinde blok içinde `let` var mı?
/// Yoksa değişken gölgelemesi imkânsızdır — AST klonlama tamamen atlanır.
pub(crate) fn has_shadow_candidates(stmts: &[Stmt]) -> bool {
    stmts.iter().any(stmt_has_block_let)
}

fn stmt_has_block_let(s: &Stmt) -> bool {
    match s {
        Stmt::Block { statements, .. } => {
            statements.iter().any(|st| matches!(st, Stmt::Let { .. }))
                || statements.iter().any(stmt_has_block_let)
        }
        Stmt::If { then_branch, else_branch, .. } => {
            stmt_has_block_let(then_branch)
                || else_branch.as_ref().map(|b| stmt_has_block_let(b)).unwrap_or(false)
        }
        Stmt::While { body, .. } => stmt_has_block_let(body),
        Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            stmt_has_block_let(try_block)
                || catch_clause.as_ref().map(|c| stmt_has_block_let(&c.body)).unwrap_or(false)
                || finally_block.as_ref().map(|b| stmt_has_block_let(b)).unwrap_or(false)
        }
        _ => false,
    }
}

