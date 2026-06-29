//! HudHudScript Channel Abstraction
//!
//! Provides a unified `Channel` trait for sending and receiving messages
//! across Telegram, Slack, Discord, Web, and terminal transports.
//!
//! # Architecture
//!
//! ```text
//!                  ┌─────────────┐
//!                  │  Script API │  channel.send(), channel.notify()
//!                  └──────┬──────┘
//!                         │
//!                  ┌──────▼──────┐
//!                  │ ChannelReg  │  config-driven, kural 7 single source
//!                  └──────┬──────┘
//!         ┌───────────────┼───────────────┐
//!  ┌──────▼──────┐ ┌─────▼──────┐ ┌──────▼──────┐
//!  │  Telegram   │ │  Slack     │ │    Web      │
//!  │  Transport  │ │  Transport │ │  Transport  │
//!  └─────────────┘ └────────────┘ └─────────────┘
//! ```
//!
//! # Kural 7 (Single Source)
//!
//! Telegram, Slack, Discord, Web için ayrı silolar yasak.
//! Tek trait, çok transport.

pub mod allowlist;
pub mod channel;
pub mod error;
pub mod message;
pub mod registry;
pub mod slack_discord;
pub mod telegram;
pub mod web;

pub use allowlist::ChannelAllowlist;
pub use channel::Channel;
pub use error::ChannelError;
pub use message::{InboundMessage, OutboundMessage, OutboundMessageKind};
pub use registry::ChannelRegistry;
pub use slack_discord::{DiscordChannel, SlackChannel};
pub use telegram::TelegramChannel;
pub use web::WebChannel;
