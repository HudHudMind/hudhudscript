use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimeControlErrorCode {
    /// E0226 — `break` used outside a loop
    RuntimeBreak = 226,
    /// E0228 — `continue` used outside a loop
    RuntimeContinue = 228,
    /// E0241 — `return` used outside a function
    RuntimeReturn = 241,
    /// E0248 — Uncaught user exception
    RuntimeThrow = 248,
    /// E0254 — `yield` used outside a generator
    RuntimeYield = 254,
}
