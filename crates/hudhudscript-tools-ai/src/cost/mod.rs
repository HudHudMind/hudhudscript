//! AI Provider Cost Tracking (Issue #607)
//!
//! Token counting per request/response, per-provider cost calculation, and
//! configurable budget alerts.  Pricing tables cover OpenAI, Anthropic,
//! Ollama (free / local) and DeepSeek.

pub mod error;
pub mod pricing;
pub mod tracker;

pub use error::*;
pub use pricing::*;
pub use tracker::*;

pub(crate) fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
