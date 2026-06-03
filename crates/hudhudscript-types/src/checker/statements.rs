use super::*;

impl TypeChecker {
    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> {
        match stmt {
            Stmt::VarDecl(decl) => self.check_var_decl(decl),

            // Let/Const are separate statement kinds that must be registered in the symbol table
            Stmt::Let { name, value, span } => {
                let ty = self.check_expr(value)?;
                if let Err(_) = self.symbol_table.define_with_info(name.clone(), SymbolInfo::mutable_owned(ty)) {
                    return self.handle_duplicate(&name, *span);
                }
                Ok(())
            }
            Stmt::Const { name, value, span } => {
                let ty = self.check_expr(value)?;
                if let Err(_) = self.symbol_table.define_with_info(name.clone(), SymbolInfo::immutable_owned(ty)) {
                    return self.handle_duplicate(&name, *span);
                }
                Ok(())
            }

            Stmt::Assignment {
                target,
                value,
                span,
            } => {
                let value_type = self.check_expr(value)?;

                // For simple identifier assignments
                if let Expr::Identifier(name, _) = target {
                    if let Some(info) = self.symbol_table.lookup_info(name) {
                        // Issue #330: reject assignment to immutable bindings
                        if !info.mutable {
                            return Err(type_codes::invalid_operator(
                                "assignment".to_string(),
                                format!("immutable binding '{}'", name),
                                *span,
                            ));
                        }
                        if !value_type.is_compatible_with(&info.ty) {
                            return Err(type_codes::mismatch(
                                format!("{}", info.ty),
                                format!("{}", value_type),
                                *span,
                            ));
                        }
                        Ok(())
                    } else {
                        Err(type_codes::undefined_variable(name.clone(), *span))
                    }
                } else {
                    // For complex assignments (member access, index), just check the target and value
                    self.check_expr(target)?;
                    Ok(())
                }
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.check_expr(expr)?;
                }
                Ok(())
            }
            Stmt::Throw { value, .. } => {
                self.check_expr(value)?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_type = self.check_expr(condition)?;
                if !cond_type.is_compatible_with(&Type::Boolean) && cond_type != Type::Any {
                    return Err(type_codes::mismatch(
                        "Boolean".to_string(),
                        format!("{}", cond_type),
                        condition.span(),
                    ));
                }

                self.check_stmt(then_branch)?;

                if let Some(else_stmts) = else_branch {
                    self.check_stmt(else_stmts)?;
                }
                Ok(())
            }
            Stmt::While {
                condition, body, ..
            } => {
                let cond_type = self.check_expr(condition)?;
                if !cond_type.is_compatible_with(&Type::Boolean) && cond_type != Type::Any {
                    return Err(type_codes::mismatch(
                        "Boolean".to_string(),
                        format!("{}", cond_type),
                        condition.span(),
                    ));
                }

                self.check_stmt(body)?;
                Ok(())
            }
            Stmt::For {
                variable,
                iterable,
                body,
                span,
            } => {
                // Check the iterable expression
                let iter_type = self.check_expr(iterable)?;

                // Determine element type: Array<T> -> T, otherwise Any
                let elem_type = match iter_type {
                    Type::Array(inner) => *inner,
                    Type::Any => Type::Any,
                    _ => Type::Any,
                };

                // Open a scope so the loop variable doesn't leak out
                self.symbol_table.enter_scope();
                if let Err(_) = self.symbol_table.define(variable.clone(), elem_type) {
                    self.handle_duplicate(&variable, *span)?;
                }
                self.check_stmt(body)?;
                self.symbol_table.exit_scope();
                Ok(())
            }
            Stmt::ForCStyle {
                init,
                condition,
                update,
                body,
                ..
            } => {
                // C-style for: open a scope for the init statement
                self.symbol_table.enter_scope();
                if let Some(init_stmt) = init {
                    self.check_stmt(init_stmt)?;
                }
                if let Some(cond_expr) = condition {
                    self.check_expr(cond_expr)?;
                }
                self.check_stmt(body)?;
                if let Some(update_stmt) = update {
                    self.check_stmt(update_stmt)?;
                }
                self.symbol_table.exit_scope();
                Ok(())
            }
            Stmt::ForRange {
                start,
                stop,
                step,
                body,
                ..
            } => {
                self.check_expr(start)?;
                self.check_expr(stop)?;
                if let Some(step_expr) = step {
                    self.check_expr(step_expr)?;
                }
                self.check_stmt(body)?;
                Ok(())
            }
            Stmt::Block { statements, .. } => {
                self.symbol_table.enter_scope();
                for s in statements {
                    self.check_stmt(s)?;
                }
                self.symbol_table.exit_scope();
                Ok(())
            }
            Stmt::Function {
                name,
                params,
                body,
                type_params,
                ..
            } => {
                // Register the function itself in the current scope so it can be called
                let param_types: Vec<Type> = params.iter().map(|_| Type::Any).collect();
                let fn_type = Type::Function {
                    params: param_types.clone(),
                    return_type: Box::new(Type::Any),
                };
                // Ignore duplicate errors (forward declarations are OK)
                let _ = self.symbol_table.define(name.clone(), fn_type);

                // Check the body in a new scope with parameters bound
                self.symbol_table.enter_scope();

                // Issue #1009: Register generic type parameters and their constraints
                self.register_generic_params(type_params);

                for (param_name, param_type) in params.iter().zip(param_types.iter()) {
                    let _ = self
                        .symbol_table
                        .define(param_name.clone(), param_type.clone());
                }
                for s in body {
                    self.check_stmt(s)?;
                }

                // Issue #1009: Clean up generic constraints when leaving scope
                self.unregister_generic_params(type_params);

                self.symbol_table.exit_scope();
                Ok(())
            }
            Stmt::Switch {
                value,
                cases,
                default,
                ..
            } => {
                self.check_expr(value)?;
                for case in cases {
                    self.check_expr(&case.value)?;
                    self.symbol_table.enter_scope();
                    for s in &case.body {
                        self.check_stmt(s)?;
                    }
                    self.symbol_table.exit_scope();
                }
                if let Some(default_stmts) = default {
                    self.symbol_table.enter_scope();
                    for s in default_stmts {
                        self.check_stmt(s)?;
                    }
                    self.symbol_table.exit_scope();
                }
                Ok(())
            }
            Stmt::Try {
                try_block,
                catch_clause,
                finally_block,
                ..
            } => {
                self.check_stmt(try_block)?;
                if let Some(catch) = catch_clause {
                    self.symbol_table.enter_scope();
                    // Bind the catch parameter as Any (we don't know the error type)
                    let _ = self.symbol_table.define(catch.param.clone(), Type::Any);
                    self.check_stmt(&catch.body)?;
                    self.symbol_table.exit_scope();
                }
                if let Some(finally) = finally_block {
                    self.check_stmt(finally)?;
                }
                Ok(())
            }
            Stmt::Export { item, .. } => {
                // Check the exported item as a regular statement
                self.check_stmt(item)
            }
            Stmt::Match { value, arms, span } => {
                let subject_type = self.check_expr(value)?;
                // Issue #1018: Check exhaustiveness for union types.
                self.check_match_exhaustiveness(&subject_type, arms, *span);
                for arm in arms {
                    self.symbol_table.enter_scope();
                    // If the pattern binds a name, introduce it as Any
                    self.bind_match_pattern_types(&arm.pattern);
                    // Issue #748: check guard expression if present
                    if let Some(guard) = &arm.guard {
                        self.check_expr(guard)?;
                    }
                    for s in &arm.body {
                        self.check_stmt(s)?;
                    }
                    self.symbol_table.exit_scope();
                }
                Ok(())
            }
            Stmt::Class(class_decl) => self.check_class_decl(class_decl),

            // Check expression sub-trees of domain-specific statements
            Stmt::Spawn { args, .. } => {
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(())
            }
            Stmt::Despawn { .. } => Ok(()),
            Stmt::Send {
                message, target, ..
            } => {
                self.check_expr(message)?;
                self.check_expr(target)?;
                Ok(())
            }
            Stmt::Receive { source, .. } => {
                self.check_expr(source)?;
                Ok(())
            }
            Stmt::Require { condition, .. } => {
                self.check_expr(condition)?;
                Ok(())
            }
            Stmt::Perform { action, .. } => {
                self.check_expr(action)?;
                Ok(())
            }
            Stmt::Remember { content, .. } => {
                self.check_expr(content)?;
                Ok(())
            }
            Stmt::Recall { query, .. } => {
                self.check_expr(query)?;
                Ok(())
            }
            Stmt::Forget { target, .. } => {
                self.check_expr(target)?;
                Ok(())
            }
            // These don't have expression sub-trees to check
            Stmt::Import { .. }
            | Stmt::Decl(_)
            | Stmt::McpServer(_)
            | Stmt::EnumDecl { .. }
            | Stmt::Break { .. }
            | Stmt::Continue { .. }
            | Stmt::Destructure { .. }
            | Stmt::Trait { .. } => Ok(()),
        }
    }
}
