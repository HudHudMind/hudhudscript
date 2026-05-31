use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ToolRegistryExceptionCode {
    /// E0209 — Tool Registry Call Failed
    RegistryCallFailed = 209,
    /// E0210 — Tool Discovery Failed
    RegistryDiscoveryFailed = 210,
    /// E0211 — Duplicate Tool Registration
    RegistryDuplicateTool = 211,
    /// E0212 — Tool Server Not Found In Registry
    RegistryServerNotFound = 212,
    /// E0213 — Tool Not Found In Registry
    RegistryToolNotFound = 213,
    /// E0214 — Tool Schema Validation Failed
    RegistryValidationFailed = 214,
}
