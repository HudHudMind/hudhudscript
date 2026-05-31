//! Function, arrow, and scope helpers for `CompileTarget`.

use super::*;

impl Compiler {

    pub fn compile_arrow(
        &mut self,
        params: &[String],
        arrow_body: &hudhudscript_ast::ArrowFunctionBody,
        is_async: bool,
    ) -> CompileResult<()> {
        let anon_name = format!("__arrow_{}", self.bytecode.functions.borrow().len());
        let body_stmts: Vec<hudhudscript_ast::Stmt> = match arrow_body {
            hudhudscript_ast::ArrowFunctionBody::Block(stmts) => stmts.clone(),
            hudhudscript_ast::ArrowFunctionBody::Expression(expr) => {
                vec![hudhudscript_ast::Stmt::Return {
                    value: Some(*expr.clone()),
                    span: hudhudscript_ast::Span::default(),
                }]
            }
        };
        let chunk = self.compile_function_body_async(params.to_vec(), &body_stmts, is_async)?;
        let has_captures = !chunk.captures.is_empty();
        let params_clone = chunk.params.clone();
        self.bytecode
            .functions
            .borrow_mut()
            .insert(anon_name.clone(), Arc::new(chunk));

        if has_captures {
            // DefineFunction: runtime'da upvalue_cell_for() çağrılır, değerler capture edilir
            use hudhudscript_bytecode::DefineFunctionPayload;
            let name_sym = hudhudscript_bytecode::interner::intern(&anon_name);
            let payload_idx = self.add_define_function_payload(DefineFunctionPayload {
                name: hudhudscript_bytecode::SymId(name_sym.0),
                chunk_name: anon_name.clone(),
            });
            self.bytecode.push_instr(Instruction::DefineFunction(payload_idx));
            let tr = crate::compiler::regalloc::temp_reg();
            let sym = name_sym.0 as u16;
            self.bytecode.push_instr(Instruction::LoadGlobal { dst: tr, sym });
            self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr });
        } else {
            let func_val = Value16::function(FunctionData {
                name: anon_name.clone(),
                params: params_clone,
                chunk_name: anon_name,
                captures: Default::default(),
            });
            let idx = self.bytecode.add_constant(func_val);
            { let tr = crate::compiler::regalloc::temp_reg(); self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 }); self.bytecode.push_instr(Instruction::Move { dst: 255, src: tr }); }
        }
        Ok(())
    }

    // ── Statement helpers ────────────────────────────────────────────────
    pub fn compile_function_decl(
        &mut self,
        name: &str,
        params: &[String],
        body: &[Stmt],
        is_async: bool,
        is_generator: bool,
        span: Span,
    ) -> CompileResult<()> {
        // FUNCTION0003: duplicate function detection
        if let Some(scope) = self.declared_fns.last_mut() {
            let line = span.start.line;
            if let Some(&prev_line) = scope.get(name) {
                return Err(compile_codes::duplicate_declaration(format!(
                    "Function '{}' already defined at line {} — cannot redefine in the same scope",
                    name, prev_line
                )));
            }
            scope.insert(name.to_string(), line);
        }

        // CROSS-1 (TCO): generator bodies stay on the regular Call path
        // (yield requires frame preservation; tail-call would discard
        // intermediate generator state).  All other functions pass
        // their name so self-tail-call detection works.
        let fn_name_for_tco = if is_generator {
            None
        } else {
            Some(name.to_string())
        };
        let mut chunk = self.compile_function_body_named_async(
            params.to_vec(),
            fn_name_for_tco,
            body,
            is_async,
        )?;
        chunk.is_generator = is_generator;
        if is_generator {
            self.known_generators.insert(name.to_string());
        }
        let has_fn_captures = !chunk.captures.is_empty();
        let name_sym_id = hudhudscript_bytecode::interner::intern(name).0;
        self.bytecode
            .functions
            .borrow_mut()
            .insert(name.to_string(), Arc::new(chunk));

        if has_fn_captures {
            // Closure: DefineFunction handler runtime'da upvalue_cell_for() çağırır
            use hudhudscript_bytecode::DefineFunctionPayload;
            let payload_idx = self.add_define_function_payload(DefineFunctionPayload {
                name: hudhudscript_bytecode::SymId(name_sym_id),
                chunk_name: name.to_string(),
            });
            self.bytecode.push_instr(Instruction::DefineFunction(payload_idx));
        } else {
            let func_val = Value16::function(FunctionData {
                name: name.to_string(),
                params: params.to_vec(),
                chunk_name: name.to_string(),
                captures: Default::default(),
            });
            let idx = self.bytecode.add_constant(func_val);
            let tr = crate::compiler::regalloc::temp_reg();
            self.bytecode.push_instr(Instruction::LoadConst { dst: tr, const_idx: idx as u16 });
            self.bytecode.push_instr(Instruction::StoreGlobal {
                src: tr,
                sym: name_sym_id as u16,
            });
        }
        Ok(())
    }
}
