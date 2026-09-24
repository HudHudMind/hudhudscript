//! AST → Typed HIR lowering (JIT_AOT_ARCHITECTURE.md §4.3, AŞAMA-0.5).
//!
//! Scope (widened step by step; unsupported constructs are REJECTED with
//! clean errors — semantics are never guessed):
//! - `function name(params) { body }` declarations (sync, non-generator)
//! - statements: Return / Expr / Let / Assign
//! - expressions: Int/Float/Bool/Null/String literals, Identifier,
//!   Binary (arithmetic + comparisons), Call with identifier callee,
//!   Unary Neg/Not
//! - inference-lite typing: literals carry their obvious type; params
//!   and identifiers are `Any` until a symbol table lands; a binary
//!   arith op of two Numbers is Number, otherwise Any.

use std::fmt;

use hudhudscript_ast::{Expr, Stmt, UnaryOp};

use crate::hir::{HirBinOp, HirExpr, HirFunction, HirModule, HirParam, HirStmt, HirUnOp};
use crate::types::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirLowerError {
    Unsupported { item: String, reason: String },
}

impl fmt::Display for HirLowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HirLowerError::Unsupported { item, reason } => {
                write!(f, "cannot lower {item} to HIR: {reason}")
            }
        }
    }
}

impl std::error::Error for HirLowerError {}

/// Lower a parsed program into a HirModule. Top-level statements are
/// wrapped into a synthetic `_hudhud_init()` function (scripting
/// language semantics — JavaScript/Python gibi main() zorunluluğu YOK).
/// Fonksiyon tanımları normal şekilde module.functions'a gider.
pub fn lower_module_with_init(stmts: &[Stmt]) -> Result<HirModule, HirLowerError> {
    let class_table = crate::hir_class::ClassTable::from_stmts(stmts);
    let mut known = std::collections::HashSet::new();
    for s in stmts {
        if let Stmt::Function { name, .. } = s {
            known.insert(name.clone());
        }
    }
    for (m, _) in &class_table.methods {
        known.insert(m.clone());
    }
    crate::hir_closure::init_closure_table(known.clone());

    crate::hir_class::with_class_table(class_table.clone(), || {
        let mut module = HirModule::default();
        class_table.lower_methods(stmts, &mut module)?;
        let mut init_body: Vec<Stmt> = Vec::new();

        for stmt in stmts {
            match stmt {
                Stmt::Class(_) | Stmt::Decl(hudhudscript_ast::Decl::Subject { .. }) => {}
                Stmt::Function { name, params, body, is_async, is_generator, .. } => {
                    if *is_async {
                        return Err(reject("function", &format!("`{name}` is async (async lane arrives later)")));
                    }
                    if *is_generator {
                        return Err(reject("function", &format!("`{name}` is a generator (generator lane arrives later)")));
                    }
                    let f = lower_function(name, params, body)?;
                    module.functions.insert(name.clone(), f);
                }
                other => {
                    init_body.push(other.clone());
                }
            }
        }

        if !init_body.is_empty() {
            let init = lower_function("_hudhud_init", &[], &init_body)?;
            module.functions.insert("_hudhud_init".to_string(), init);
        }

        for f in crate::hir_closure::take_lifted_functions() {
            module.functions.insert(f.name.clone(), f);
        }

        Ok(module)
    })
}

/// Lower a parsed program (top-level statements) into a HirModule.
pub fn lower_module(stmts: &[Stmt]) -> Result<HirModule, HirLowerError> {
    let class_table = crate::hir_class::ClassTable::from_stmts(stmts);
    crate::hir_class::with_class_table(class_table.clone(), || {
        let mut module = HirModule::default();
        class_table.lower_methods(stmts, &mut module)?;
        for stmt in stmts {
            match stmt {
                Stmt::Class(_) | Stmt::Decl(hudhudscript_ast::Decl::Subject { .. }) => {}
                Stmt::Function { name, params, body, is_async, is_generator, .. } => {
                    if *is_async {
                        return Err(reject("function", &format!("`{name}` is async (async lane arrives later)")));
                    }
                    if *is_generator {
                        return Err(reject("function", &format!("`{name}` is a generator (generator lane arrives later)")));
                    }
                    let f = lower_function(name, params, body)?;
                    module.functions.insert(name.clone(), f);
                }
                other => return Err(crate::hir_ops::unsupported_stmt(other)),
            }
        }
        Ok(module)
    })
}

use crate::hir_ops::has_return_value;

/// Lower one function declaration.
///
/// Return-type inference-lite: a body with at least one `return expr`
/// is `Number` (the numeric native lane); bare/absent returns are `Any`
/// (unit lane). Precise per-type inference arrives with the symbol
/// table; this rule is total and never guesses per-expression.
pub fn lower_function(name: &str, params: &[String], body: &[Stmt]) -> Result<HirFunction, HirLowerError> {
    let returns_value = has_return_value(body);
    Ok(HirFunction {
        name: name.to_string(),
        params: params.iter().map(|p| HirParam { name: p.clone(), ty: Type::Any }).collect(),
        return_type: if returns_value { Type::Number } else { Type::Any },
        body: lower_stmts(body)?,
    })
}

fn lower_stmts(body: &[Stmt]) -> Result<Vec<HirStmt>, HirLowerError> {
    let mut out = Vec::new();
    for stmt in body {
        out.extend(lower_stmt(stmt)?);
    }
    Ok(out)
}

/// Lower a statement; constructs that desugar (for → let+while) return
/// multiple HirStmts.
pub(crate) fn lower_stmt(stmt: &Stmt) -> Result<Vec<HirStmt>, HirLowerError> {
    match stmt {
        Stmt::Return { value, .. } => Ok(vec![HirStmt::Return(value.as_ref().map(lower_expr).transpose()?)]),
        Stmt::Expr(expr) => {
            // i++ / i-- ATAMA ifadesidir: i = i + 1 (eski-değer semantiği
            // yalnızca expression bağlamında gerekir — o bağlam reddedilir)
            if let Expr::Unary { op, expr: inner, .. } = expr {
                if !matches!(op, UnaryOp::PostIncrement | UnaryOp::PostDecrement) {
                    return Ok(vec![HirStmt::Expr(lower_expr(expr)?)]);
                }
                if let Expr::Identifier(name, _) = inner.as_ref() {
                    let hir_op = match op {
                        UnaryOp::PostIncrement => HirBinOp::Add,
                        _ => HirBinOp::Sub,
                    };
                    let val = HirExpr::Binary {
                        op: hir_op,
                        lhs: Box::new(HirExpr::Local { name: name.clone(), ty: Type::Any }),
                        rhs: Box::new(HirExpr::IntLit(1)),
                        ty: Type::Any,
                    };
                    return Ok(vec![HirStmt::Assign { name: name.clone(), value: val }]);
                }
            }
            Ok(vec![HirStmt::Expr(lower_expr(expr)?)])
        }
        Stmt::Let { name, value, span } => {
            if let Some(desugared) = crate::hir_array_combinators::desugar_array_combinator(name, value, true, *span) {
                return lower_stmts(&desugared);
            }
            let v = lower_expr(value)?;
            let ty = v.ty();
            Ok(vec![HirStmt::Let { name: name.clone(), ty, value: v }])
        }
        Stmt::Assignment { target, value, span } => {
            if let Expr::Identifier(name, _) = target {
                if let Some(desugared) = crate::hir_array_combinators::desugar_array_combinator(name, value, false, *span) {
                    return lower_stmts(&desugared);
                }
            }
            let v = lower_expr(value)?;
            match target {
                Expr::Identifier(name, _) => {
                    Ok(vec![HirStmt::Assign { name: name.clone(), value: v }])
                }
                Expr::Index { object, index, .. } => {
                    let arr = lower_expr(object)?;
                    let idx = lower_expr(index)?;
                    Ok(vec![HirStmt::ArrayStore { array: arr, index: idx, value: v }])
                }
                Expr::Member { object, property, .. } => {
                    let obj = lower_expr(object)?;
                    Ok(vec![HirStmt::PropertySet { object: obj, name: property.clone(), value: v }])
                }
                other => return Err(reject("assignment", &format!("unsupported target {:?}", std::mem::discriminant(other)))),
            }
        }
        Stmt::Break { .. } => Ok(vec![HirStmt::Break]),
        Stmt::Continue { .. } => Ok(vec![HirStmt::Continue]),
        Stmt::VarDecl(v) => {
            if let Some(init) = &v.initializer {
                if let Some(desugared) = crate::hir_array_combinators::desugar_array_combinator(&v.name, init, true, v.span) {
                    return lower_stmts(&desugared);
                }
            }
            let init = v.initializer.as_ref()
                .map(lower_expr)
                .transpose()?
                .unwrap_or(HirExpr::NullLit);
            let ty = init.ty();
            Ok(vec![HirStmt::Let { name: v.name.clone(), ty, value: init }])
        }
        Stmt::If { condition, then_branch, else_branch, .. } => {
            let c = lower_expr(condition)?;
            let t = lower_block_stmt(then_branch)?;
            let e = match else_branch {
                Some(b) => lower_block_stmt(b)?,
                None => Vec::new(),
            };
            Ok(vec![HirStmt::If { cond: c, then_branch: t, else_branch: e }])
        }
        Stmt::While { condition, body, .. } => {
            if let Expr::Binary { left, op, right, span } = condition {
                if let Expr::Member { object, property, span: pspan } = right.as_ref() {
                    if property == "length" {
                        if let Expr::Identifier(obj_name, _) = object.as_ref() {
                            if !body_assigns_var(body, obj_name)
                                && !body_may_mutate_heap(body)
                            {
                                let hoisted_name = format!("__hoisted_len_{obj_name}");
                                let hoisted_let = Stmt::Let {
                                    name: hoisted_name.clone(),
                                    value: *right.clone(),
                                    span: *pspan,
                                };
                                let new_cond = Expr::Binary {
                                    left: left.clone(),
                                    op: *op,
                                    right: Box::new(Expr::Identifier(hoisted_name, *span)),
                                    span: *span,
                                };
                                let mut out = lower_stmt(&hoisted_let)?;
                                let c = lower_expr(&new_cond)?;
                                let b = lower_block_stmt(body)?;
                                out.push(HirStmt::While { cond: c, body: b });
                                return Ok(out);
                            }
                        }
                    }
                }
            }
            let c = lower_expr(condition)?;
            let b = lower_block_stmt(body)?;
            Ok(vec![HirStmt::While { cond: c, body: b }])
        }
        Stmt::Throw { value, .. } => {
            let v = lower_expr(value)?;
            Ok(vec![HirStmt::Throw(v)])
        }
        Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            let try_body = lower_block_stmt(try_block)?;
            let (catch_param, catch_body) = match catch_clause {
                Some(c) => (Some(c.param.clone()), lower_block_stmt(&c.body)?),
                None => (None, Vec::new()),
            };
            let finally_body = match finally_block {
                Some(b) => lower_block_stmt(b)?,
                None => Vec::new(),
            };
            Ok(vec![HirStmt::Try {
                try_body,
                catch_param,
                catch_body,
                finally_body,
            }])
        }
        Stmt::For { variable, iterable, body, .. } => {
            crate::hir_desugar::desugar_for_in(variable, iterable, body)
        }
        Stmt::ForCStyle { init, condition, update, body, .. } => {
            crate::hir_desugar::desugar_for_c_style(init, condition, update, body)
        }
        Stmt::ForRange { .. } => Err(reject("for range", "range for loops arrive with the for-in lane")),
        Stmt::Spawn { subject_name, args, span } => {
            let expr = Expr::Spawn {
                subject_name: subject_name.clone(),
                args: args.clone(),
                span: *span,
            };
            Ok(vec![HirStmt::Expr(lower_expr(&expr)?)])
        }
        other => Err(crate::hir_ops::unsupported_stmt(other)),
    }
}

/// Gövdede heap mutasyonu YAPABİLECEK bir işlem var mı?
/// Çağrılar (push/pop/foo(arr)), indeks store'lar (a[i]=v) ve property
/// store'lar (o.p=v) bayat `.length` riski taşır; skaler atamalar (i=i+1)
/// taşımaz. Güvenli tarafta kal: şüphede `true` (hoist etme).
fn body_may_mutate_heap(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(e) => expr_has_call(e),
        Stmt::Assignment { target, value, .. } => {
            // a[i] = v  veya  o.p = v  → heap store
            if matches!(target, Expr::Index { .. } | Expr::Member { .. }) {
                return true;
            }
            expr_has_call(value)
        }
        Stmt::Let { value, .. } | Stmt::VarDecl(hudhudscript_ast::VarDecl { initializer: Some(value), .. }) => expr_has_call(value),
        Stmt::Return { value: Some(v), .. } => expr_has_call(v),
        Stmt::If { condition, then_branch, else_branch, .. } => {
            expr_has_call(condition)
                || body_may_mutate_heap(then_branch)
                || else_branch.as_ref().map(|b| body_may_mutate_heap(b)).unwrap_or(false)
        }
        Stmt::While { condition, body, .. } => {
            expr_has_call(condition) || body_may_mutate_heap(body)
        }
        Stmt::Block { statements, .. } => statements.iter().any(body_may_mutate_heap),
        Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            body_may_mutate_heap(try_block)
                || catch_clause.as_ref().map(|c| body_may_mutate_heap(&c.body)).unwrap_or(false)
                || finally_block.as_ref().map(|b| body_may_mutate_heap(b)).unwrap_or(false)
        }
        _ => false,
    }
}

/// İfade ağacında çağrı var mı (okuma/yazma ayırt edilemez → var say)?
fn expr_has_call(e: &Expr) -> bool {
    match e {
        Expr::Call { callee, args, .. } => {
            // Date.to_millis / Math.* saf ve heap'e dokunmaz; diğer tüm
            // çağrılar (metot dâhil) mutasyon yapabilir.
            let pure = matches!(callee.as_ref(),
                Expr::Identifier(n, _) if n == "Date" || n == "Math");
            if !pure {
                return true;
            }
            args.iter().any(expr_has_call)
        }
        Expr::Binary { left, right, .. } => expr_has_call(left) || expr_has_call(right),
        Expr::Unary { expr, .. } => expr_has_call(expr),
        Expr::Index { object, index, .. } => expr_has_call(object) || expr_has_call(index),
        Expr::Member { object, .. } => expr_has_call(object),
        Expr::Ternary { condition, true_expr, false_expr, .. } => {
            expr_has_call(condition) || expr_has_call(true_expr) || expr_has_call(false_expr)
        }
        Expr::Array { elements, .. } => elements.iter().any(expr_has_call),
        Expr::Object { properties, .. } => properties.iter().any(|(_, v)| expr_has_call(v)),
        _ => false,
    }
}

fn body_assigns_var(stmt: &Stmt, var: &str) -> bool {
    match stmt {
        Stmt::Assignment { target, .. } => {
            matches!(target, Expr::Identifier(n, _) if n == var)
        }
        Stmt::Let { name, .. } => name == var,
        Stmt::VarDecl(v) => v.name == var,
        Stmt::Block { statements, .. } => statements.iter().any(|s| body_assigns_var(s, var)),
        Stmt::If { then_branch, else_branch, .. } => {
            body_assigns_var(then_branch, var)
                || else_branch.as_ref().map(|b| body_assigns_var(b, var)).unwrap_or(false)
        }
        Stmt::While { body, .. } => body_assigns_var(body, var),
        Stmt::Try { try_block, catch_clause, finally_block, .. } => {
            body_assigns_var(try_block, var)
                || catch_clause.as_ref().map(|c| body_assigns_var(&c.body, var)).unwrap_or(false)
                || finally_block.as_ref().map(|b| body_assigns_var(b, var)).unwrap_or(false)
        }
        _ => false,
    }
}

pub(crate) use crate::hir_expr_lower::lower_expr;

pub(crate) fn reject(item: &str, reason: &str) -> HirLowerError {
    HirLowerError::Unsupported { item: item.to_string(), reason: reason.to_string() }
}

/// Bir `Box<Stmt>`'yi Vec<HirStmt>'a indirger (Block → içindekiler, diğer → desugared).
pub(crate) fn lower_block_stmt(stmt: &Stmt) -> Result<Vec<HirStmt>, HirLowerError> {
    match stmt {
        Stmt::Block { statements, .. } => lower_stmts(statements),
        other => lower_stmt(other),
    }
}
