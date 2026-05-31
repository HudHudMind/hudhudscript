use super::*;

impl Formatter {
    pub(super) fn format_rule(
        &self,
        indent: &str,
        name: &str,
        conditions: &[ConditionDecl],
        actions: &[ActionDecl],
        priority: u32,
    ) -> String {
        let mut output = format!("{}rule {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        output.push_str(&format!("{}priority: {},\n", field_indent, priority));
        if !conditions.is_empty() {
            output.push_str(&format!("{}conditions: [\n", field_indent));
            let inner_indent = format!("{}{}", field_indent, self.config.indent);
            for cond in conditions {
                output.push_str(&format!(
                    "{}{{ type: \"{}\", field: \"{}\", value: {} }},\n",
                    inner_indent,
                    escape_string(&cond.condition_type),
                    escape_string(&cond.field),
                    self.format_expr(&cond.value)
                ));
            }
            output.push_str(&format!("{}],\n", field_indent));
        }
        if !actions.is_empty() {
            output.push_str(&format!("{}actions: [\n", field_indent));
            let inner_indent = format!("{}{}", field_indent, self.config.indent);
            for action in actions {
                let params_str = action
                    .params
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.format_expr(v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                output.push_str(&format!(
                    "{}{{ type: \"{}\", {} }},\n",
                    inner_indent,
                    escape_string(&action.action_type),
                    params_str
                ));
            }
            output.push_str(&format!("{}],\n", field_indent));
        }
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a swarm declaration
    pub(super) fn format_swarm(
        &self,
        indent: &str,
        name: &str,
        agents: &[String],
        strategy: &str,
    ) -> String {
        let mut output = format!("{}swarm {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        output.push_str(&format!("{}strategy: \"{}\",\n", field_indent, strategy));
        if !agents.is_empty() {
            let agents_str = agents
                .iter()
                .map(|a| format!("\"{}\"", escape_string(a)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}agents: [{}],\n", field_indent, agents_str));
        }
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a community declaration
    pub(super) fn format_community(
        &self,
        indent: &str,
        name: &str,
        members: &[String],
        councils: &[String],
        culture: &CultureDecl,
    ) -> String {
        let mut output = format!("{}community {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        if !members.is_empty() {
            let members_str = members
                .iter()
                .map(|m| format!("\"{}\"", escape_string(m)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}members: [{}],\n", field_indent, members_str));
        }
        if !councils.is_empty() {
            let councils_str = councils
                .iter()
                .map(|c| format!("\"{}\"", escape_string(c)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}councils: [{}],\n", field_indent, councils_str));
        }
        output.push_str(&format!("{}culture: {{\n", field_indent));
        let inner_indent = format!("{}{}", field_indent, self.config.indent);
        if !culture.values.is_empty() {
            let values_str = culture
                .values
                .iter()
                .map(|v| format!("\"{}\"", escape_string(v)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}values: [{}],\n", inner_indent, values_str));
        }
        if !culture.norms.is_empty() {
            let norms_str = culture
                .norms
                .iter()
                .map(|n| format!("\"{}\"", escape_string(n)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}norms: [{}],\n", inner_indent, norms_str));
        }
        output.push_str(&format!(
            "{}communication_style: \"{}\",\n",
            inner_indent, culture.communication_style
        ));
        output.push_str(&format!("{}}},\n", field_indent));
        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Format a protocol declaration
    pub(super) fn format_protocol(
        &self,
        indent: &str,
        name: &str,
        execution: &Option<String>,
        governance: &Option<String>,
        timeout: &Option<f64>,
        session: &[(String, Expr)],
    ) -> String {
        let mut output = format!("{}protocol {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        if let Some(exec) = execution {
            output.push_str(&format!("{}execution: \"{}\",\n", field_indent, exec));
        }
        if let Some(gov) = governance {
            output.push_str(&format!("{}governance: {},\n", field_indent, gov));
        }
        if let Some(t) = timeout {
            output.push_str(&format!("{}timeout: {},\n", field_indent, t));
        }
        for (key, value) in session {
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

    /// Format a governance declaration
    pub(super) fn format_governance(
        &self,
        indent: &str,
        name: &str,
        base_type: &str,
        fields: &[(String, Expr)],
    ) -> String {
        let mut output = format!("{}governance {} : {} {{\n", indent, name, base_type);
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

    /// Format a strategy declaration
    #[allow(clippy::too_many_arguments)]
    pub(super) fn format_strategy(
        &self,
        indent: &str,
        name: &str,
        execution: &Option<String>,
        governance: &Option<String>,
        timeout: &Option<f64>,
        permissions: &[String],
        realm: &Option<String>,
        session: &[(String, Expr)],
    ) -> String {
        let mut output = format!("{}strategy {} {{\n", indent, name);
        let field_indent = format!("{}{}", indent, self.config.indent);
        if let Some(exec) = execution {
            output.push_str(&format!("{}execution: \"{}\",\n", field_indent, exec));
        }
        if let Some(gov) = governance {
            output.push_str(&format!("{}governance: {},\n", field_indent, gov));
        }
        if let Some(t) = timeout {
            output.push_str(&format!("{}timeout: {},\n", field_indent, t));
        }
        if !permissions.is_empty() {
            let perms_str = permissions
                .iter()
                .map(|p| format!("\"{}\"", escape_string(p)))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("{}permissions: [{}],\n", field_indent, perms_str));
        }
        if let Some(r) = realm {
            output.push_str(&format!(
                "{}realm: \"{}\",\n",
                field_indent,
                escape_string(r)
            ));
        }
        for (key, value) in session {
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
}
