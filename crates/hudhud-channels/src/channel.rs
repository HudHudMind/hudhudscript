//! Channel trait — the single abstraction for all transports.
//!
//! Kural 7: Telegram, Slack, Discord, Web için ayrı silolar yasak.
//! Tek trait, çok transport.

use crate::error::ChannelError;
use crate::message::{InboundMessage, OutboundMessage};
use async_trait::async_trait;

/// A communication channel that can send and optionally receive messages.
///
/// Channels that are send-only (e.g., Slack/Discord incoming webhooks)
/// must still implement `poll()` but may return an empty `Vec`.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync`. The `poll()` method may be
/// called from a background thread outside the VM's GC arena.
/// `InboundMessage` therefore carries only `String` values, never
/// `Value16` (GCv2 C7).
#[async_trait]
pub trait Channel: Send + Sync {
    /// Human-readable name: "telegram", "slack", "discord", "web".
    fn name(&self) -> &str;

    /// Send a message through this channel.
    async fn send(&self, msg: &OutboundMessage) -> Result<(), ChannelError>;

    /// Poll for incoming messages.
    ///
    /// Send-only channels return an empty `Vec`.
    /// Long-poll channels (Telegram) may block for a configurable timeout.
    async fn poll(&self) -> Result<Vec<InboundMessage>, ChannelError>;

    /// Whether this channel supports receiving (polling).
    fn supports_receive(&self) -> bool {
        false
    }
}
