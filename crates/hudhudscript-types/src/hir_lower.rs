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
    // M3 (v0.9.36): Loop Engineering DSL önce saf fonksiyon AST'sine yazılır;
    // desteklenmeyen detayda None → bilinen dürüst hata (CLI VM-fallback).
    // Hızlı yol: Decl bildirimleri (Loop DSL / SOP) yoksa klonlama yapmadan doğrudan stmts kullan.
    let has_decls = stmts.iter().any(|s| matches!(s, Stmt::Decl(_)));
    let rewritten;
    let stmts: &[Stmt] = if has_decls {
        rewritten = crate::hir_loop::rewrite_loops(stmts)
            .ok_or_else(|| reject("loop engineering", "unsupported loop construct"))?;
        &rewritten
    } else {
        crate::hir_sop::set_event_names(vec![]);
        stmts
    };
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
                Stmt::Class(_)
                | Stmt::Decl(hudhudscript_ast::Decl::Subject { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Role { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Relation { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Council { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Compose { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Event { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Effect { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Ability { .. }) => {}
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
                Stmt::Class(_)
                | Stmt::Decl(hudhudscript_ast::Decl::Subject { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Role { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Relation { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Council { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Compose { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Event { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Effect { .. })
                | Stmt::Decl(hudhudscript_ast::Decl::Ability { .. }) => {}
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
    // Blok-gölgeleme alfa-yeniden adlandırması (v0.9.36): yalnızca gölgeleme
    // adayı (iç blokta let) varsa çalışır — normal kod dokunulmaz ve klonlanmaz.
    let shadowed;
    let body: &[Stmt] = if crate::hir_scoping::has_shadow_candidates(body) {
        shadowed = crate::hir_scoping::resolve_shadows(params, body);
        &shadowed
    } else {
        body
    };
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
                            if !crate::hir_ops::body_assigns_var(body, obj_name)
                                && !crate::hir_ops::body_may_mutate_heap(body)
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
