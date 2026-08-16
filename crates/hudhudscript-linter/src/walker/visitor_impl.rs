use super::LintContext;
use crate::rules;
use hudhudscript_ast::visitor::{self, AstVisitor, VisitControl};
use hudhudscript_ast::*;
use hudhudscript_errors::ScopeManager;

impl<'cfg> AstVisitor for LintContext<'cfg> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> VisitControl {
        match stmt {
            // ── Variable declarations ──────────────────────────────────
            Stmt::Let { name, span, .. } => {
                self.declare_variable(name, *span);
                self.let_declarations.push((name.clone(), *span));
                // Continue — visitor walks the value expression child
            }
            Stmt::Const { name, span, .. } => {
                self.declare_variable(name, *span);
                self.const_names.insert(name.clone());
                // Continue — visitor walks the value expression child
            }
            Stmt::VarDecl(decl) => {
                self.declare_variable(&decl.name, decl.span);
                if decl.is_const {
                    self.const_names.insert(decl.name.clone());
                } else {
                    self.let_declarations.push((decl.name.clone(), decl.span));
                }
                // Continue — visitor walks the initializer child
            }

            // ── Assignment — check const-reassign ─────────────────────
            Stmt::Assignment { target, span, .. } => {
                if let Expr::Identifier(name, _) = target {
                    self.assigned_names.insert(name.clone());
                    if self.const_names.contains(name) {
                        rules::const_reassign::check(
                            name,
                            *span,
                            self.config,
                            &mut self.diagnostics,
                        );
                    }
                }
                // Continue — visitor walks target + value expressions
            }

            // ── Function declaration ───────────────────────────────────
            Stmt::Function {
                name,
                params,
                body,
                span,
                ..
            } => {
                // Empty block check
                rules::empty_block::check(
                    body,
                    "function",
                    name,
                    *span,
                    self.config,
                    &mut self.diagnostics,
                );
                // Missing return check
                rules::missing_return::check(body, name, *span, self.config, &mut self.diagnostics);
                // Push scope for function body and declare params
                self.push_scope();
                for p in params {
                    self.declare_variable(p, *span);
                }
                // Continue — visitor walks the body children.
                // Scope is popped in leave_stmt.
            }

            // ── Class — manually walk because each method needs its own scope ──
            Stmt::Class(class_decl) => {
                rules::naming_convention::check_type_name(
                    &class_decl.name,
                    "class",
                    class_decl.span,
                    self.config,
                    &mut self.diagnostics,
                );
                for member in &class_decl.members {
                    match member {
                        ClassMember::Method {
                            body,
                            name,
                            params,
                            span,
                            ..
                        } => {
                            rules::empty_block::check(
                                body,
                                "method",
                                name,
                                *span,
                                self.config,
                                &mut self.diagnostics,
                            );
                            self.push_scope();
                            for p in params {
                                self.declare_variable(&p.name, p.span);
                            }
                            visitor::walk_stmts(self, body);
                            self.pop_scope();
                        }
                        ClassMember::Constructor {
                            body, params, span, ..
                        } => {
                            rules::empty_block::check(
                                body,
                                "constructor",
                                &class_decl.name,
                                *span,
                                self.config,
                                &mut self.diagnostics,
                            );
                            self.push_scope();
                            for p in params {
                                self.declare_variable(&p.name, p.span);
                            }
                            visitor::walk_stmts(self, body);
                            self.pop_scope();
                        }
                        ClassMember::Field { initializer, .. } => {
                            if let Some(init) = initializer {
                                visitor::walk_expr(self, init);
                            }
                        }
                    }
                }
                // We handled children manually — skip the visitor's child walk
                return VisitControl::SkipChildren;
            }

            // ── Enum ───────────────────────────────────────────────────
            Stmt::EnumDecl { name, span, .. } => {
                rules::naming_convention::check_type_name(
                    name,
                    "enum",
                    *span,
                    self.config,
                    &mut self.diagnostics,
                );
            }

            // ── Block — scope + depth tracking ─────────────────────────
            Stmt::Block { span, .. } => {
                self.depth += 1;
                rules::deep_nesting::check(self.depth, *span, self.config, &mut self.diagnostics);
                self.push_scope();
                // Continue — visitor walks children. Scope/depth restored in leave_stmt.
            }

            // ── For loop — scope for loop variable ─────────────────────
            Stmt::For { variable, span, .. } => {
                self.push_scope();
                self.declare_variable(variable, *span);
                // Continue — visitor walks iterable + body.
                // Scope popped in leave_stmt.
            }
            Stmt::ForCStyle { .. } => {
                self.push_scope();
                // Continue — visitor walks init/cond/update/body.
                // Scope popped in leave_stmt.
            }

            // ── Try — manually walk because catch clause needs its own scope ──
            Stmt::Try {
                try_block,
                catch_clause,
                finally_block,
                ..
            } => {
                visitor::walk_stmt(self, try_block);
                if let Some(cc) = catch_clause {
                    // Empty catch check
                    let catch_body_empty = match cc.body.as_ref() {
                        Stmt::Block { statements, .. } => statements.is_empty(),
                        _ => false,
                    };
                    rules::empty_catch::check(
                        catch_body_empty,
                        &cc.param,
                        cc.span,
                        self.config,
                        &mut self.diagnostics,
                    );
                    self.push_scope();
                    self.declare_variable(&cc.param, cc.span);
                    visitor::walk_stmt(self, &cc.body);
                    self.pop_scope();
                }
                if let Some(fb) = finally_block {
                    visitor::walk_stmt(self, fb);
                }
                return VisitControl::SkipChildren;
            }

            // ── Receive — declares a variable ──────────────────────────
            Stmt::Receive { variable, span, .. } => {
                self.declare_variable(variable, *span);
                // Continue — visitor walks source expression child
            }

            // All other statements: no special pre-walk handling needed.
            // The visitor walks their children automatically.
            _ => {}
        }
        VisitControl::Continue
    }

    fn leave_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Function { .. } => {
                self.pop_scope();
            }
            Stmt::Block { .. } => {
                self.pop_scope();
                self.depth -= 1;
            }
            Stmt::For { .. } | Stmt::ForCStyle { .. } => {
                self.pop_scope();
            }
            // Class and Try are handled with SkipChildren so leave_stmt won't
            // be called for them — but include them for safety.
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> VisitControl {
        match expr {
            Expr::Identifier(name, _) => {
                rules::unused_variables::record_ref(self, name);
            }
            Expr::ArrowFunction { params, span, .. } => {
                self.push_scope();
                for p in params {
                    self.declare_variable(p, *span);
                }
                // Continue — visitor walks body children.
                // Scope popped in leave_expr.
            }
            // ── no-print rule: detect print/println calls ─────────────
            Expr::Call { callee, span, .. } => {
                if let Expr::Identifier(name, _) = callee.as_ref() {
                    rules::no_print::check(name, *span, self.config, &mut self.diagnostics);
                }
            }
            _ => {}
        }
        VisitControl::Continue
    }

    fn leave_expr(&mut self, expr: &Expr) {
        if matches!(expr, Expr::ArrowFunction { .. }) {
            self.pop_scope();
        }
    }

    fn visit_decl(&mut self, decl: &Decl) -> VisitControl {
        match decl {
            Decl::Agent { name, span, .. } => {
                rules::naming_convention::check_type_name(
                    name,
                    "agent",
                    *span,
                    self.config,
                    &mut self.diagnostics,
                );
            }
            Decl::Subject { name, span, .. } => {
                rules::naming_convention::check_type_name(
                    name,
                    "subject",
                    *span,
                    self.config,
                    &mut self.diagnostics,
                );
            }
            Decl::Role { name, span, .. } => {
                rules::naming_convention::check_type_name(
                    name,
                    "role",
                    *span,
                    self.config,
                    &mut self.diagnostics,
                );
            }
            Decl::Effect { .. } => {
                self.push_scope();
                // Continue — visitor walks body. Scope popped in leave_decl.
            }
            _ => {}
        }
        VisitControl::Continue
    }

    fn leave_decl(&mut self, decl: &Decl) {
        if matches!(decl, Decl::Effect { .. }) {
            self.pop_scope();
        }
    }
}
