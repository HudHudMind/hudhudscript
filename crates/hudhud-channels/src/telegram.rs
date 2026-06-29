//! Telegram channel transport.
//!
//! Uses `api.telegram.org/sendMessage` for outbound and long-polling
//! (`getUpdates`) for inbound messages. The bot token is read from
//! an environment variable at construction time.

use crate::channel::Channel;
use crate::error::ChannelError;
use crate::message::{InboundMessage, OutboundMessage};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    chat: TelegramChat,
    text: Option<String>,
    reply_to_message: Option<Box<TelegramMessage>>,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
}

#[derive(Debug, Deserialize)]
struct GetUpdatesResponse {
    ok: bool,
    result: Vec<TelegramUpdate>,
}

/// Telegram channel via Bot API.
///
/// Requires `HUDHUD_TELEGRAM_TOKEN` environment variable.
pub struct TelegramChannel {
    bot_token: String,
    allowed_chat_ids: HashSet<i64>,
    client: Client,
}

impl TelegramChannel {
    pub fn new(
        bot_token: String,
        allowed_chat_ids: HashSet<i64>,
    ) -> Result<Self, ChannelError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ChannelError::Transport {
                channel: "telegram".to_string(),
                message: e.to_string(),
            })?;
        Ok(Self {
            bot_token,
            allowed_chat_ids,
            client,
        })
    }

    async fn get_updates(&self) -> Result<Vec<TelegramUpdate>, ChannelError> {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates",
            self.bot_token
        );
        let resp: GetUpdatesResponse = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| ChannelError::Transport {
                channel: "telegram".to_string(),
                message: e.to_string(),
            })?
            .json()
            .await
            .map_err(|e| ChannelError::Transport {
                channel: "telegram".to_string(),
                message: e.to_string(),
            })?;
        if !resp.ok {
            return Ok(Vec::new());
        }
        Ok(resp.result)
    }
}

#[async_trait]
impl Channel for TelegramChannel {
    fn name(&self) -> &str {
        "telegram"
    }

    async fn send(&self, msg: &OutboundMessage) -> Result<(), ChannelError> {
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.bot_token
        );
        for &chat_id in &self.allowed_chat_ids {
            let params = [
                ("chat_id", chat_id.to_string()),
                ("text", msg.text.clone()),
            ];
            self.client
                .post(&url)
                .form(&params)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| ChannelError::Transport {
                    channel: "telegram".to_string(),
                    message: e.to_string(),
                })?;
        }
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<InboundMessage>, ChannelError> {
        let updates = self.get_updates().await?;
        let mut messages = Vec::new();
        for update in updates {
            if let Some(msg) = update.message {
                if !self.allowed_chat_ids.contains(&msg.chat.id) {
                    continue;
                }
                let text = msg.text.unwrap_or_default();
                let reply_to = msg
                    .reply_to_message
                    .as_ref()
                    .and_then(|m| m.text.clone());
                messages.push(InboundMessage {
                    channel: "telegram".to_string(),
                    sender_id: msg.chat.id.to_string(),
                    text,
                    reply_to_request: reply_to,
                });
            }
        }
        Ok(messages)
    }

    fn supports_receive(&self) -> bool {
        true
    }
}
