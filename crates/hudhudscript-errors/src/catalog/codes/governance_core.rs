use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum GovernanceCoreErrorCode {
    /// E0093 — Agent not registered with governance
    GovernanceAgentNotFound = 93,
    /// E0094 — Governance cache ID collision
    GovernanceCacheIdCollision = 94,
    /// E0095 — Circular dependency in governance graph
    GovernanceCircularDependency = 95,
    /// E0096 — Constitution not found in governance registry
    GovernanceConstitutionNotFound = 96,
    /// E0097 — Governance format validation failed
    GovernanceFormatValidation = 97,
    /// E0098 — Invalid governance configuration
    GovernanceInvalidConfiguration = 98,
    /// E0099 — Invalid role at governance layer
    GovernanceInvalidRole = 99,
    /// E0100 — Governance resource not found
    GovernanceResourceNotFound = 100,
    /// E0101 — Governance serialization or deserialization failed
    GovernanceSerializationError = 101,
}
