use super::*;

impl Compiler {
    pub(super) fn compile_decl_constitution(
        &mut self,
        name: &str,
        description: Option<&String>,
        laws: &[hudhudscript_ast::LawDecl],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut const_obj = HashMap::new();
        let constitution_id = format!("cons.{}", name);
        const_obj.insert("id".to_string(), Value16::string(constitution_id));
        const_obj.insert("name".to_string(), Value16::string(name.to_string()));
        if let Some(desc) = description {
            const_obj.insert("description".to_string(), Value16::string(desc.clone()));
        }
        let laws_array: Vec<Value16> = laws
            .iter()
            .map(|law| {
                let mut law_obj = HashMap::new();
                law_obj.insert("name".to_string(), Value16::string(law.name.clone()));
                law_obj.insert(
                    "description".to_string(),
                    Value16::string(law.description.clone()),
                );
                law_obj.insert(
                    "enforcement_level".to_string(),
                    Value16::string(law.enforcement_level.clone()),
                );
                law_obj.insert(
                    "rules".to_string(),
                    Value16::array(
                        law.rules
                            .iter()
                            .map(|r| Value16::string(expr_to_rule_string(r)))
                            .collect(),
                    ),
                );
                Value16::object(law_obj)
            })
            .collect();
        const_obj.insert("laws".to_string(), Value16::array(laws_array));
        let idx = self.bytecode.add_constant(Value16::object(const_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("constitution", name);
        Ok(())
    }

    pub(super) fn compile_decl_law(
        &mut self,
        name: &str,
        description: &str,
        enforcement_level: &str,
        rules: &[Expr],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut law_obj = HashMap::new();
        law_obj.insert("name".to_string(), Value16::string(name.to_string()));
        law_obj.insert(
            "description".to_string(),
            Value16::string(description.to_string()),
        );
        law_obj.insert(
            "enforcement_level".to_string(),
            Value16::string(enforcement_level.to_string()),
        );
        law_obj.insert(
            "rules".to_string(),
            Value16::array(
                rules
                    .iter()
                    .map(|r| Value16::string(expr_to_rule_string(r)))
                    .collect(),
            ),
        );
        let idx = self.bytecode.add_constant(Value16::object(law_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("law", name);
        Ok(())
    }

    pub(super) fn compile_decl_council(
        &mut self,
        name: &str,
        constitution: &str,
        members: &[hudhudscript_ast::CouncilMemberDecl],
        rules: &[String],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut council_obj = HashMap::new();
        council_obj.insert("name".to_string(), Value16::string(name.to_string()));
        council_obj.insert(
            "constitution".to_string(),
            Value16::string(constitution.to_string()),
        );
        let members_array: Vec<Value16> = members
            .iter()
            .map(|member| {
                let mut member_obj = HashMap::new();
                member_obj.insert(
                    "agent_id".to_string(),
                    Value16::string(member.agent_id.clone()),
                );
                member_obj.insert("role".to_string(), Value16::string(member.role.clone()));
                Value16::object(member_obj)
            })
            .collect();
        council_obj.insert("members".to_string(), Value16::array(members_array));
        council_obj.insert(
            "rules".to_string(),
            Value16::array(rules.iter().map(|r| Value16::string(r.clone())).collect()),
        );
        let idx = self.bytecode.add_constant(Value16::object(council_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("council", name);
        Ok(())
    }

    pub(super) fn compile_decl_rule(
        &mut self,
        name: &str,
        conditions: &[hudhudscript_ast::ConditionDecl],
        actions: &[hudhudscript_ast::ActionDecl],
        priority: u32,
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let effective_priority = if priority != 0 {
            priority
        } else {
            let mut inferred: u32 = 100;
            for action in actions {
                let p = match action.action_type.as_str() {
                    "forbid" => 300u32,
                    "deny" => 200u32,
                    "allow" => 100u32,
                    _ => 0u32,
                };
                if p > inferred {
                    inferred = p;
                }
            }
            inferred
        };
        let mut rule_obj = HashMap::new();
        rule_obj.insert("name".to_string(), Value16::string(name.to_string()));
        rule_obj.insert(
            "priority".to_string(),
            Value16::number(effective_priority as f64),
        );
        let conditions_array: Vec<Value16> = conditions
            .iter()
            .map(|cond| {
                let mut cond_obj = HashMap::new();
                cond_obj.insert(
                    "type".to_string(),
                    Value16::string(cond.condition_type.clone()),
                );
                cond_obj.insert("field".to_string(), Value16::string(cond.field.clone()));
                let val = self.expr_to_const_value(&cond.value);
                cond_obj.insert("value".to_string(), val);
                Value16::object(cond_obj)
            })
            .collect();
        let actions_array: Vec<Value16> = actions
            .iter()
            .map(|action| {
                let mut action_obj = HashMap::new();
                action_obj.insert(
                    "type".to_string(),
                    Value16::string(action.action_type.clone()),
                );
                for (key, value_expr) in &action.params {
                    let val = self.expr_to_const_value(value_expr);
                    action_obj.insert(key.clone(), val);
                }
                Value16::object(action_obj)
            })
            .collect();
        rule_obj.insert("conditions".to_string(), Value16::array(conditions_array));
        rule_obj.insert("actions".to_string(), Value16::array(actions_array));
        let idx = self.bytecode.add_constant(Value16::object(rule_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("rule", name);
        Ok(())
    }

    pub(super) fn compile_decl_governance(
        &mut self,
        name: &str,
        base_type: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let (default_compliance, default_flexibility, default_enforcement) = match base_type {
            "democracy" => (0.8, 0.7, 0.6),
            "monarchy" => (0.9, 0.2, 0.9),
            "republic" => (0.85, 0.6, 0.7),
            "theocracy" => (1.0, 0.1, 1.0),
            "technocracy" => (0.7, 0.8, 0.7),
            "meritocracy" => (0.75, 0.7, 0.65),
            "oligarchy" => (0.6, 0.4, 0.8),
            "anarchy" => (0.0, 1.0, 0.0),
            "parliamentary" => (0.85, 0.65, 0.7),
            "autocracy" => (0.95, 0.1, 0.95),
            "consensus" => (0.9, 0.8, 0.5),
            "hybrid" => (0.7, 0.7, 0.7),
            _ => (0.7, 0.7, 0.7),
        };
        let mut gov_obj = HashMap::new();
        gov_obj.insert("name".to_string(), Value16::string(name.to_string()));
        gov_obj.insert("type".to_string(), Value16::string(base_type.to_string()));
        gov_obj.insert(
            "constitution_compliance".to_string(),
            Value16::number(default_compliance),
        );
        gov_obj.insert(
            "law_flexibility".to_string(),
            Value16::number(default_flexibility),
        );
        gov_obj.insert(
            "rule_enforcement".to_string(),
            Value16::number(default_enforcement),
        );
        for (key, value_expr) in fields {
            let val = self.expr_to_const_value(value_expr);
            gov_obj.insert(key.clone(), val);
        }
        let idx = self.bytecode.add_constant(Value16::object(gov_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("governance", name);
        Ok(())
    }
}
