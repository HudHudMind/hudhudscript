use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimeGovernanceExceptionCode {
    /// E0232 — Action violates active constitution
    RuntimeGovernanceViolation = 232,
}
