use super::*;

impl Compiler {
    #[inline]
    pub(crate) fn emit_decl_store(&mut self, kind: &str, name: &str) {
        let idx = self.bytecode.add_two_sym_payload(sym(kind).0, sym(name).0);
        self.bytecode.push_instr(Instruction::DeclStore { payload_idx: idx as u16, src: 255 });
    }

    pub(crate) fn compile_decl_fields(
        &mut self,
        kind: &str,
        name: &str,
        fields: &[(String, Expr)],
    ) -> CompileResult<()> {
        for (key, value) in fields {
            let key_idx = self.bytecode.add_constant(Value16::string(key.clone()));
            { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: key_idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
            self.compile_expr(value)?;
        }
        { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::MakeObject { dst: tr, count: fields.len() as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        self.emit_decl_store(kind, name);
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
                let mut obj: HashMap<String, Value16> = HashMap::new();
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
