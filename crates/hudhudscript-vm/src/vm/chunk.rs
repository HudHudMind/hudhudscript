use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, Value16};
use std::collections::HashMap;
use std::sync::Arc;

impl VM {
    pub(crate) fn call_chunk(
        &mut self,
        chunk: &FunctionChunk,
        params: &[String],
        args: &[Value16],
        bytecode: &Bytecode,
        func_name: &str,
    ) -> CompileResult<Value16> {
        self.call_chunk_with_captures(chunk, params, args, bytecode, func_name, &HashMap::new())
    }

    pub(crate) fn call_chunk_with_captures(
        &mut self,
        chunk: &FunctionChunk,
        params: &[String],
        args: &[Value16],
        bytecode: &Bytecode,
        func_name: &str,
        closure_captures: &HashMap<String, Arc<parking_lot::RwLock<Value16>>>,
    ) -> CompileResult<Value16> {
        let func_sym =
            hudhudscript_bytecode::SymId(hudhudscript_bytecode::interner::intern(func_name).0);
        let stop_depth = self.frame_stack.len();
        self.exec_call_push_frame(
            chunk,
            params,
            args,
            bytecode,
            func_sym,
            Some(closure_captures),
            0,
            255,
        )?;
        let returned = self.run_frame_loop(bytecode, &[], stop_depth)?;
        if !returned {
            self.registers[255] = Value16::null();
        }
        Ok(self.registers[255])
    }
}
