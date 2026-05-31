use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum GovernanceConstitutionErrorCode {
    /// E0042 — Invalid constitution version string
    ConstitutionInvalidVersion = 42,
    /// E0043 — Law not present in constitution
    ConstitutionLawNotFound = 43,
    /// E0044 — Constitution has no previous version to roll back to
    ConstitutionNoPreviousVersion = 44,
    /// E0045 — Constitution not registered
    ConstitutionNotFound = 45,
}
