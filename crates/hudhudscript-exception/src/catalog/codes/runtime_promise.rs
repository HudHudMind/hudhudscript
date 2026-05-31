use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum RuntimePromiseExceptionCode {
    /// E0193 — Promise has already been rejected
    PromiseAlreadyRejected = 193,
    /// E0194 — Promise has already been resolved
    PromiseAlreadyResolved = 194,
    /// E0195 — Promise receiver was dropped before settlement
    PromiseReceiverDropped = 195,
    /// E0196 — Awaited promise was rejected
    PromiseRejected = 196,
}
