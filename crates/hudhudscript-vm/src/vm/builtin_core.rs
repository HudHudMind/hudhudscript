use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};

impl crate::vm::VM {
    pub(crate) fn check_arg_count(
        &self,
        name: &str,
        expected: u8,
        actual: u8,
    ) -> CompileResult<()> {
        if expected != actual {
            return Err(compile_codes::runtime_error(format!(
                "{} expects {} argument(s), got {}",
                name, expected, actual
            )));
        }
        Ok(())
    }
}
