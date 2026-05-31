use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum UiExceptionCode {
    /// E0011 — UI Bridge Lost Its Connection
    BridgeConnectionLost = 11,
    /// E0012 — Underlying UI Framework Reported Error
    BridgeFrameworkError = 12,
    /// E0013 — UI Bridge Initialization Failed
    BridgeInitFailed = 13,
    /// E0014 — UI Bridge Render Step Failed
    BridgeRenderFailed = 14,
    /// E0015 — Requested UI Framework Not Built In
    BridgeUnsupported = 15,
    /// E0145 — Navigation Blocked By Guard
    NavigationBlocked = 145,
    /// E0146 — No Navigation History To Pop
    NavigationNoHistory = 146,
    /// E0147 — Navigation Target Screen Unknown
    NavigationScreenNotFound = 147,
}
