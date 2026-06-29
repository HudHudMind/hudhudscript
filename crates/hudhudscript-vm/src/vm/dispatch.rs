use super::{decode_packed, PackedResult, VM};
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, Instruction, Value16};

impl VM {
    // R1: dispatch_chunk3 merged into dispatch_chunk5.
    // dispatch_control.rs deleted — single dispatch call now.
    #[inline(always)]
    pub(crate) fn dispatch_packed(
        &mut self,
        p: u32,
        instructions: &[Instruction],
        _constants: &[Value16],
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<PackedResult> {
        let (opcode, arg1, arg2) = decode_packed(p);
        self.dispatch_chunk5(opcode, arg1, arg2, _constants, instructions, bytecode, ip)
    }
}
