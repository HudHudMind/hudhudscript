use super::*;

impl Compiler {
    pub(super) fn compile_decl_protocol(
        &mut self,
        name: &str,
        execution: Option<&String>,
        governance: Option<&String>,
        timeout: Option<&f64>,
        session: &[(String, Expr)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut protocol_obj = HashMap::new();
        if let Some(exec) = execution {
            protocol_obj.insert("execution".to_string(), Value16::string(exec.clone()));
        }
        if let Some(gov) = governance {
            protocol_obj.insert("governance".to_string(), Value16::string(gov.clone()));
        }
        if let Some(t) = timeout {
            protocol_obj.insert("timeout".to_string(), Value16::number(*t));
        }
        if !session.is_empty() {
            let mut session_obj = HashMap::new();
            for (hook_name, hook_expr) in session {
                let hook_val = self.compile_session_hook(name, hook_name, hook_expr)?;
                session_obj.insert(hook_name.clone(), hook_val);
            }
            protocol_obj.insert("session".to_string(), Value16::object(session_obj));
        }
        let idx = self.bytecode.add_constant(Value16::object(protocol_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("protocol", name);
        Ok(())
    }

    pub(super) fn compile_decl_strategy(
        &mut self,
        name: &str,
        execution: Option<&String>,
        governance: Option<&String>,
        timeout: Option<&f64>,
        permissions: &[String],
        realm: Option<&String>,
        session: &[(String, Expr)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut strategy_obj = HashMap::new();
        strategy_obj.insert("name".to_string(), Value16::string(name.to_string()));
        if let Some(exec) = execution {
            strategy_obj.insert("execution".to_string(), Value16::string(exec.clone()));
        }
        if let Some(gov) = governance {
            strategy_obj.insert("governance".to_string(), Value16::string(gov.clone()));
        }
        if let Some(t) = timeout {
            strategy_obj.insert("timeout".to_string(), Value16::number(*t));
        }
        strategy_obj.insert(
            "permissions".to_string(),
            Value16::array(
                permissions
                    .iter()
                    .map(|p| Value16::string(p.clone()))
                    .collect(),
            ),
        );
        if let Some(r) = realm {
            strategy_obj.insert("realm".to_string(), Value16::string(r.clone()));
        }
        if !session.is_empty() {
            let mut session_obj = HashMap::new();
            for (hook_name, hook_expr) in session {
                let hook_val = self.compile_session_hook(name, hook_name, hook_expr)?;
                session_obj.insert(hook_name.clone(), hook_val);
            }
            strategy_obj.insert("session".to_string(), Value16::object(session_obj));
        }
        let idx = self.bytecode.add_constant(Value16::object(strategy_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("strategy", name);
        Ok(())
    }
}
