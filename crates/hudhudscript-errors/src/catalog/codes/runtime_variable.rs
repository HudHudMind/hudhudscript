use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimeVariableErrorCode {
    /// E0233 — Assignment to immutable variable
    RuntimeImmutableVariable = 233,
    /// E0234 — Index out of bounds
    RuntimeIndexOutOfBounds = 234,
    /// E0251 — Reference to undefined variable
    RuntimeUndefinedVariable = 251,
    /// E0252 — Access before initialization (temporal dead zone)
    RuntimeUninitializedVariable = 252,
    /// E0253 — Variable already defined in scope
    RuntimeVariableAlreadyDefined = 253,
}
