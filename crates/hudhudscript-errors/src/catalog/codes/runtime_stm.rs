use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimeStmErrorCode {
    /// E0245 — STM transaction exceeded retry limit
    RuntimeStmMaxRetriesExceeded = 245,
    /// E0246 — STM transaction timed out
    RuntimeStmTimeout = 246,
}
