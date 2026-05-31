use super::*;

impl Formatter {
    pub(super) fn format_stmt(&mut self, stmt: &Stmt) -> String {
        let indent = self.get_indent();

        match stmt {
            Stmt::Let { name, value, .. } => {
                format!("{}let {} = {};\n", indent, name, self.format_expr(value))
            }
            Stmt::Const { name, value, .. } => {
                format!("{}const {} = {};\n", indent, name, self.format_expr(value))
            }
            Stmt::VarDecl(decl) => self.format_var_decl(decl),
            Stmt::Assignment { target, value, .. } => {
                format!(
                    "{}{} = {};\n",
                    indent,
                    self.format_expr(target),
                    self.format_expr(value)
                )
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    format!("{}return {};\n", indent, self.format_expr(v))
                } else {
                    format!("{}return;\n", indent)
                }
            }
            Stmt::Break { .. } => format!("{}break;\n", indent),
            Stmt::Continue { .. } => format!("{}continue;\n", indent),
            Stmt::Expr(expr) => {
                format!("{}{};\n", indent, self.format_expr(expr))
            }
            Stmt::Block { statements, .. } => {
                let mut output = format!("{}{{\n", indent);
                self.current_indent += 1;
                for s in statements {
                    output.push_str(&self.format_stmt(s));
                }
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let mut output = format!("{}if ({}) {{\n", indent, self.format_expr(condition));
                self.current_indent += 1;
                output.push_str(&self.format_stmt(then_branch));
                self.current_indent -= 1;

                if let Some(else_stmt) = else_branch {
                    output.push_str(&format!("{}}} else {{\n", indent));
                    self.current_indent += 1;
                    output.push_str(&self.format_stmt(else_stmt));
                    self.current_indent -= 1;
                }
                output.push_str(&format!("{}}}\n", indent));
                output
            }
            Stmt::While {
                condition, body, ..
            } => {
                let mut output = format!("{}while ({}) {{\n", indent, self.format_expr(condition));
                self.current_indent += 1;
                output.push_str(&self.format_stmt(body));
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
            Stmt::For {
                variable,
                iterable,
                body,
                ..
            } => {
                let mut output = format!(
                    "{}for ({} in {}) {{\n",
                    indent,
                    variable,
                    self.format_expr(iterable)
                );
                self.current_indent += 1;
                output.push_str(&self.format_stmt(body));
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
            Stmt::ForCStyle {
                init,
                condition,
                update,
                body,
                ..
            } => {
                let init_str = if let Some(init_stmt) = init {
                    // Format the init statement and trim trailing newline/semicolon for inline use
                    let s = self.format_stmt(init_stmt);
                    s.trim().trim_end_matches(';').to_string()
                } else {
                    String::new()
                };
                let cond_str = if let Some(cond) = condition {
                    self.format_expr(cond)
                } else {
                    String::new()
                };
                let update_str = if let Some(upd_stmt) = update {
                    let s = self.format_stmt(upd_stmt);
                    s.trim().trim_end_matches(';').to_string()
                } else {
                    String::new()
                };
                let mut output = format!(
                    "{}for ({}; {}; {}) {{\n",
                    indent, init_str, cond_str, update_str
                );
                self.current_indent += 1;
                output.push_str(&self.format_stmt(body));
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
            Stmt::ForRange {
                start,
                stop,
                step,
                body,
                ..
            } => {
                let mut output = if let Some(step_expr) = step {
                    format!(
                        "{}for({}, {}, {}) {{\n",
                        indent,
                        self.format_expr(start),
                        self.format_expr(stop),
                        self.format_expr(step_expr)
                    )
                } else {
                    format!(
                        "{}for({}, {}) {{\n",
                        indent,
                        self.format_expr(start),
                        self.format_expr(stop)
                    )
                };
                self.current_indent += 1;
                output.push_str(&self.format_stmt(body));
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
            Stmt::Decl(decl) => self.format_decl(decl),
            Stmt::Function {
                name,
                params,
                body,
                is_async,
                ..
            } => {
                let async_kw = if *is_async { "async " } else { "" };
                let params_str = params.join(", ");
                let mut output = format!(
                    "{}{}function {}({}) {{\n",
                    indent, async_kw, name, params_str
                );

                self.current_indent += 1;
                for stmt in body {
                    output.push_str(&self.format_stmt(stmt));
                }
                self.current_indent -= 1;

                output.push_str(&format!("{}}}\n", indent));
                output
            }
            Stmt::Switch {
                value,
                cases,
                default,
                ..
            } => self.format_switch(&indent, value, cases, default),
            Stmt::Try {
                try_block,
                catch_clause,
                finally_block,
                ..
            } => self.format_try(&indent, try_block, catch_clause, finally_block),
            Stmt::Throw { value, .. } => {
                format!("{}throw {};\n", indent, self.format_expr(value))
            }
            Stmt::Import { path, imports, .. } => self.format_import_stmt(&indent, path, imports),
            Stmt::Export { item, source, .. } => {
                let item_str = self.format_stmt(item);
                // The inner statement already has indent, so strip it and re-add with "export "
                let trimmed = item_str.trim_start();
                if let Some(src) = source {
                    format!(
                        "{}export {} from \"{}\";",
                        indent,
                        trimmed.trim_end_matches(';').trim_end(),
                        src
                    )
                } else {
                    format!("{}export {}", indent, trimmed)
                }
            }
            Stmt::Class(class_decl) => self.format_class(&indent, class_decl),
            Stmt::McpServer(mcp_decl) => self.format_mcp_server(&indent, mcp_decl),
            Stmt::Match { value, arms, .. } => self.format_match(&indent, value, arms),
            Stmt::EnumDecl { name, variants, .. } => self.format_enum_decl(&indent, name, variants),
            // SOP statements
            Stmt::Spawn {
                subject_name, args, ..
            } => {
                if args.is_empty() {
                    format!("{}spawn {};\n", indent, subject_name)
                } else {
                    let args_str = args
                        .iter()
                        .map(|a| self.format_expr(a))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}spawn {}({});\n", indent, subject_name, args_str)
                }
            }
            Stmt::Despawn { name, .. } => {
                format!("{}despawn {};\n", indent, name)
            }
            Stmt::Send {
                message, target, ..
            } => format!(
                "{}send {} to {};\n",
                indent,
                self.format_expr(message),
                self.format_expr(target)
            ),
            Stmt::Receive {
                variable, source, ..
            } => format!(
                "{}receive {} from {};\n",
                indent,
                variable,
                self.format_expr(source)
            ),
            Stmt::Require { condition, .. } => {
                format!("{}require {};\n", indent, self.format_expr(condition))
            }
            Stmt::Perform { action, .. } => {
                format!("{}perform {};\n", indent, self.format_expr(action))
            }
            // RAG statements
            Stmt::Remember {
                content,
                store_name,
                ..
            } => {
                if let Some(store) = store_name {
                    format!(
                        "{}remember {} in {};\n",
                        indent,
                        self.format_expr(content),
                        store
                    )
                } else {
                    format!("{}remember {};\n", indent, self.format_expr(content))
                }
            }
            Stmt::Recall {
                query, store_name, ..
            } => {
                if let Some(store) = store_name {
                    format!(
                        "{}recall {} from {};\n",
                        indent,
                        self.format_expr(query),
                        store
                    )
                } else {
                    format!("{}recall {};\n", indent, self.format_expr(query))
                }
            }
            Stmt::Forget {
                target, store_name, ..
            } => {
                if let Some(store) = store_name {
                    format!(
                        "{}forget {} from {};\n",
                        indent,
                        self.format_expr(target),
                        store
                    )
                } else {
                    format!("{}forget {};\n", indent, self.format_expr(target))
                }
            }
            Stmt::Destructure {
                pattern,
                value,
                is_const,
                ..
            } => {
                let keyword = if *is_const { "const" } else { "let" };
                let pattern_str = self.format_pattern(pattern);
                format!(
                    "{}{} {} = {};\n",
                    indent,
                    keyword,
                    pattern_str,
                    self.format_expr(value)
                )
            }
            Stmt::Trait { name, methods, .. } => {
                let mut output = format!("{}trait {} {{\n", indent, name);
                self.current_indent += 1;
                let inner_indent = self.get_indent();
                for method in methods {
                    let params_str = method
                        .params
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    if let Some(ref ret) = method.return_type {
                        output.push_str(&format!(
                            "{}function {}({}): {};\n",
                            inner_indent,
                            method.name,
                            params_str,
                            self.format_type_annotation(ret)
                        ));
                    } else {
                        output.push_str(&format!(
                            "{}function {}({});\n",
                            inner_indent, method.name, params_str
                        ));
                    }
                }
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
        }
    }
}
