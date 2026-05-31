use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum TokenomicsRatelimitExceptionCode {
    /// E0207 — Requests-Per-Minute Limit Reached
    RateLimitRpmExceeded = 207,
    /// E0208 — Tokens-Per-Minute Limit Reached
    RateLimitTpmExceeded = 208,
}
