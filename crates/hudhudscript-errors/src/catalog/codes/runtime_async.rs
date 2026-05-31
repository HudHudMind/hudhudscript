use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimeAsyncErrorCode {
    /// E0008 — Promise handle not found in runtime
    AsyncRuntimePromiseNotFound = 8,
    /// E0009 — Async runtime internal failure
    AsyncRuntimeRuntimeError = 9,
    /// E0010 — Failed to spawn async task
    AsyncRuntimeTaskSpawnFailed = 10,
}
