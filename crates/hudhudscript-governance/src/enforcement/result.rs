//! Enforcement result types

use crate::types::LawId;

/// Result of constitution enforcement
#[derive(Debug, Clone, PartialEq)]
pub struct EnforcementResult {
    /// Whether the action is allowed
    pub allowed: bool,
    /// List of violated law IDs
    pub violations: Vec<LawId>,
    /// Human-readable message
    pub message: String,
    /// Advisory law violations (informational only)
    pub advisory_violations: Vec<LawId>,
}

impl EnforcementResult {
    /// Create a new enforcement result indicating the action is allowed
    pub fn allowed() -> Self {
        Self {
            allowed: true,
            violations: vec![],
            message: "Action complies with constitution".to_string(),
            advisory_violations: vec![],
        }
    }

    /// Create a new enforcement result indicating the action is denied
    pub fn denied(violations: Vec<LawId>) -> Self {
        Self {
            allowed: false,
            violations,
            message: "Action violates mandatory laws".to_string(),
            advisory_violations: vec![],
        }
    }

    /// Add advisory violations to the result
    pub fn with_advisory_violations(mut self, advisory_violations: Vec<LawId>) -> Self {
        self.advisory_violations = advisory_violations;
        self
    }

    /// Set a custom message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}
