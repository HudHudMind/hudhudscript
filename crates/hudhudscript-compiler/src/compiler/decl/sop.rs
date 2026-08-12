use super::*;

impl Compiler {
    pub(super) fn compile_decl_subject(
        &mut self,
        name: &str,
        decorators: &[hudhudscript_ast::Decorator],
        of_subject: Option<String>,
        roles: &[String],
        states: &[(String, Expr)],
        capabilities: &[String],
        ability_defs: &[hudhudscript_ast::SubjectAbilityDef],
        intents: &[String],
        uses: &[(String, Option<String>)],
        memory: &[(String, Expr)],
        perception: &[(String, Expr)],
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut subject_obj = hudhudscript_bytecode::ObjMap::default();
        subject_obj.insert("name".to_string(), Value16::string(name.to_string()));
        subject_obj.insert("__type".to_string(), Value16::string("subject".to_string()));
        if let Some(ref base) = of_subject {
            subject_obj.insert("__of_subject".to_string(), Value16::string(base.clone()));
        }

        if !decorators.is_empty() {
            let dec_arr: Vec<Value16> = decorators
                .iter()
                .map(|d| {
                    let mut dec_obj = hudhudscript_bytecode::ObjMap::default();
                    dec_obj.insert("name".to_string(), Value16::string(d.name.clone()));
                    if !d.params.is_empty() {
                        let mut params_obj = hudhudscript_bytecode::ObjMap::default();
                        for (k, v_expr) in &d.params {
                            let val = self.expr_to_const_value(v_expr);
                            params_obj.insert(k.clone(), val);
                        }
                        dec_obj.insert("params".to_string(), Value16::object(params_obj));
                    }
                    Value16::object(dec_obj)
                })
                .collect();
            subject_obj.insert("decorators".to_string(), Value16::array(dec_arr));
            for d in decorators {
                match d.name.as_str() {
                    "ai" | "payment" | "cloud" | "hudhud" | "custom" => {
                        subject_obj.insert(format!("__{}", d.name), Value16::bool_(true));
                    }
                    _ => {}
                }
            }
        }

        if !roles.is_empty() {
            subject_obj.insert(
                "roles".to_string(),
                Value16::array(roles.iter().map(|r| Value16::string(r.clone())).collect()),
            );
        }

        let all_capabilities: Vec<String> = capabilities.to_vec();
        if !all_capabilities.is_empty() {
            subject_obj.insert(
                "can".to_string(),
                Value16::array(
                    all_capabilities
                        .iter()
                        .map(|c| Value16::string(c.clone()))
                        .collect(),
                ),
            );
        }

        if !states.is_empty() {
            let mut state_obj = hudhudscript_bytecode::ObjMap::default();
            for (state_name, state_expr) in states {
                let val = self.expr_to_const_value(state_expr);
                state_obj.insert(state_name.clone(), val.clone());
                subject_obj.insert(state_name.clone(), val);
            }
            subject_obj.insert("state".to_string(), Value16::object(state_obj));
        }

        if !intents.is_empty() {
            subject_obj.insert(
                "intents".to_string(),
                Value16::array(intents.iter().map(|i| Value16::string(i.clone())).collect()),
            );
        }

        if !uses.is_empty() {
            let uses_arr: Vec<Value16> = uses
                .iter()
                .map(|(provider, via)| {
                    let mut u = hudhudscript_bytecode::ObjMap::default();
                    u.insert("provider".to_string(), Value16::string(provider.clone()));
                    if let Some(channel) = via {
                        u.insert("via".to_string(), Value16::string(channel.clone()));
                    }
                    Value16::object(u)
                })
                .collect();
            subject_obj.insert("uses".to_string(), Value16::array(uses_arr));
            for (provider, via) in uses {
                let val = if let Some(channel) = via {
                    Value16::string(channel.clone())
                } else {
                    Value16::bool_(true)
                };
                subject_obj.insert(provider.clone(), val);
            }
        }

        if !memory.is_empty() {
            let mut mem_obj = hudhudscript_bytecode::ObjMap::default();
            for (k, v_expr) in memory {
                let val = self.expr_to_const_value(v_expr);
                mem_obj.insert(k.clone(), val.clone());
                subject_obj.insert(k.clone(), val);
            }
            subject_obj.insert("memory".to_string(), Value16::object(mem_obj));
        }

        if !perception.is_empty() {
            let mut perc_obj = hudhudscript_bytecode::ObjMap::default();
            for (k, v_expr) in perception {
                let val = self.expr_to_const_value(v_expr);
                perc_obj.insert(k.clone(), val.clone());
                subject_obj.insert(k.clone(), val);
            }
            subject_obj.insert("perception".to_string(), Value16::object(perc_obj));
        }

        for (key, value_expr) in fields {
            let val = self.expr_to_const_value(value_expr);
            subject_obj.insert(key.clone(), val);
        }

        // SOP0003: role enforcement
        for role_name in roles {
            if let Some(required) = self.known_roles.get(role_name) {
                let defined: std::collections::HashSet<&str> =
                    ability_defs.iter().map(|d| d.name.as_str()).collect();
                for req in required {
                    if !defined.contains(req.as_str()) {
                        eprintln!(
                            "[SOP] Warning: subject '{}' has role '{}' but missing ability '{}' (role requires: [{}])",
                            name, role_name, req, required.join(", ")
                        );
                    }
                }
            } else {
                eprintln!(
                    "[SOP] Warning: subject '{}' references unknown role '{}'",
                    name, role_name
                );
            }
        }

        let idx = self.bytecode.add_constant(Value16::object(subject_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_move(255, tr ); }
        self.emit_decl_store("subject", name, 255);

        // Compile ability definitions inside subject body as subject-scoped chunks
        for cap_def in ability_defs {
            let chunk_name = format!("ability::{}::{}", name, cap_def.name);
            let chunk = self.compile_function_body(cap_def.params.clone(), &cap_def.body)?;
            let chunk_arc = Arc::new(chunk);
            self.bytecode
                .add_function(chunk_name, chunk_arc);
        }

        Ok(())
    }

    pub(super) fn compile_decl_role(
        &mut self,
        name: &str,
        capabilities: &[String],
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.known_roles.insert(name.to_string(), capabilities.to_vec());
        use std::collections::HashMap;
        let mut role_obj = hudhudscript_bytecode::ObjMap::default();
        role_obj.insert("__type".to_string(), Value16::string("role".to_string()));
        role_obj.insert("name".to_string(), Value16::string(name.to_string()));
        if !capabilities.is_empty() {
            role_obj.insert(
                "can".to_string(),
                Value16::array(
                    capabilities
                        .iter()
                        .map(|c| Value16::string(c.clone()))
                        .collect(),
                ),
            );
        }
        for (key, value_expr) in fields {
            let val = self.expr_to_const_value(value_expr);
            role_obj.insert(key.clone(), val);
        }
        let idx = self.bytecode.add_constant(Value16::object(role_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_move(255, tr ); }
        self.emit_decl_store("role", name, 255);
        Ok(())
    }

    pub(super) fn compile_decl_relation(
        &mut self,
        subject_a: &str,
        subject_b: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let rel_name = format!("{}_{}", subject_a, subject_b);
        let mut rel_obj = hudhudscript_bytecode::ObjMap::default();
        rel_obj.insert(
            "__type".to_string(),
            Value16::string("relation".to_string()),
        );
        rel_obj.insert(
            "subject_a".to_string(),
            Value16::string(subject_a.to_string()),
        );
        rel_obj.insert(
            "subject_b".to_string(),
            Value16::string(subject_b.to_string()),
        );
        for (key, value_expr) in fields {
            let val = self.expr_to_const_value(value_expr);
            rel_obj.insert(key.clone(), val);
        }
        let idx = self.bytecode.add_constant(Value16::object(rel_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_move(255, tr ); }
        self.emit_decl_store("relation", &rel_name, 255);
        Ok(())
    }

    pub(super) fn compile_decl_effect(
        &mut self,
        event_name: &str,
        params: &[String],
        body: &[hudhudscript_ast::Stmt],
    ) -> CompileResult<()> {
        use std::collections::HashMap;

        // P2.3: effect on — pure state mutation handler. Should NOT use `self`.
        if params.first().map(|s| s.as_str()) == Some("self") {
            return Err(compile_codes::type_error(format!(
                "effect '{}': `self` parameter is forbidden in effects. Use `on {}(self, ...)` instead.",
                event_name, event_name
            )));
        }

        let chunk_name = format!("effect::{}", event_name);
        let chunk = self.compile_function_body(params.to_vec(), body)?;
        let chunk_arc = Arc::new(chunk);
        self.bytecode
            .add_function(chunk_name.clone(), Arc::clone(&chunk_arc));
        // Also register under the event name so it can be called directly
        self.bytecode
            .add_function(event_name.to_string(), chunk_arc);
        let func_val = Value16::function(FunctionData {
            name: chunk_name.clone(),
            params: params.to_vec(),
            chunk_name: chunk_name.clone(),
                chunk_sym: hudhudscript_bytecode::interner::intern(&chunk_name.clone()).0,
            captures: Default::default(),
        });

        let mut effect_obj = hudhudscript_bytecode::ObjMap::default();
        effect_obj.insert("__type".to_string(), Value16::string("effect".to_string()));
        effect_obj.insert("event".to_string(), Value16::string(event_name.to_string()));
        effect_obj.insert("handler".to_string(), func_val.clone());
        let idx = self.bytecode.add_constant(Value16::object(effect_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_move(255, tr ); }
        let effect_store_name = format!("effect_on_{}", event_name);
        self.emit_decl_store("effect", &effect_store_name, 255);
        Ok(())
    }

    /// SOP0007: Compile composition rules into DeclStore constants
    pub(super) fn compile_decl_compose(
        &mut self,
        base_subject: &str,
        rules: &[hudhudscript_ast::ComposeRule],
        field_rules: &[(String, hudhudscript_ast::FieldCorrespondence)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        use hudhudscript_ast::ComposeMode;
        let mut compose_obj = hudhudscript_bytecode::ObjMap::default();
        compose_obj.insert("__type".to_string(), Value16::string("compose".to_string()));
        compose_obj.insert("base".to_string(), Value16::string(base_subject.to_string()));
        let mut rules_arr = Vec::new();
        for rule in rules {
            let mut rule_obj = hudhudscript_bytecode::ObjMap::default();
            rule_obj.insert("ability".to_string(), Value16::string(rule.ability_name.clone()));
            match &rule.mode {
                ComposeMode::Combine(subjects) => {
                    rule_obj.insert("mode".to_string(), Value16::string("combine".to_string()));
                    rule_obj.insert("subjects".to_string(), Value16::array(
                        subjects.iter().map(|s| Value16::string(s.clone())).collect()
                    ));
                }
                ComposeMode::Override(subject) => {
                    rule_obj.insert("mode".to_string(), Value16::string("override".to_string()));
                    rule_obj.insert("subject".to_string(), Value16::string(subject.clone()));
                }
                ComposeMode::Before(subject) => {
                    rule_obj.insert("mode".to_string(), Value16::string("before".to_string()));
                    rule_obj.insert("subject".to_string(), Value16::string(subject.clone()));
                }
                ComposeMode::After(subject) => {
                    rule_obj.insert("mode".to_string(), Value16::string("after".to_string()));
                    rule_obj.insert("subject".to_string(), Value16::string(subject.clone()));
                }
            }
            rules_arr.push(Value16::object(rule_obj));
        }
        compose_obj.insert("rules".to_string(), Value16::array(rules_arr));
        // SOP0009: field correspondence rules
        let mut field_rules_arr = Vec::new();
        for (field_name, corr) in field_rules {
            let mut fr = hudhudscript_bytecode::ObjMap::default();
            fr.insert("field".to_string(), Value16::string(field_name.clone()));
            let corr_str = match corr {
                hudhudscript_ast::FieldCorrespondence::Correspond => "correspond",
                hudhudscript_ast::FieldCorrespondence::Separate => "separate",
            };
            fr.insert("mode".to_string(), Value16::string(corr_str.to_string()));
            field_rules_arr.push(Value16::object(fr));
        }
        compose_obj.insert("field_rules".to_string(), Value16::array(field_rules_arr));
        let idx = self.bytecode.add_constant(Value16::object(compose_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_move(255, tr ); }
        self.emit_decl_store("compose", base_subject, 255);
        Ok(())
    }
}
