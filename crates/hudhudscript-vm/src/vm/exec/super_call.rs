use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, SymId, Value16};

impl VM {
    /// Handle `Instruction::SuperCall` outside the main instruction driver.
    #[inline(never)]
    pub(crate) fn exec_super_call(
        &mut self,
        method_name_sym: SymId,
        arg_count: u8,
        first_arg: u8,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let n = arg_count as usize;
        let first = first_arg as usize;

        if n > 8 {
            self.args_scratch.clear();
            self.args_scratch
                .extend((0..n).map(|i| self.registers[first + i]));
            let args = std::mem::take(&mut self.args_scratch);
            let result = self.exec_super_call_inner(&args, method_name_sym, bytecode);
            self.args_scratch = args;
            result
        } else {
            let mut args = [Value16::null(); 8];
            for index in 0..n {
                args[index] = self.registers[first + index];
            }
            self.exec_super_call_inner(&args[..n], method_name_sym, bytecode)
        }
    }

    fn exec_super_call_inner(
        &mut self,
        args: &[Value16],
        method_name_sym: SymId,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let method_name = bytecode.resolve_symbol(method_name_sym.0);
        if self.get_var("this").is_none() {
            return Err(compile_codes::runtime_error(
                "super() called without 'this' context".to_string(),
            ));
        }

        let current_class = self
            .class_context_stack
            .last()
            .map(|symbol| {
                hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(
                    symbol.0,
                ))
            })
            .ok_or_else(|| {
                compile_codes::runtime_error("super used outside class context".to_string())
            })?;
        let parent_name = self
            .classes
            .get(&current_class)
            .and_then(|(parent, _)| parent.clone())
            .ok_or_else(|| {
                compile_codes::runtime_error(format!(
                    "Class {} has no parent for super call",
                    current_class
                ))
            })?;

        let chunk_name = format!("{}::{}", parent_name, method_name);
        if let Some(chunk) = bytecode.get_function(chunk_name.as_str()) {
            self.class_context_stack.push(SymId(
                hudhudscript_bytecode::interner::intern(&parent_name).0,
            ));
            let func_sym = hudhudscript_bytecode::interner::intern(&chunk_name);
            self.exec_call_push_frame(
                &chunk,
                &chunk.params,
                args,
                bytecode,
                SymId(func_sym.0),
                Some(&std::collections::HashMap::new()),
                0,
                255,
            )?;
            if let Some(frame) = self.frame_stack.last_mut() {
                frame.class_context = true;
            }
            Ok(())
        } else {
            Err(compile_codes::runtime_error(format!(
                "Parent method not found: {}",
                chunk_name
            )))
        }
    }
}
