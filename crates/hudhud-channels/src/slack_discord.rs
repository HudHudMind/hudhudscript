//! Slack and Discord incoming-webhook transports.
//!
//! Both are send-only. The webhook URL is read from environment variables
//! at construction time.

use crate::channel::Channel;
use crate::error::ChannelError;
use crate::message::{InboundMessage, OutboundMessage};
use async_trait::async_trait;
use reqwest::Client;

/// Slack incoming-webhook channel (send-only).
///
/// Requires `HUDHUD_SLACK_WEBHOOK` environment variable pointing to a
/// Slack incoming-webhook URL.
pub struct SlackChannel {
    webhook_url: String,
    client: Client,
}

impl SlackChannel {
    pub fn new(webhook_url: String) -> Result<Self, ChannelError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ChannelError::Transport {
                channel: "slack".to_string(),
                message: e.to_string(),
            })?;
        Ok(Self {
            webhook_url,
            client,
        })
    }
}

#[async_trait]
impl Channel for SlackChannel {
    fn name(&self) -> &str {
        "slack"
    }

    async fn send(&self, msg: &OutboundMessage) -> Result<(), ChannelError> {
        let payload = serde_json::json!({ "text": msg.text });
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ChannelError::Transport {
                channel: "slack".to_string(),
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<InboundMessage>, ChannelError> {
        Ok(Vec::new())
    }

    fn supports_receive(&self) -> bool {
        false
    }
}

/// Discord incoming-webhook channel (send-only).
///
/// Requires `HUDHUD_DISCORD_WEBHOOK` environment variable.
pub struct DiscordChannel {
    webhook_url: String,
    client: Client,
}

impl DiscordChannel {
    pub fn new(webhook_url: String) -> Result<Self, ChannelError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ChannelError::Transport {
                channel: "discord".to_string(),
                message: e.to_string(),
            })?;
        Ok(Self {
            webhook_url,
            client,
        })
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str {
        "discord"
    }

    async fn send(&self, msg: &OutboundMessage) -> Result<(), ChannelError> {
        let payload = serde_json::json!({ "content": msg.text });
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ChannelError::Transport {
                channel: "discord".to_string(),
                message: e.to_string(),
            })?;
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<InboundMessage>, ChannelError> {
        Ok(Vec::new())
    }

    fn supports_receive(&self) -> bool {
        false
    }
}
