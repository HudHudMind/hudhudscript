use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum LspErrorCode {
    /// E0122 — LSP Failed To Start Tokio Runtime
    LspRuntimeStartFailed = 122,
    /// E0123 — Generic LSP Server Error
    LspServer = 123,
}
