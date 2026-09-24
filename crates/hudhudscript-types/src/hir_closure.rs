//! Closure conversion and lambda lifting for typed HIR lowering.

use std::cell::RefCell;
use std::collections::HashSet;
use hudhudscript_ast::{ArrowFunctionBody, Expr, Span, Stmt};

use crate::hir::{HirExpr, HirFunction};
use crate::hir_lower::{lower_function, HirLowerError};
use crate::types::Type;

#[derive(Debug, Clone, Default)]
pub struct ClosureTable {
    pub closures: Vec<LiftedClosure>,
    pub functions: Vec<HirFunction>,
}

#[derive(Debug, Clone)]
pub struct LiftedClosure {
    pub id: usize,
    pub fn_name: String,
    pub captured_vars: Vec<String>,
}

thread_local! {
    static CURRENT_CLOSURES: RefCell<Option<ClosureTable>> = const { RefCell::new(None) };
    static CLOSURE_COUNTER: RefCell<usize> = const { RefCell::new(1) };
    static KNOWN_FUNCTIONS: RefCell<Option<HashSet<String>>> = const { RefCell::new(None) };
}

pub fn init_closure_table(known: HashSet<String>) {
    CURRENT_CLOSURES.with(|cell| {
        *cell.borrow_mut() = Some(ClosureTable::default());
    });
    KNOWN_FUNCTIONS.with(|cell| {
        *cell.borrow_mut() = Some(known);
    });
}

pub fn is_known_function(name: &str) -> bool {
    if matches!(
        name,
        "print" | "typeof" | "assert" | "panic" | "Date.to_millis"
    ) || name.starts_with("Math.") {
        return true;
    }
    KNOWN_FUNCTIONS.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|set| set.contains(name))
            .unwrap_or(false)
    })
}

pub fn take_lifted_functions() -> Vec<HirFunction> {
    CURRENT_CLOSURES.with(|cell| {
        if let Some(table) = cell.borrow_mut().as_mut() {
            std::mem::take(&mut table.functions)
        } else {
            Vec::new()
        }
    })
}

pub(crate) fn has_active_closures() -> bool {
    CURRENT_CLOSURES.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|t| !t.closures.is_empty())
            .unwrap_or(false)
    })
}

pub(crate) fn dispatch_closure_call(
    var_name: &str,
    args: Vec<HirExpr>,
) -> Option<HirExpr> {
    CURRENT_CLOSURES.with(|cell| {
        let borrow = cell.borrow();
        let table = borrow.as_ref()?;
        if table.closures.is_empty() {
            return None;
        }

        let mut call_args = vec![HirExpr::Local {
            name: var_name.to_string(),
            ty: Type::Any,
        }];
        call_args.extend(args);

        if table.closures.len() == 1 {
            return Some(HirExpr::Call {
                callee: table.closures[0].fn_name.clone(),
                args: call_args,
                ty: Type::Any,
            });
        }

        let tid = HirExpr::PropertyGet {
            object: Box::new(HirExpr::Local {
                name: var_name.to_string(),
                ty: Type::Any,
            }),
            name: "__closure_id".to_string(),
            ty: Type::Number,
        };

        let mut dispatch = HirExpr::Call {
            callee: table.closures[0].fn_name.clone(),
            args: call_args.clone(),
            ty: Type::Any,
        };

        for closure in table.closures.iter().skip(1) {
            let cond = HirExpr::Binary {
                op: crate::hir::HirBinOp::Eq,
                lhs: Box::new(tid.clone()),
                rhs: Box::new(HirExpr::IntLit(closure.id as i64)),
                ty: Type::Boolean,
            };
            let target = HirExpr::Call {
                callee: closure.fn_name.clone(),
                args: call_args.clone(),
                ty: Type::Any,
            };
            dispatch = HirExpr::Ternary {
                condition: Box::new(cond),
                true_expr: Box::new(target),
                false_expr: Box::new(dispatch),
                ty: Type::Any,
            };
        }

        Some(dispatch)
    })
}

pub(crate) fn lift_closure(
    params: &[String],
    body: &ArrowFunctionBody,
    span: Span,
) -> Result<(HirExpr, HirFunction), HirLowerError> {
    let id = CLOSURE_COUNTER.with(|c| {
        let val = *c.borrow();
        *c.borrow_mut() = val + 1;
        val
    });
    let fn_name = format!("__closure_{id}");

    let stmts = match body {
        ArrowFunctionBody::Block(s) => s.clone(),
        ArrowFunctionBody::Expression(e) => {
            vec![Stmt::Return { value: Some(*e.clone()), span }]
        }
    };

    let captured = find_free_vars(&stmts, params);
    let rewritten_stmts = rewrite_free_vars(&stmts, &captured, span);

    let mut fn_params = vec!["self".to_string()];
    fn_params.extend(params.iter().cloned());

    let hir_fn = lower_function(&fn_name, &fn_params, &rewritten_stmts)?;

    let mut props = vec![
        ("__closure_id".to_string(), HirExpr::IntLit(id as i64)),
    ];
    for var in &captured {
        props.push((var.clone(), HirExpr::Local { name: var.clone(), ty: Type::Any }));
    }

    let obj = HirExpr::ObjectLit { properties: props, ty: Type::Any };

    CURRENT_CLOSURES.with(|cell| {
        if let Some(table) = cell.borrow_mut().as_mut() {
            table.closures.push(LiftedClosure {
                id,
                fn_name: fn_name.clone(),
                captured_vars: captured,
            });
            table.functions.push(hir_fn.clone());
        }
    });

    Ok((obj, hir_fn))
}

fn find_free_vars(stmts: &[Stmt], params: &[String]) -> Vec<String> {
    let mut bound: HashSet<String> = params.iter().cloned().collect();
    bound.insert("self".to_string());
    let mut free = Vec::new();

    for stmt in stmts {
        collect_free_in_stmt(stmt, &mut bound, &mut free);
    }

    free.sort();
    free.dedup();
    free
}

fn collect_free_in_stmt(stmt: &Stmt, bound: &mut HashSet<String>, free: &mut Vec<String>) {
    match stmt {
        Stmt::Let { name, value, .. } => {
            collect_free_in_expr(value, bound, free);
            bound.insert(name.clone());
        }
        Stmt::Assignment { target, value, .. } => {
            if let Expr::Identifier(name, _) = target {
                if !bound.contains(name) {
                    free.push(name.clone());
                }
            } else {
                collect_free_in_expr(target, bound, free);
            }
            collect_free_in_expr(value, bound, free);
        }
        Stmt::Return { value: Some(v), .. } => collect_free_in_expr(v, bound, free),
        Stmt::Expr(e) => collect_free_in_expr(e, bound, free),
        Stmt::If { condition, then_branch, else_branch, .. } => {
            collect_free_in_expr(condition, bound, free);
            collect_free_in_stmt(then_branch, bound, free);
            if let Some(e) = else_branch {
                collect_free_in_stmt(e, bound, free);
            }
        }
        Stmt::While { condition, body, .. } => {
            collect_free_in_expr(condition, bound, free);
            collect_free_in_stmt(body, bound, free);
        }
        Stmt::Block { statements, .. } => {
            for s in statements {
                collect_free_in_stmt(s, bound, free);
            }
        }
        _ => {}
    }
}

fn collect_free_in_expr(expr: &Expr, bound: &HashSet<String>, free: &mut Vec<String>) {
    match expr {
        Expr::Identifier(name, _) => {
            if !bound.contains(name) {
                free.push(name.clone());
            }
        }
        Expr::Binary { left, right, .. } => {
            collect_free_in_expr(left, bound, free);
            collect_free_in_expr(right, bound, free);
        }
        Expr::Unary { expr: inner, .. } => collect_free_in_expr(inner, bound, free),
        Expr::Call { callee, args, .. } => {
            collect_free_in_expr(callee, bound, free);
            for a in args {
                collect_free_in_expr(a, bound, free);
            }
        }
        Expr::Member { object, .. } => collect_free_in_expr(object, bound, free),
        Expr::Index { object, index, .. } => {
            collect_free_in_expr(object, bound, free);
            collect_free_in_expr(index, bound, free);
        }
        _ => {}
    }
}

fn rewrite_free_vars(stmts: &[Stmt], free: &[String], span: Span) -> Vec<Stmt> {
    stmts.iter().map(|s| rewrite_stmt(s, free, span)).collect()
}

fn rewrite_stmt(stmt: &Stmt, free: &[String], span: Span) -> Stmt {
    match stmt {
        Stmt::Assignment { target, value, span: s_span } => {
            let new_target = if let Expr::Identifier(name, id_span) = target {
                if free.contains(name) {
                    Expr::Member {
                        object: Box::new(Expr::Identifier("self".to_string(), *id_span)),
                        property: name.clone(),
                        span: *id_span,
                    }
                } else {
                    target.clone()
                }
            } else {
                rewrite_expr(target, free, span)
            };
            Stmt::Assignment {
                target: new_target,
                value: rewrite_expr(value, free, span),
                span: *s_span,
            }
        }
        Stmt::Return { value, span: s_span } => Stmt::Return {
            value: value.as_ref().map(|v| rewrite_expr(v, free, span)),
            span: *s_span,
        },
        Stmt::Expr(e) => Stmt::Expr(rewrite_expr(e, free, span)),
        Stmt::Block { statements, span: s_span } => Stmt::Block {
            statements: rewrite_free_vars(statements, free, span),
            span: *s_span,
        },
        other => other.clone(),
    }
}

fn rewrite_expr(expr: &Expr, free: &[String], span: Span) -> Expr {
    match expr {
        Expr::Identifier(name, id_span) if free.contains(name) => {
            Expr::Member {
                object: Box::new(Expr::Identifier("self".to_string(), *id_span)),
                property: name.clone(),
                span: *id_span,
            }
        }
        Expr::Binary { left, op, right, span: b_span } => Expr::Binary {
            left: Box::new(rewrite_expr(left, free, span)),
            op: *op,
            right: Box::new(rewrite_expr(right, free, span)),
            span: *b_span,
        },
        Expr::Unary { op, expr: inner, span: u_span } => Expr::Unary {
            op: *op,
            expr: Box::new(rewrite_expr(inner, free, span)),
            span: *u_span,
        },
        Expr::Call { callee, args, span: c_span } => Expr::Call {
            callee: Box::new(rewrite_expr(callee, free, span)),
            args: args.iter().map(|a| rewrite_expr(a, free, span)).collect(),
            span: *c_span,
        },
        Expr::Member { object, property, span: m_span } => Expr::Member {
            object: Box::new(rewrite_expr(object, free, span)),
            property: property.clone(),
            span: *m_span,
        },
        other => other.clone(),
    }
}
