use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The outcome of applying a control action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActuationResult {
    /// Whether the actuation succeeded.
    pub success: bool,
    /// Human-readable description of what happened.
    pub description: String,
    /// How long the actuation took.
    #[serde(skip)]
    pub duration: Duration,
}

impl ActuationResult {
    /// Create a successful actuation result.
    pub fn success(description: impl Into<String>, duration: Duration) -> Self {
        Self {
            success: true,
            description: description.into(),
            duration,
        }
    }

    /// Create a failed actuation result.
    pub fn failure(description: impl Into<String>, duration: Duration) -> Self {
        Self {
            success: false,
            description: description.into(),
            duration,
        }
    }
}
