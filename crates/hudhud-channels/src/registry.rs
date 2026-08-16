//! Channel registry — single source of truth for active channels.
//!
//! Channels are registered from `hudhud.toml [channels]` config.
//! When the config section is absent, the registry stays empty:
//! zero threads, zero network, zero footprint.

use crate::channel::Channel;
use crate::error::ChannelError;
use crate::message::{InboundMessage, OutboundMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for the inner channel map.
type ChannelMap = HashMap<String, Arc<dyn Channel>>;

/// Registry of active communication channels.
///
/// # Config gate
///
/// If `[channels]` is absent from `hudhud.toml`, no channels are registered,
/// the poll loop does not spawn, and `send` is a no-op.
pub struct ChannelRegistry {
    channels: RwLock<ChannelMap>,
}

impl ChannelRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    /// Register a channel transport.
    pub async fn register(&self, channel: Arc<dyn Channel>) {
        let name = channel.name().to_string();
        self.channels.write().await.insert(name, channel);
    }

    /// Send a message to a specific channel.
    ///
    /// Returns `Ok(())` if the channel is not found (silent no-op for
    /// unconfigured channels — production safety).
    pub async fn send_to(
        &self,
        channel_name: &str,
        msg: &OutboundMessage,
    ) -> Result<(), ChannelError> {
        let channels = self.channels.read().await;
        if let Some(ch) = channels.get(channel_name) {
            ch.send(msg).await
        } else {
            Ok(())
        }
    }

    /// Send a message to all channels that match the given names.
    ///
    /// If `names` is empty, sends to ALL registered channels.
    /// Errors from individual channels are logged but do not abort
    /// delivery to remaining channels.
    pub async fn broadcast(
        &self,
        names: &[String],
        msg: &OutboundMessage,
    ) -> Vec<(String, Result<(), ChannelError>)> {
        let channels = self.channels.read().await;
        let targets: Vec<&String> = if names.is_empty() {
            channels.keys().collect()
        } else {
            names.iter().collect()
        };

        let mut results = Vec::new();
        for name_ref in targets {
            let name: &String = name_ref;
            if let Some(ch) = channels.get(name) {
                let result = ch.send(msg).await;
                results.push((name.clone(), result));
            }
        }
        results
    }

    /// Poll all channels for incoming messages.
    ///
    /// Channels that do not support receive are skipped.
    pub async fn poll_all(&self) -> Vec<InboundMessage> {
        let channels = self.channels.read().await;
        let mut messages = Vec::new();
        for ch in channels.values() {
            if ch.supports_receive() {
                if let Ok(mut msgs) = ch.poll().await {
                    messages.append(&mut msgs);
                }
            }
        }
        messages
    }

    /// Check if any channels are registered.
    pub async fn is_empty(&self) -> bool {
        self.channels.read().await.is_empty()
    }

    /// Number of registered channels.
    pub async fn len(&self) -> usize {
        self.channels.read().await.len()
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
