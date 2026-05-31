use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Unique identifier for a pending approval request.
pub type ApprovalId = String;

/// The state of a single tool-call approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalState {
    /// Waiting for a human decision.
    Pending,
    /// A human approved the call — execution may proceed.
    Approved,
    /// A human denied the call — execution must be skipped.
    Denied,
    /// The approved call was executed.
    Executed,
    /// The denied call was cleanly skipped.
    Skipped,
}

impl std::fmt::Display for ApprovalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalState::Pending => write!(f, "Pending"),
            ApprovalState::Approved => write!(f, "Approved"),
            ApprovalState::Denied => write!(f, "Denied"),
            ApprovalState::Executed => write!(f, "Executed"),
            ApprovalState::Skipped => write!(f, "Skipped"),
        }
    }
}

/// A single approval request record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique ID for this request.
    pub id: ApprovalId,
    /// Name of the tool that wants to be called.
    pub tool_name: String,
    /// Arguments the tool will receive (for display to the human approver).
    pub arguments: serde_json::Value,
    /// Current state in the approval state machine.
    pub state: ApprovalState,
    /// When the request was created.
    pub created_at: SystemTime,
    /// When the state last changed.
    pub updated_at: SystemTime,
    /// Optional reason supplied by the approver.
    pub reason: Option<String>,
}

/// Error returned when an invalid state transition is attempted.
#[derive(Debug)]
pub enum ApprovalError {
    NotFound(ApprovalId),
    InvalidTransition {
        id: ApprovalId,
        from: ApprovalState,
        to: ApprovalState,
    },
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ApprovalError::NotFound(id) => write!(f, "Approval request not found: {}", id),
            ApprovalError::InvalidTransition { id, from, to } => write!(
                f,
                "Invalid transition from {} to {} for request {}",
                from, to, id
            ),
        }
    }
}

impl std::error::Error for ApprovalError {}

impl ApprovalError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ApprovalError::InvalidTransition { .. } => {
                hudhudscript_errors::ErrorCode::ApprovalInvalidTransition
            }
            ApprovalError::NotFound(..) => hudhudscript_errors::ErrorCode::ApprovalNotFound,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<ApprovalError> for hudhudscript_errors::Error {
    fn from(e: ApprovalError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
