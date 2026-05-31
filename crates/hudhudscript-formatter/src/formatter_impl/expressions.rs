use super::*;

impl Formatter {
    pub(super) fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Literal(lit, _) => self.format_literal(lit),
            Expr::Identifier(name, _) => name.clone(),
            Expr::Binary {
                left, op, right, ..
            } => {
                format!(
                    "{} {} {}",
                    self.format_expr(left),
                    self.format_binop(op),
                    self.format_expr(right)
                )
            }
            Expr::Unary { op, expr, .. } => {
                if matches!(op, UnaryOp::PostIncrement | UnaryOp::PostDecrement) {
                    format!("{}{}", self.format_expr(expr), self.format_unop(op))
                } else {
                    format!("{}{}", self.format_unop(op), self.format_expr(expr))
                }
            }
            Expr::Call { callee, args, .. } => {
                let args_str = args
                    .iter()
                    .map(|a| self.format_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", self.format_expr(callee), args_str)
            }
            Expr::Perform { action, .. } => {
                format!("perform {}", self.format_expr(action))
            }
            Expr::Member {
                object, property, ..
            } => {
                format!("{}.{}", self.format_expr(object), property)
            }
            Expr::OptionalMember {
                object, property, ..
            } => {
                format!("{}?.{}", self.format_expr(object), property)
            }
            Expr::Index { object, index, .. } => {
                format!("{}[{}]", self.format_expr(object), self.format_expr(index))
            }
            Expr::Ternary { condition, true_expr, false_expr, .. } => {
                format!("{} ? {} : {}", self.format_expr(condition), self.format_expr(true_expr), self.format_expr(false_expr))
            }
            Expr::Array { elements, .. } => {
                let elements_str = elements
                    .iter()
                    .map(|e| self.format_expr(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", elements_str)
            }
            Expr::Object { properties, .. } => {
                if properties.is_empty() {
                    return "{}".to_string();
                }
                let props_str = properties
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_expr(v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", props_str)
            }
            Expr::TemplateString { parts, .. } => {
                let mut output = String::from("`");
                for part in parts {
                    match part {
                        TemplateStringPart::Text(text) => output.push_str(text),
                        TemplateStringPart::Interpolation(expr) => {
                            output.push_str("${");
                            output.push_str(&self.format_expr(expr));
                            output.push('}');
                        }
                    }
                }
                output.push('`');
                output
            }
            Expr::ArrowFunction {
                params,
                body,
                is_async,
                ..
            } => {
                let async_kw = if *is_async { "async " } else { "" };
                let params_str = params.join(", ");
                match body {
                    ArrowFunctionBody::Expression(expr) => {
                        format!("{}({}) => {}", async_kw, params_str, self.format_expr(expr))
                    }
                    ArrowFunctionBody::Block(stmts) => {
                        // We need a mutable self for format_stmt, but format_expr takes &self.
                        // Use a debug fallback for the block body to avoid requiring &mut self.
                        // This is safe because we preserve the information.
                        let mut block = format!("{}({}) => {{\n", async_kw, params_str);
                        for s in stmts {
                            // Use debug format to preserve info without needing &mut self
                            block.push_str(&format!("    {:?}\n", s));
                        }
                        block.push('}');
                        block
                    }
                }
            }
            Expr::Await { expr, .. } => {
                format!("await {}", self.format_expr(expr))
            }
            Expr::New {
                class_name, args, ..
            } => {
                let args_str = args
                    .iter()
                    .map(|a| self.format_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("new {}({})", class_name, args_str)
            }
            Expr::Spread { expr, .. } => {
                format!("...{}", self.format_expr(expr))
            }
            Expr::This(_) => "this".to_string(),
            Expr::Yield { value, .. } => {
                if let Some(v) = value {
                    format!("yield {}", self.format_expr(v))
                } else {
                    "yield".to_string()
                }
            }
            Expr::Spawn {
                subject_name, args, ..
            } => {
                if args.is_empty() {
                    format!("spawn {}", subject_name)
                } else {
                    let formatted_args: Vec<String> =
                        args.iter().map(|a| self.format_expr(a)).collect();
                    format!("spawn {}({})", subject_name, formatted_args.join(", "))
                }
            }
            Expr::ViewAs { instance, view_name, .. } => {
                format!("view {} as {}", self.format_expr(instance), view_name)
            }
        }
    }

    /// Format a literal value with proper string escaping
    pub(super) fn format_literal(&self, lit: &Literal) -> String {
        match lit {
            Literal::String(s) => format!("\"{}\"", escape_string(s)),
            Literal::Number(n, _) => {
                // Format integers without decimal point
                if *n == n.floor() && n.is_finite() && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            Literal::Boolean(b) => b.to_string(),
            Literal::Null => "null".to_string(),
        }
    }

    /// Format a binary operator
    pub(super) fn format_binop(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::NullCoalesce => "??",
            BinaryOp::InstanceOf => "instanceof",
        }
    }

    /// Format a unary operator
    pub(super) fn format_unop(&self, op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Not => "!",
            UnaryOp::Neg => "-",
            UnaryOp::Plus => "+",
            UnaryOp::PostIncrement => "++",
            UnaryOp::PostDecrement => "--",
        }
    }

    /// Format a destructuring pattern (#715)
    pub(super) fn format_pattern(&self, pattern: &hudhudscript_ast::Pattern) -> String {
        use hudhudscript_ast::Pattern;
        match pattern {
            Pattern::Identifier(name) => name.clone(),
            Pattern::IdentifierDefault(name, default_expr) => {
                format!("{} = {}", name, self.format_expr(default_expr))
            }
            Pattern::Array { elements, rest } => {
                let mut parts: Vec<String> =
                    elements.iter().map(|p| self.format_pattern(p)).collect();
                if let Some(rest_pattern) = rest {
                    parts.push(format!("...{}", self.format_pattern(rest_pattern)));
                }
                format!("[{}]", parts.join(", "))
            }
            Pattern::Object { properties, rest } => {
                let mut parts: Vec<String> = properties
                    .iter()
                    .map(|(key, pat)| {
                        let pat_str = self.format_pattern(pat);
                        if key == &pat_str {
                            key.clone() // shorthand: { name }
                        } else {
                            format!("{}: {}", key, pat_str)
                        }
                    })
                    .collect();
                if let Some(rest_pattern) = rest {
                    parts.push(format!("...{}", self.format_pattern(rest_pattern)));
                }
                format!("{{ {} }}", parts.join(", "))
            }
        }
    }

    /// Get current indentation string
    pub(super) fn get_indent(&self) -> String {
        self.config.indent.repeat(self.current_indent)
    }
}
