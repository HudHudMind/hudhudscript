use super::*;

impl Compiler {
    pub(super) fn compile_decl_uiapp(
        &mut self,
        name: &str,
        entry_screen: Option<&String>,
        screens: &[hudhudscript_ast::UiScreenDecl],
        components: &[hudhudscript_ast::UiComponentDecl],
    ) -> CompileResult<()> {
        use crate::bytecode::Value16 as BcValue;
        let mut obj = hudhudscript_bytecode::ObjMap::default();
        obj.insert(
            "__kind__".to_string(),
            BcValue::string("ui_app".to_string()),
        );
        obj.insert("name".to_string(), BcValue::string(name.to_string()));
        if let Some(entry) = entry_screen {
            obj.insert("entry_screen".to_string(), BcValue::string(entry.clone()));
        }
        obj.insert(
            "screens".to_string(),
            BcValue::array(
                screens
                    .iter()
                    .map(|s| BcValue::string(s.name.clone()))
                    .collect(),
            ),
        );
        obj.insert(
            "components".to_string(),
            BcValue::array(
                components
                    .iter()
                    .map(|c| BcValue::string(c.name.clone()))
                    .collect(),
            ),
        );
        let const_idx = self.bytecode.add_constant(BcValue::object(obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: const_idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("ui", name, 255);
        Ok(())
    }

    pub(super) fn compile_decl_deploy(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields("deploy", name, fields)
    }
}
