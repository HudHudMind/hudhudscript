use super::*;

impl Compiler {
    #[inline]
    pub(crate) fn emit_decl_store(&mut self, kind: &str, name: &str, src: u8) {
        let idx = self.bytecode.add_two_sym_payload(sym(kind).0, sym(name).0);
        self.bytecode.push_instr(Instruction::DeclStore { payload_idx: idx as u16, src });
    }

    pub(crate) fn compile_decl_fields(
        &mut self,
        kind: &str,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        // Create empty object, then use SetProperty for each field.
        // Matches VM's register-based MakeObject (count=0, SetProperty per key).
        {
            let tr = crate::compiler::regalloc::temp_reg();
            self.bytecode.push_instr(Instruction::MakeObject { dst: tr, count: 0 });
            self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
        }
        for (key, value) in fields {
            // Save current object (in reg 255) to a temp reg before
            // compile_expr overwrites reg 255.
            let obj_reg = crate::compiler::regalloc::temp_reg();
            self.bytecode.push_instr(Instruction::Move { dst: obj_reg, src: 255 });

            self.compile_expr(value)?;
            // compile_expr stores result in register 255.
            let val_reg = 255u8;
            let key_sym = self.ct_sym(key);
            let dst_reg = crate::compiler::regalloc::temp_reg();
            self.bytecode.push_instr(Instruction::SetProperty {
                dst: dst_reg,
                obj: obj_reg,
                val: val_reg,
                prop_sym: key_sym.0 as u16,
            });
            // Move updated object back to reg 255 for next iteration / DeclStore.
            self.bytecode.push_instr(Instruction::Move { dst: 255, src: dst_reg });
        }
        self.emit_decl_store(kind, name, 255);
        Ok(())
    }

    pub(crate) fn compile_decl_as_object(
        &mut self,
        kind: &str,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        self.compile_decl_fields(kind, name, fields)
    }

    pub(crate) fn expr_to_const_value(&self, expr: &Expr) -> Value16 {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Number(n, _) => Value16::number(*n),
                Literal::Number(n, _) => Value16::number(*n),
                Literal::String(s) => Value16::string(s.clone()),
                Literal::Boolean(b) => Value16::bool_(*b),
                Literal::Null => Value16::null(),
            },
            Expr::Array { elements, .. } => Value16::array(
                elements
                    .iter()
                    .map(|e| self.expr_to_const_value(e))
                    .collect(),
            ),
            Expr::Object { properties, .. } => {
                use std::collections::HashMap;
                let mut obj: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
                for (k, v) in properties {
                    obj.insert(k.clone(), self.expr_to_const_value(v));
                }
                Value16::object(obj)
            }
            Expr::Identifier(name, _) => Value16::string(name.clone()),
            _ => Value16::null(),
        }
    }
}
