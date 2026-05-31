use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimeAgentErrorCode {
    /// E0224 — Agent with that name is already registered
    RuntimeAgentAlreadyExists = 224,
    /// E0225 — Referenced agent does not exist
    RuntimeAgentNotFound = 225,
    /// E0247 — Task handle does not exist
    RuntimeTaskNotFound = 247,
}
