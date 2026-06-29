use super::*;

impl Compiler {
    pub(super) fn compile_decl_store(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        use std::collections::HashMap;
        let mut store_obj = hudhudscript_bytecode::ObjMap::default();
        store_obj.insert("__type".to_string(), Value16::string("store".to_string()));
        store_obj.insert("name".to_string(), Value16::string(name.to_string()));
        for (key, value_expr) in fields {
            let val = self.expr_to_const_value(value_expr);
            store_obj.insert(key.clone(), val);
        }
        if !store_obj.contains_key("dimensions") {
            store_obj.insert("dimensions".to_string(), Value16::number(128.0));
        }
        if !store_obj.contains_key("distance") {
            store_obj.insert(
                "distance".to_string(),
                Value16::string("cosine".to_string()),
            );
        }
        let idx = self.bytecode.add_constant(Value16::object(store_obj));
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store("store", name, 255);
        Ok(())
    }
}
