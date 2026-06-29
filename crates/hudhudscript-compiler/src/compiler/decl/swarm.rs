use super::*;

impl Compiler {
    pub(super) fn compile_decl_swarm(
        &mut self,
        name: &str,
        agents: &[String],
        strategy: &str,
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut swarm_obj = hudhudscript_bytecode::ObjMap::default();
        swarm_obj.insert("name".to_string(), Value16::string(name.to_string()));
        swarm_obj.insert(
            "agents".to_string(),
            Value16::array(agents.iter().map(|a| Value16::string(a.clone())).collect()),
        );
        swarm_obj.insert(
            "strategy".to_string(),
            Value16::string(strategy.to_string()),
        );
        let idx = self.bytecode.add_constant(Value16::object(swarm_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("swarm", name, 255);
        Ok(())
    }

    pub(super) fn compile_decl_community(
        &mut self,
        name: &str,
        members: &[String],
        councils: &[String],
        culture: &hudhudscript_ast::CultureDecl,
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut community_obj = hudhudscript_bytecode::ObjMap::default();
        community_obj.insert("name".to_string(), Value16::string(name.to_string()));
        community_obj.insert(
            "members".to_string(),
            Value16::array(members.iter().map(|m| Value16::string(m.clone())).collect()),
        );
        community_obj.insert(
            "councils".to_string(),
            Value16::array(
                councils
                    .iter()
                    .map(|c| Value16::string(c.clone()))
                    .collect(),
            ),
        );
        let mut culture_obj = hudhudscript_bytecode::ObjMap::default();
        culture_obj.insert(
            "values".to_string(),
            Value16::array(
                culture
                    .values
                    .iter()
                    .map(|v| Value16::string(v.clone()))
                    .collect(),
            ),
        );
        culture_obj.insert(
            "norms".to_string(),
            Value16::array(
                culture
                    .norms
                    .iter()
                    .map(|n| Value16::string(n.clone()))
                    .collect(),
            ),
        );
        culture_obj.insert(
            "communication_style".to_string(),
            Value16::string(culture.communication_style.clone()),
        );
        community_obj.insert("culture".to_string(), Value16::object(culture_obj));
        let idx = self.bytecode.add_constant(Value16::object(community_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("community", name, 255);
        Ok(())
    }
}
