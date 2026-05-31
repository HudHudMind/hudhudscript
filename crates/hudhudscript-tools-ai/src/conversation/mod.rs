//! Conversation / Chat State Management (Issue #594)
//!
//! Manages multi-turn conversation state for AI agents: message history with
//! role-based messages, context window truncation, JSON persistence, tool-use
//! loop support, and streaming response accumulation.

pub mod conversation;
pub mod error;
pub mod streaming;
pub mod types;

pub use conversation::*;
pub use error::*;
pub use streaming::*;
pub use types::*;

pub(crate) fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
