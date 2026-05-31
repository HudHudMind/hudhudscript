use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileError, CompileResult, SourcePosition};
use hudhudscript_bytecode::Bytecode;
use hudhudscript_bytecode::Value16;

impl VM {
    /// style is exposed via the `vm_interpreter::Interpreter` shim which
    /// owns the VM and chains this setter internally.
    pub(crate) fn runtime_error_with_pos(
        msg: impl Into<String>,
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileError {
        if let Some((line, col)) = bytecode.get_source_position(ip) {
            compile_codes::runtime_error_at(msg, SourcePosition::new(line, col))
        } else {
            compile_codes::runtime_error(msg)
        }
    }

    /// Convert a runtime `CompileError` to a catchable `Value` that mirrors
    /// the interpreter's try/catch binding shape (see
    /// `crates/hudhudscript-interpreter/src/execution/control_flow.rs`
    /// `execute_try`).
    ///
    /// The interpreter's fallback arm (triggered for every runtime error
    /// that is NOT explicitly one of `RuntimeThrow` / `RuntimeCallError` /
    /// `RuntimeTypeError` / `RuntimePromiseRejected`) produces a
    /// `Value16::string(format!("{}", err))` — i.e. the full `[E0XXX]
    /// Title: Message` catalog-formatted string.
    ///
    /// All VM runtime errors currently share `ErrorCode::CompileRuntimeError`
    /// (the VM is single-code, unlike the interpreter's rich taxonomy),
    /// so they all land in that fallback. We emit the same `Value::String`
    /// shape so that scripts see `typeof(e) == "string"` in both runtimes.
    pub(crate) fn runtime_error_to_value(err: &CompileError) -> Value16 {
        Value16::string(format!("{}", err))
    }
}
