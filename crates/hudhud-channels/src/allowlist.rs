//! Channel pairing and allowlist management.
//!
//! Channels that support receive (Telegram, Web) must validate that
//! incoming messages come from approved senders. The allowlist is
//! initially empty and populated via a pairing flow.

use std::collections::HashSet;
use std::sync::RwLock;

/// Manages which sender IDs are allowed to interact via a channel.
#[derive(Debug)]
pub struct ChannelAllowlist {
    allowed: RwLock<HashSet<String>>,
    paired: RwLock<HashSet<String>>,
}

impl ChannelAllowlist {
    pub fn new() -> Self {
        Self {
            allowed: RwLock::new(HashSet::new()),
            paired: RwLock::new(HashSet::new()),
        }
    }

    /// Check if a sender is allowed.
    pub fn is_allowed(&self, sender_id: &str) -> bool {
        self.allowed.read().unwrap().contains(sender_id)
    }

    /// Pre-authorize a sender (e.g., from config).
    pub fn allow(&self, sender_id: &str) {
        self.allowed.write().unwrap().insert(sender_id.to_string());
    }

    /// Remove a sender.
    pub fn deny(&self, sender_id: &str) {
        self.allowed.write().unwrap().remove(sender_id);
    }

    /// Register a successful pairing (first-time connect).
    pub fn pair(&self, sender_id: &str) {
        self.paired.write().unwrap().insert(sender_id.to_string());
    }

    /// List allowed sender IDs.
    pub fn allowed_ids(&self) -> Vec<String> {
        self.allowed.read().unwrap().iter().cloned().collect()
    }

    /// List paired sender IDs.
    pub fn paired_ids(&self) -> Vec<String> {
        self.paired.read().unwrap().iter().cloned().collect()
    }
}

impl Default for ChannelAllowlist {
    fn default() -> Self {
        Self::new()
    }
}
