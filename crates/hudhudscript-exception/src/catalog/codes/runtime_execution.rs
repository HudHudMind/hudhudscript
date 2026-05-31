use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimeExecutionExceptionCode {
    /// E0227 — Error invoking a callable value
    RuntimeCallError = 227,
    /// E0229 — Custom runtime error
    RuntimeCustom = 229,
    /// E0230 — Division by zero
    RuntimeDivisionByZero = 230,
    /// E0231 — Runtime execution failed
    RuntimeExecutionFailed = 231,
    /// E0235 — Invalid operation for operand types
    RuntimeInvalidOperation = 235,
    /// E0236 — Module-level error
    RuntimeModuleError = 236,
    /// E0237 — Execution gas limit exceeded
    RuntimeOutOfGas = 237,
    /// E0238 — Promise rejection surfaced in interpreter
    RuntimePromiseRejected = 238,
    /// E0239 — Property not found on value
    RuntimePropertyNotFound = 239,
    /// E0240 — Resource access failed
    RuntimeResourceError = 240,
    /// E0242 — Security sandbox violation
    RuntimeSecurityViolation = 242,
    /// E0243 — Call stack overflow
    RuntimeStackOverflow = 243,
    /// E0244 — Invalid runtime state
    RuntimeStateError = 244,
    /// E0249 — Tool invocation failed
    RuntimeToolError = 249,
    /// E0250 — Runtime type mismatch
    RuntimeTypeError = 250,
}
