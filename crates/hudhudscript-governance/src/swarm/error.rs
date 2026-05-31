//! Swarm errors — unified error catalog bridge.

use crate::types::AgentId;

/// Error types for swarm operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwarmError {
    /// Agent ID does not exist
    AgentNotFound(AgentId),
    /// Agent ID is already in the swarm
    DuplicateAgent(AgentId),
    /// Shared state key not found
    StateKeyNotFound(String),
}

impl SwarmError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            SwarmError::AgentNotFound(..) => hudhudscript_errors::ErrorCode::SwarmAgentNotFound,
            SwarmError::DuplicateAgent(..) => hudhudscript_errors::ErrorCode::SwarmDuplicateAgent,
            SwarmError::StateKeyNotFound(..) => {
                hudhudscript_errors::ErrorCode::SwarmStateKeyNotFound
            }
        }
    }
}

impl std::fmt::Display for SwarmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            SwarmError::AgentNotFound(id) => write!(f, "Agent not found: {}", id),
            SwarmError::DuplicateAgent(id) => write!(f, "Duplicate agent: {}", id),
            SwarmError::StateKeyNotFound(key) => write!(f, "State key not found: {}", key),
        }
    }
}

impl std::error::Error for SwarmError {}
