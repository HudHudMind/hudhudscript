use super::*;

impl Compiler {
    pub(super) fn compile_decl_import(
        &mut self,
        module: &str,
        alias: Option<&String>,
    ) -> CompileResult<()> {
        let alias_sym = alias.map(|a| sym(a));
        let base_dir = self
            .module_base_dir
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned());
        let idx = self
            .bytecode
            .add_load_module_payload(hudhudscript_bytecode::LoadModulePayload {
                path: module.to_string(),
                alias: alias_sym,
                base_dir,
            });
        self.bytecode.push_instr(Instruction::LoadModule(idx));
        Ok(())
    }

    pub(super) fn compile_decl_agent(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        use hudhudscript_ast::Expr;
        use std::collections::HashMap;
        let mut agent_obj = hudhudscript_bytecode::ObjMap::default();
        agent_obj.insert("name".to_string(), Value16::string(name.to_string()));

        let mut dynamic_provider_expr = None;
        for (key, value_expr) in fields {
            if key == "provider" {
                // provider: DeepSeek → store as string reference (backward compatibility)
                if let Expr::Identifier(ident, _) = value_expr {
                    agent_obj.insert(key.clone(), Value16::string(ident.clone()));
                } else {
                    dynamic_provider_expr = Some(value_expr);
                }
            } else {
                let val = self.expr_to_const_value(value_expr);
                agent_obj.insert(key.clone(), val);
            }
        }
        let idx = self.bytecode.add_constant(Value16::object(agent_obj));

        let obj_reg = crate::compiler::regalloc::temp_reg();
        self.bytecode.push_instr(Instruction::LoadConst {
            dst: obj_reg,
            const_idx: idx as u16,
        });

        if let Some(expr) = dynamic_provider_expr {
            self.compile_expr(expr)?;
            let key_sym = sym("provider");
            self.bytecode.push_instr(Instruction::SetProperty {
                dst: obj_reg,
                obj: obj_reg,
                val: 255,
                prop_sym: key_sym.0 as u16,
            });
        }

        self.bytecode.push_move(255, obj_reg);
        self.emit_decl_store("agent", name, 255);
        Ok(())
    }

    pub(super) fn compile_decl_action(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("action", name, fields)
    }

    pub(super) fn compile_decl_tool(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("tool", name, fields)
    }

    pub(super) fn compile_decl_resource(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("resource", name, fields)
    }

    pub(super) fn compile_decl_provider(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut provider_obj = hudhudscript_bytecode::ObjMap::default();
        provider_obj.insert("name".to_string(), Value16::string(name.to_string()));
        for (key, value_expr) in fields {
            let val = self.provider_field_value(value_expr);
            let normalized_key = match key.as_str() {
                "apiKey" => "api_key".to_string(),
                "baseUrl" => "base_url".to_string(),
                other => other.to_string(),
            };
            if normalized_key != *key {
                provider_obj.insert(key.clone(), val.clone());
            }
            provider_obj.insert(normalized_key, val);
        }
        let idx = self.bytecode.add_constant(Value16::object(provider_obj));
        {
            let tr = crate::compiler::regalloc::temp_reg();
            self.bytecode.push_instr(Instruction::LoadConst {
                dst: tr,
                const_idx: idx as u16,
            });
            self.bytecode.push_move(255, tr);
        }
        self.emit_decl_store("provider", name, 255);
        Ok(())
    }

    /// ENV0001: resolve provider field values. `env("NAME")` → lazy marker, else const.
    fn provider_field_value(&self, expr: &Expr) -> Value16 {
        use hudhudscript_ast::Literal;
        if let Expr::Call { callee, args, .. } = expr {
            if let Expr::Identifier(name, _) = callee.as_ref() {
                if name == "env" && args.len() == 1 {
                    if let Expr::Literal(Literal::String(s), _) = &args[0] {
                        return Value16::string(format!("__hudhud_env__:{}", s));
                    }
                }
            }
        }
        self.expr_to_const_value(expr)
    }

    pub(super) fn compile_decl_entity(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("entity", name, fields)
    }

    pub(super) fn compile_decl_statemachine(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("statemachine", name, fields)
    }

    pub(super) fn compile_decl_event(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("event", name, fields)?;
        // SOP: also register event schema for validation
        self.emit_decl_store("event_schema", name, 255);
        Ok(())
    }

    pub(super) fn compile_decl_contract(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("contract", name, fields)
    }

    pub(super) fn compile_decl_treaty(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("treaty", name, fields)
    }

    pub(super) fn compile_decl_music(
        &mut self,
        kind: &str,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        let mut obj_fields: Vec<(String, Expr)> = vec![(
            "__kind".to_string(),
            Expr::Literal(
                Literal::String(kind.to_string()),
                hudhudscript_ast::Span::default(),
            ),
        )];
        obj_fields.extend(fields.iter().cloned());
        self.compile_decl_fields("music", name, &obj_fields)
    }
}
