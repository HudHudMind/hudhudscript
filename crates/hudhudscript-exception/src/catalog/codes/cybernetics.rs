use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum CyberneticsExceptionCode {
    /// E0062 — Cybernetic Loop Actuator Write Failed
    CyberneticsActuationFailed = 62,
    /// E0063 — Cybernetic Loop Observer Read Failed
    CyberneticsObserverError = 63,
    /// E0064 — Cybernetic Loop Policy Decision Failed
    CyberneticsPolicyError = 64,
}
