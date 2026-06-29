use super::*;

impl Formatter {
    pub(super) fn format_enum_decl(
        &self,
        indent: &str,
        name: &str,
        variants: &[EnumVariant],
    ) -> String {
        let mut output = format!("{}enum {} {{\n", indent, name);
        let variant_indent = format!("{}{}", indent, self.config.indent);
        for variant in variants {
            if variant.fields.is_empty() {
                output.push_str(&format!("{}{},\n", variant_indent, variant.name));
            } else {
                let fields_str = variant.fields.join(", ");
                output.push_str(&format!(
                    "{}{}({}),\n",
                    variant_indent, variant.name, fields_str
                ));
            }
        }
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a declaration - handles ALL Decl variants without destructive catch-all
    pub(super) fn format_decl(&mut self, decl: &Decl) -> String {
        let indent = self.get_indent();

        match decl {
            Decl::Agent { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "agent", name, fields)
            }
            Decl::Action { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "action", name, fields)
            }
            Decl::Tool { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "tool", name, fields)
            }
            Decl::Resource { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "resource", name, fields)
            }
            Decl::Import { module, alias, .. } => {
                if let Some(a) = alias {
                    format!("{}use {} as {};\n", indent, module, a)
                } else {
                    format!("{}use {};\n", indent, module)
                }
            }
            Decl::Constitution {
                name,
                description,
                laws,
                ..
            } => self.format_constitution(&indent, name, description, laws),
            Decl::Law {
                name,
                description,
                enforcement_level,
                rules,
                ..
            } => self.format_law(&indent, name, description, enforcement_level, rules),
            Decl::Council {
                name,
                constitution,
                members,
                rules,
                ..
            } => self.format_council(&indent, name, constitution, members, rules),
            Decl::Rule {
                name,
                conditions,
                actions,
                priority,
                ..
            } => self.format_rule(&indent, name, conditions, actions, *priority),
            Decl::Swarm {
                name,
                agents,
                strategy,
                ..
            } => self.format_swarm(&indent, name, agents, strategy),
            Decl::Community {
                name,
                members,
                councils,
                culture,
                ..
            } => self.format_community(&indent, name, members, councils, culture),
            Decl::Provider { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "provider", name, fields)
            }
            Decl::Protocol {
                name,
                execution,
                governance,
                timeout,
                session,
                ..
            } => self.format_protocol(&indent, name, execution, governance, timeout, session),
            Decl::Governance {
                name,
                base_type,
                fields,
                ..
            } => self.format_governance(&indent, name, base_type, fields),
            Decl::Strategy {
                name,
                execution,
                governance,
                timeout,
                permissions,
                realm,
                session,
                ..
            } => self.format_strategy(
                &indent,
                name,
                execution,
                governance,
                timeout,
                permissions,
                realm,
                session,
            ),

            // SOP declarations — basic formatting
            Decl::Subject { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "subject", name, fields)
            }
            Decl::Relation {
                subject_a,
                subject_b,
                fields,
                ..
            } => {
                let header = format!("{}relation {} <-> {} {{\n", indent, subject_a, subject_b);
                let mut body = String::new();
                for (key, val) in fields {
                    body.push_str(&format!(
                        "{}    {}: {},\n",
                        indent,
                        key,
                        self.format_expr(val)
                    ));
                }
                format!("{}{}{}}}", header, body, indent)
            }
            Decl::Role {
                name,
                capabilities,
                fields,
                ..
            } => {
                let mut out = format!("{}role {} {{\n", indent, name);
                for cap in capabilities {
                    out.push_str(&format!("{}    can {},\n", indent, cap));
                }
                for (key, val) in fields {
                    out.push_str(&format!(
                        "{}    {}: {},\n",
                        indent,
                        key,
                        self.format_expr(val)
                    ));
                }
                out.push_str(&format!("{}}}", indent));
                out
            }
            Decl::Store { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "store", name, fields)
            }
            Decl::Effect {
                event_name, body, ..
            } => {
                let header = format!("{}effect on {} {{\n", indent, event_name);
                self.current_indent += 1;
                let mut stmts = String::new();
                for stmt in body {
                    stmts.push_str(&self.format_stmt(stmt));
                }
                self.current_indent -= 1;
                format!("{}{}{}}}", header, stmts, indent)
            }
            Decl::Compose { base_subject, rules, .. } => {
                let header = format!("{}compose {} {{\n", indent, base_subject);
                let mut body = String::new();
                for rule in rules {
                    let mode_str = match &rule.mode {
                        hudhudscript_ast::ComposeMode::Combine(subjects) => format!("combine [{}]", subjects.join(", ")),
                        hudhudscript_ast::ComposeMode::Override(s) => format!("override {}", s),
                        hudhudscript_ast::ComposeMode::Before(s) => format!("before {}", s),
                        hudhudscript_ast::ComposeMode::After(s) => format!("after {}", s),
                    };
                    body.push_str(&format!("{}  on {}: {}\n", indent, rule.ability_name, mode_str));
                }
                format!("{}{}{}}}", header, body, indent)
            }

            // Issue #285: dedicated domain-specific declarations
            Decl::Entity { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "entity", name, fields)
            }
            Decl::StateMachine { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "statemachine", name, fields)
            }
            Decl::Event { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "event", name, fields)
            }
            Decl::Contract { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "contract", name, fields)
            }
            Decl::Treaty { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "treaty", name, fields)
            }

            // Issue #278: Music DSL declarations
            Decl::Music {
                kind, name, fields, ..
            } => self.format_decl_with_fields(&indent, kind, name, fields),

            Decl::AgentAction { name, .. } => {
                format!("{indent}action {name} {{ ... }}")
            }

            Decl::Ability { name, .. } => {
                format!("{indent}on {name}(...) {{ ... }}")
            }

            // UI and Deploy declarations
            Decl::UiApp { name, .. } => self.format_decl_with_fields(&indent, "ui", name, &[]),
            Decl::Deploy { name, fields, .. } => {
                self.format_decl_with_fields(&indent, "deploy", name, fields)
            }
            _ => String::new(),
        }
    }

    /// Format a declaration with name and field pairs (agent, task, tool, resource, provider)
    pub(super) fn format_decl_with_fields(
        &self,
        indent: &str,
        keyword: &str,
        name: &str,
        fields: &[(String, Expr)],
    ) -> String {
        if fields.is_empty() {
            return format!("{}{} {} {{}}\n", indent, keyword, name);
        }
        let mut output = format!("{}{} {} {{\n", indent, keyword, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        for (key, value) in fields {
            output.push_str(&format!(
                "{}{}: {},\n",
                field_indent,
                key,
                self.format_expr(value)
            ));
        }
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a constitution declaration
    pub(super) fn format_constitution(
        &mut self,
        indent: &str,
        name: &str,
        description: &Option<String>,
        laws: &[LawDecl],
    ) -> String {
        let mut output = format!("{}constitution {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        if let Some(desc) = description {
            output.push_str(&format!(
                "{}description: \"{}\",\n",
                field_indent,
                escape_string(desc)
            ));
        }
        for law in laws {
            output.push_str(&format!("{}law {} {{\n", field_indent, law.name));
            let inner_indent = format!("{}{}", field_indent, self.config.indent);
            output.push_str(&format!(
                "{}description: \"{}\",\n",
                inner_indent,
                escape_string(&law.description)
            ));
            output.push_str(&format!(
                "{}enforcement_level: \"{}\",\n",
                inner_indent, law.enforcement_level
            ));
            if !law.rules.is_empty() {
                let rules_str = law
                    .rules
                    .iter()
                    .map(|r| self.format_expr(r))
                    .collect::<Vec<_>>()
                    .join(", ");
                output.push_str(&format!("{}rules: [{}],\n", inner_indent, rules_str));
            }
            output.push_str(&format!("{}}}\n", field_indent));
        }
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a law declaration
    pub(super) fn format_law(
        &self,
        indent: &str,
        name: &str,
        description: &str,
        enforcement_level: &str,
        rules: &[hudhudscript_ast::Expr],
    ) -> String {
        let mut output = format!("{}law {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        output.push_str(&format!(
            "{}description: \"{}\",\n",
            field_indent,
            escape_string(description)
        ));
        output.push_str(&format!(
            "{}enforcement_level: \"{}\",\n",
            field_indent, enforcement_level
        ));
        if !rules.is_empty() {
            // Issue #474: Format Expr rules using the expression formatter
            let rules_str = rules
                .iter()
                .map(|r| self.format_expr(r))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}rules: [{}],\n", field_indent, rules_str));
        }
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a council declaration
    pub(super) fn format_council(
        &self,
        indent: &str,
        name: &str,
        constitution: &str,
        members: &[CouncilMemberDecl],
        rules: &[String],
    ) -> String {
        let mut output = format!("{}council {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        output.push_str(&format!(
            "{}constitution: \"{}\",\n",
            field_indent,
            escape_string(constitution)
        ));
        if !members.is_empty() {
            output.push_str(&format!("{}members: [\n", field_indent));
            let inner_indent = format!("{}{}", field_indent, self.config.indent);
            for member in members {
                output.push_str(&format!(
                    "{}{{ agent: \"{}\", role: \"{}\" }},\n",
                    inner_indent,
                    escape_string(&member.agent_id),
                    escape_string(&member.role)
                ));
            }
            output.push_str(&format!("{}],\n", field_indent));
        }
        if !rules.is_empty() {
            let rules_str = rules
                .iter()
                .map(|r| format!("\"{}\"", escape_string(r)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}rules: [{}],\n", field_indent, rules_str));
        }
        output.push_str(&format!("{}}}\n", indent));
        output
    }
}
