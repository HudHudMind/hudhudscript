use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ToolApprovalExceptionCode {
    /// E0001 — Approval state transition is invalid
    ApprovalInvalidTransition = 1,
    /// E0002 — Approval request id does not exist
    ApprovalNotFound = 2,
}
