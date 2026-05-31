use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ToolCoreExceptionCode {
    /// E0295 — Tool dispatch ran but failed
    ToolExecutionFailed = 295,
    /// E0296 — Tool received invalid arguments
    ToolInvalidArguments = 296,
    /// E0297 — Tool call blocked by security policy
    ToolSecurityViolation = 297,
    /// E0298 — Tool input failed semantic validation
    ToolValidation = 298,
}
