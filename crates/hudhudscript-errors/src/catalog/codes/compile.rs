use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum CompileErrorCode {
    /// E0034 — Generic compilation failure in bytecode emitter
    CompileGeneric = 34,
    /// E0035 — Generic compilation failure with source location
    CompileGenericAt = 35,
    /// E0036 — Compiler produced invalid bytecode
    CompileInvalidBytecode = 36,
    /// E0037 — Invalid bytecode at specific source location
    CompileInvalidBytecodeAt = 37,
    /// E0038 — Runtime error surfaced during compilation
    CompileRuntimeError = 38,
    /// E0039 — Runtime error during compilation at source location
    CompileRuntimeErrorAt = 39,
    /// E0040 — Language feature not yet supported by the compiler
    CompileUnsupportedFeature = 40,
    /// E0041 — Unsupported language feature at source location
    CompileUnsupportedFeatureAt = 41,
}
