use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ResourceExceptionCode {
    /// E0217 — Resource Discovery Failed
    ResourceDiscoveryFailed = 217,
    /// E0218 — Invalid Resource URI
    ResourceInvalidUri = 218,
    /// E0219 — Resource Not Found
    ResourceNotFound = 219,
    /// E0220 — Resource Read Failed
    ResourceReadFailed = 220,
    /// E0267 — Text Stream Channel Closed Unexpectedly
    StreamChannelClosed = 267,
    /// E0268 — Text Stream Decode Error
    StreamDecodeError = 268,
    /// E0269 — Text Stream Encode Error
    StreamEncodeError = 269,
    /// E0270 — Swarm agent execution failed
    SwarmAgentFailed = 270,
    /// E0271 — Agent not in swarm
    SwarmAgentNotFound = 271,
    /// E0272 — Agent already in swarm
    SwarmDuplicateAgent = 272,
    /// E0273 — Swarm did not meet success quorum
    SwarmInsufficientSuccess = 273,
    /// E0274 — Swarm has no agents
    SwarmNoAgents = 274,
    /// E0275 — Swarm state key not found
    SwarmStateKeyNotFound = 275,
    /// E0276 — Swarm execution timed out
    SwarmTimeout = 276,
}
