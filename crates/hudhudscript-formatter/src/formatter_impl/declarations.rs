use super::*;

impl Formatter {
    /// Format a VarDecl
    pub(super) fn format_var_decl(&self, decl: &VarDecl) -> String {
        let indent = self.get_indent();
        let keyword = if decl.is_const { "const" } else { "var" };
        let type_str = if let Some(ref ty) = decl.type_annotation {
            format!(": {}", self.format_type_annotation(ty))
        } else {
            String::new()
        };
        if let Some(ref init) = decl.initializer {
            format!(
                "{}{} {}{} = {};\n",
                indent,
                keyword,
                decl.name,
                type_str,
                self.format_expr(init)
            )
        } else {
            format!("{}{} {}{};\n", indent, keyword, decl.name, type_str)
        }
    }

    /// Format a type annotation
    pub(super) fn format_type_annotation(&self, ty: &hudhudscript_ast::TypeAnnotation) -> String {
        match ty {
            hudhudscript_ast::TypeAnnotation::String => "string".to_string(),
            hudhudscript_ast::TypeAnnotation::Number => "number".to_string(),
            hudhudscript_ast::TypeAnnotation::Boolean => "boolean".to_string(),
            hudhudscript_ast::TypeAnnotation::Null => "null".to_string(),
            hudhudscript_ast::TypeAnnotation::Any => "any".to_string(),
            hudhudscript_ast::TypeAnnotation::Tool => "tool".to_string(),
            hudhudscript_ast::TypeAnnotation::Resource => "resource".to_string(),
            hudhudscript_ast::TypeAnnotation::Server => "server".to_string(),
            hudhudscript_ast::TypeAnnotation::Generic(name) => name.clone(),
            hudhudscript_ast::TypeAnnotation::Array(inner) => {
                format!("{}[]", self.format_type_annotation(inner))
            }
            hudhudscript_ast::TypeAnnotation::Union(types) => types
                .iter()
                .map(|t| self.format_type_annotation(t))
                .collect::<Vec<_>>()
                .join(" | "),
            hudhudscript_ast::TypeAnnotation::Parameterized { base, args } => {
                let args_str = args
                    .iter()
                    .map(|t| self.format_type_annotation(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", self.format_type_annotation(base), args_str)
            }
        }
    }

    /// Format a switch statement
    pub(super) fn format_switch(
        &mut self,
        indent: &str,
        value: &Expr,
        cases: &[SwitchCase],
        default: &Option<Vec<Stmt>>,
    ) -> String {
        let mut output = format!("{}switch ({}) {{\n", indent, self.format_expr(value));
        self.current_indent += 1;
        let case_indent = self.get_indent();
        for case in cases {
            output.push_str(&format!(
                "{}case {}:\n",
                case_indent,
                self.format_expr(&case.value)
            ));
            self.current_indent += 1;
            for s in &case.body {
                output.push_str(&self.format_stmt(s));
            }
            self.current_indent -= 1;
        }
        if let Some(default_body) = default {
            output.push_str(&format!("{}default:\n", case_indent));
            self.current_indent += 1;
            for s in default_body {
                output.push_str(&self.format_stmt(s));
            }
            self.current_indent -= 1;
        }
        self.current_indent -= 1;
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a try-catch-finally statement
    pub(super) fn format_try(
        &mut self,
        indent: &str,
        try_block: &Stmt,
        catch_clause: &Option<CatchClause>,
        finally_block: &Option<Box<Stmt>>,
    ) -> String {
        let mut output = format!("{}try {{\n", indent);
        self.current_indent += 1;
        output.push_str(&self.format_stmt(try_block));
        self.current_indent -= 1;

        if let Some(catch) = catch_clause {
            output.push_str(&format!("{}}} catch ({}) {{\n", indent, catch.param));
            self.current_indent += 1;
            output.push_str(&self.format_stmt(&catch.body));
            self.current_indent -= 1;
        }

        if let Some(finally) = finally_block {
            output.push_str(&format!("{}}} finally {{\n", indent));
            self.current_indent += 1;
            output.push_str(&self.format_stmt(finally));
            self.current_indent -= 1;
        }

        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format an import statement (Stmt::Import variant)
    pub(super) fn format_import_stmt(
        &self,
        indent: &str,
        path: &str,
        imports: &ImportKind,
    ) -> String {
        match imports {
            ImportKind::Named(names) => {
                let names_str = names.join(", ");
                format!(
                    "{}import {{ {} }} from \"{}\";\n",
                    indent,
                    names_str,
                    escape_string(path)
                )
            }
            ImportKind::Default(name) => {
                format!(
                    "{}import {} from \"{}\";\n",
                    indent,
                    name,
                    escape_string(path)
                )
            }
            ImportKind::Wildcard(alias) => {
                format!(
                    "{}import * as {} from \"{}\";\n",
                    indent,
                    alias,
                    escape_string(path)
                )
            }
        }
    }

    /// Format a class declaration
    pub(super) fn format_class(&mut self, indent: &str, class_decl: &ClassDecl) -> String {
        let parent_str = if let Some(ref parent) = class_decl.parent {
            format!(" <- {}", parent)
        } else {
            String::new()
        };
        let mut output = format!("{}class {}{} {{\n", indent, class_decl.name, parent_str);
        self.current_indent += 1;
        for member in &class_decl.members {
            output.push_str(&self.format_class_member(member));
        }
        self.current_indent -= 1;
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a class member
    pub(super) fn format_class_member(&mut self, member: &ClassMember) -> String {
        let indent = self.get_indent();
        match member {
            ClassMember::Field {
                access,
                is_static,
                name,
                initializer,
                ..
            } => {
                let access_str = self.format_access_modifier(access);
                let static_str = if *is_static { "static " } else { "" };
                if let Some(init) = initializer {
                    format!(
                        "{}{}{}var {} = {};\n",
                        indent,
                        access_str,
                        static_str,
                        name,
                        self.format_expr(init)
                    )
                } else {
                    format!("{}{}{}var {};\n", indent, access_str, static_str, name)
                }
            }
            ClassMember::Method {
                access,
                is_static,
                name,
                params,
                body,
                ..
            } => {
                let access_str = self.format_access_modifier(access);
                let static_str = if *is_static { "static " } else { "" };
                let params_str = self.format_params(params);
                let mut output = format!(
                    "{}{}{}function {}({}) {{\n",
                    indent, access_str, static_str, name, params_str
                );
                self.current_indent += 1;
                for s in body {
                    output.push_str(&self.format_stmt(s));
                }
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
            ClassMember::Constructor { params, body, .. } => {
                let params_str = self.format_params(params);
                let mut output = format!("{}constructor({}) {{\n", indent, params_str);
                self.current_indent += 1;
                for s in body {
                    output.push_str(&self.format_stmt(s));
                }
                self.current_indent -= 1;
                output.push_str(&format!("{}}}\n", indent));
                output
            }
        }
    }

    /// Format access modifier
    pub(super) fn format_access_modifier(&self, access: &AccessModifier) -> &'static str {
        match access {
            AccessModifier::Public => "public ",
            AccessModifier::Private => "private ",
            AccessModifier::Protected => "protected ",
        }
    }

    /// Format function parameters with optional type annotations
    pub(super) fn format_params(&self, params: &[Param]) -> String {
        params
            .iter()
            .map(|p| {
                if let Some(ref ty) = p.type_annotation {
                    format!("{}: {}", p.name, self.format_type_annotation(ty))
                } else {
                    p.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Format an MCP server declaration
    pub(super) fn format_mcp_server(&self, indent: &str, decl: &McpServerDecl) -> String {
        let mut output = format!("{}mcp server {} {{\n", indent, decl.name);
        let field_indent = format!("{}{}", indent, self.config.indent);

        let transport_str = match decl.config.transport {
            hudhudscript_ast::TransportType::Stdio => "stdio",
            hudhudscript_ast::TransportType::SSE => "sse",
        };
        output.push_str(&format!(
            "{}transport: \"{}\";\n",
            field_indent, transport_str
        ));

        if let Some(ref cmd) = decl.config.command {
            output.push_str(&format!(
                "{}command: \"{}\";\n",
                field_indent,
                escape_string(cmd)
            ));
        }
        if !decl.config.args.is_empty() {
            let args_str = decl
                .config
                .args
                .iter()
                .map(|a| format!("\"{}\"", escape_string(a)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}args: [{}];\n", field_indent, args_str));
        }
        if let Some(ref url) = decl.config.url {
            output.push_str(&format!(
                "{}url: \"{}\";\n",
                field_indent,
                escape_string(url)
            ));
        }
        if let Some(ref auth) = decl.config.auth {
            let auth_type_str = match auth.auth_type {
                hudhudscript_ast::AuthType::Bearer => "bearer",
                hudhudscript_ast::AuthType::Basic => "basic",
                hudhudscript_ast::AuthType::ApiKey => "apikey",
            };
            output.push_str(&format!("{}auth: \"{}\";\n", field_indent, auth_type_str));
            if let Some(ref token) = auth.token {
                output.push_str(&format!(
                    "{}token: \"{}\";\n",
                    field_indent,
                    escape_string(token)
                ));
            }
        }

        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a match statement
    pub(super) fn format_match(&mut self, indent: &str, value: &Expr, arms: &[MatchArm]) -> String {
        let mut output = format!("{}match {} {{\n", indent, self.format_expr(value));
        self.current_indent += 1;
        let arm_indent = self.get_indent();
        for arm in arms {
            let pattern_str = self.format_match_pattern(&arm.pattern);
            let guard_str = if let Some(guard) = &arm.guard {
                format!(" if {}", self.format_expr(guard))
            } else {
                String::new()
            };
            output.push_str(&format!(
                "{}{}{} => {{\n",
                arm_indent, pattern_str, guard_str
            ));
            self.current_indent += 1;
            for s in &arm.body {
                output.push_str(&self.format_stmt(s));
            }
            self.current_indent -= 1;
            output.push_str(&format!("{}}}\n", arm_indent));
        }
        self.current_indent -= 1;
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a match pattern
    pub(super) fn format_match_pattern(&self, pattern: &MatchPattern) -> String {
        match pattern {
            MatchPattern::Wildcard => "_".to_string(),
            MatchPattern::Literal(lit) => self.format_literal(lit),
            MatchPattern::Identifier(name) => name.clone(),
            MatchPattern::EnumVariant {
                enum_name,
                variant,
                binding,
            } => {
                if let Some(bind) = binding {
                    format!("{}::{}({})", enum_name, variant, bind)
                } else {
                    format!("{}::{}", enum_name, variant)
                }
            }
            MatchPattern::Or(sub_patterns) => sub_patterns
                .iter()
                .map(|p| self.format_match_pattern(p))
                .collect::<Vec<_>>()
                .join(" | "),
        }
    }
}
