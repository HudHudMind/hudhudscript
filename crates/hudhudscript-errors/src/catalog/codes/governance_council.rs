use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum GovernanceCouncilErrorCode {
    /// E0052 — Agent not a member of this council
    CouncilAgentNotFound = 52,
    /// E0053 — Council's bound constitution is missing
    CouncilConstitutionNotFound = 53,
    /// E0054 — Agent already in council
    CouncilDuplicateAgent = 54,
    /// E0055 — Council decision execution failed
    CouncilExecutionFailed = 55,
    /// E0056 — Invalid role for council operation
    CouncilInvalidRole = 56,
    /// E0057 — Council has no members
    CouncilNoMembers = 57,
    /// E0058 — Council not found in registry
    CouncilNotFound = 58,
    /// E0059 — Council decision timed out
    CouncilTimeout = 59,
}
