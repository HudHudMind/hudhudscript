// P4b: Pure AST pre-pass for call-site type collection.
// Walks the AST before compilation to collect function signatures and
// call-site argument types. No bytecode emitted.

use super::*;
use crate::compiler::expr::ExprType;
use hudhudscript_ast::{Expr, Literal, Stmt};
use std::collections::HashMap;

impl Compiler {
    pub(super) fn p4b_prepass_collect(&mut self, stmts: &[Stmt]) {
        // Phase 1: collect all function signatures
        for stmt in stmts {
            if let Stmt::Function { name, params, .. } = stmt {
                self.fn_param_names.insert(name.clone(), params.clone());
            }
        }
        // Phase 2: walk AST with scope-aware type environment
        let mut scope = Scope::new();
        for stmt in stmts {
            self.p4b_walk_stmt(stmt, &mut scope);
        }
    }

    fn p4b_walk_stmt(&mut self, stmt: &Stmt, scope: &mut Scope) {
        match stmt {
            Stmt::Expr(expr) => self.p4b_walk_expr(expr, scope),
            Stmt::Let { name, value, .. } => {
                self.p4b_walk_expr(value, scope);
                let ty = self.p4b_expr_type(value);
                scope.insert(name.clone(), ty);
            }
            Stmt::Assignment { target, value, .. } => {
                self.p4b_walk_expr(value, scope);
                if let Expr::Identifier(name, _) = target {
                    let new_ty = self.p4b_expr_type(value);
                    scope.update_or_degrade(name, new_ty);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.p4b_walk_expr(v, scope);
                }
            }
            Stmt::Block { statements, .. } => {
                // P4b: block does not create child scope (assignments must propagate up)
                for s in statements {
                    self.p4b_walk_stmt(s, scope);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.p4b_walk_expr(condition, scope);
                // P4b: if body uses same scope (mutations affect outer scope)
                self.p4b_walk_stmt(then_branch, scope);
                if let Some(e) = else_branch {
                    self.p4b_walk_stmt(e, scope);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.p4b_walk_expr(condition, scope);
                // P4b: while body uses same scope (mutations affect outer scope)
                self.p4b_walk_stmt(body, scope);
            }
            Stmt::For { iterable, body, .. } => {
                self.p4b_walk_expr(iterable, scope);
                self.p4b_walk_stmt(body, scope);
            }
            Stmt::Function {
                name, params, body, ..
            } => {
                // Function scope: add all params as Unknown to block outer leaks
                let mut inner = scope.child();
                for p in params {
                    inner.insert(p.clone(), ExprType::Unknown);
                }
                for s in body {
                    self.p4b_walk_stmt(s, &mut inner);
                }
            }
            _ => {}
        }
    }

    fn p4b_walk_expr(&mut self, expr: &Expr, scope: &mut Scope) {
        match expr {
            Expr::Call { callee, args, .. } => {
                // Batch-record all arg types at once
                if let Expr::Identifier(fn_name, _) = callee.as_ref() {
                    if let Some(param_names) = self.fn_param_names.get(fn_name).cloned() {
                        let mut arg_types: Vec<(String, ExprType)> = Vec::new();
                        for (i, _) in param_names.iter().enumerate() {
                            let ty = if let Some(arg) = args.get(i) {
                                let mut ty = self.p4b_expr_type(arg);
                                if ty == ExprType::Unknown {
                                    if let Expr::Identifier(name, _) = arg {
                                        ty = scope.lookup(name).unwrap_or(ExprType::Unknown);
                                    }
                                }
                                ty
                            } else {
                                ExprType::Unknown // missing arg
                            };
                            let pname = param_names[i].clone();
                            arg_types.push((pname, ty));
                        }
                        if !arg_types.is_empty() {
                            self.ct_record_call_site_types(fn_name, &arg_types);
                        }
                    }
                }
                self.p4b_walk_expr(callee, scope);
                for a in args {
                    self.p4b_walk_expr(a, scope);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.p4b_walk_expr(left, scope);
                self.p4b_walk_expr(right, scope);
            }
            Expr::Member { object, .. } => self.p4b_walk_expr(object, scope),
            Expr::Index { object, index, .. } => {
                self.p4b_walk_expr(object, scope);
                self.p4b_walk_expr(index, scope);
            }
            Expr::Array { elements, .. } => {
                for e in elements {
                    self.p4b_walk_expr(e, scope);
                }
            }
            _ => {}
        }
    }

    fn p4b_expr_type(&self, expr: &Expr) -> ExprType {
        match expr {
            Expr::Array { .. } => ExprType::Array,
            Expr::Literal(Literal::String(_), _) => ExprType::Str,
            Expr::Literal(Literal::Number(_, true), _) => ExprType::Number,
            Expr::Literal(Literal::Number(_, false), _) => ExprType::Int,
            Expr::Literal(Literal::Int(_), _) => ExprType::Int,
            Expr::Literal(Literal::BigInt(_), _) => ExprType::Int,
            _ => ExprType::Unknown,
        }
    }
}

struct Scope {
    locals: HashMap<String, ExprType>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            locals: HashMap::new(),
        }
    }
    fn child(&self) -> Self {
        Scope {
            locals: self.locals.clone(),
        }
    }
    fn insert(&mut self, name: String, ty: ExprType) {
        self.locals.insert(name, ty);
    }
    fn lookup(&self, name: &str) -> Option<ExprType> {
        self.locals.get(name).copied()
    }
    /// P4b: update type on reassignment. Degrade to Unknown if type changed or uncertain.
    fn update_or_degrade(&mut self, name: &str, new_ty: ExprType) {
        if new_ty == ExprType::Unknown {
            self.locals.insert(name.to_string(), ExprType::Unknown);
        } else if let Some(&old_ty) = self.locals.get(name) {
            if old_ty != new_ty && old_ty != ExprType::Unknown {
                self.locals.insert(name.to_string(), ExprType::Unknown);
            }
        }
    }
}
