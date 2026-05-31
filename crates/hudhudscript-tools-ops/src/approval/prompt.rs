use serde::{Deserialize, Serialize};

use crate::risk::RiskLevel;

/// The user's response to an approval prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptResponse {
    /// Approve this single invocation.
    Yes,
    /// Deny this single invocation.
    No,
    /// Approve and remember "always allow" for the remainder of the session.
    AlwaysAllow,
    /// Deny and remember "always deny" for the remainder of the session.
    AlwaysDeny,
}

impl std::fmt::Display for PromptResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptResponse::Yes => write!(f, "yes"),
            PromptResponse::No => write!(f, "no"),
            PromptResponse::AlwaysAllow => write!(f, "always-allow"),
            PromptResponse::AlwaysDeny => write!(f, "always-deny"),
        }
    }
}

/// Trait for prompting a user for approval of a tool invocation.
///
/// Implementations may use stdin/stdout, a GUI dialog, a web form, etc.
pub trait ApprovalPrompter: Send + Sync {
    /// Display the approval prompt and return the user's decision.
    fn prompt(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        risk_level: RiskLevel,
    ) -> PromptResponse;
}

/// A prompter that always approves — useful for testing or non-interactive mode.
#[derive(Debug, Clone, Default)]
pub struct AutoApprovePrompter;

impl ApprovalPrompter for AutoApprovePrompter {
    fn prompt(
        &self,
        _tool_name: &str,
        _arguments: &serde_json::Value,
        _risk_level: RiskLevel,
    ) -> PromptResponse {
        PromptResponse::Yes
    }
}

/// A prompter that always denies — useful for testing or strict mode.
#[derive(Debug, Clone, Default)]
pub struct AutoDenyPrompter;

impl ApprovalPrompter for AutoDenyPrompter {
    fn prompt(
        &self,
        _tool_name: &str,
        _arguments: &serde_json::Value,
        _risk_level: RiskLevel,
    ) -> PromptResponse {
        PromptResponse::No
    }
}
