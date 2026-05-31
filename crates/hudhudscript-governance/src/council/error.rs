//! Council errors — unified error catalog bridge.

use crate::types::{AgentId, ConstitutionId};

/// Result type for council operations
pub type CouncilResult<T> = Result<T, CouncilError>;

/// Errors that can occur during council operations
#[derive(Debug, Clone, PartialEq)]
pub enum CouncilError {
    /// Constitution not found
    ConstitutionNotFound(ConstitutionId),
    /// Duplicate agent ID in council
    DuplicateAgent(AgentId),
    /// Agent not found in council
    AgentNotFound(AgentId),
    /// Invalid role assignment
    InvalidRole(String),
}

impl CouncilError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CouncilError::ConstitutionNotFound(..) => {
                hudhudscript_errors::ErrorCode::CouncilConstitutionNotFound
            }
            CouncilError::DuplicateAgent(..) => {
                hudhudscript_errors::ErrorCode::CouncilDuplicateAgent
            }
            CouncilError::AgentNotFound(..) => hudhudscript_errors::ErrorCode::CouncilAgentNotFound,
            CouncilError::InvalidRole(..) => hudhudscript_errors::ErrorCode::CouncilInvalidRole,
        }
    }
}

impl std::fmt::Display for CouncilError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CouncilError::ConstitutionNotFound(id) => {
                write!(f, "Constitution not found: {}", id)
            }
            CouncilError::DuplicateAgent(id) => {
                write!(f, "Duplicate agent ID in council: {}", id)
            }
            CouncilError::AgentNotFound(id) => write!(f, "Agent not found in council: {}", id),
            CouncilError::InvalidRole(role) => write!(f, "Invalid role: {}", role),
        }
    }
}

impl std::error::Error for CouncilError {}
