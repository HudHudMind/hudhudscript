//! Channel error types.

use thiserror::Error;

/// Errors that can occur during channel operations.
#[derive(Debug, Error)]
pub enum ChannelError {
    /// Channel is not configured (missing config section or env var).
    #[error("Channel '{0}' is not configured")]
    NotConfigured(String),

    /// Network or transport error.
    #[error("Channel '{channel}' transport error: {message}")]
    Transport { channel: String, message: String },

    /// Rate limited by the provider.
    #[error("Channel '{channel}' rate limited: retry after {retry_after_secs}s")]
    RateLimited {
        channel: String,
        retry_after_secs: u64,
    },

    /// Sender is not allowed (pairing/allowlist).
    #[error("Channel '{channel}': sender '{sender}' is not allowed")]
    SenderNotAllowed { channel: String, sender: String },

    /// Invalid message format.
    #[error("Channel '{channel}': invalid message format — {reason}")]
    InvalidMessage { channel: String, reason: String },
}
