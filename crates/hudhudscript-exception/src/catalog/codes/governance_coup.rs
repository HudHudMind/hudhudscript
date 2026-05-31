use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum GovernanceCoupExceptionCode {
    /// E0060 — Coup target agent not found
    CoupAgentNotFound = 60,
    /// E0061 — Coup-bound constitution not found
    CoupConstitutionNotFound = 61,
}
