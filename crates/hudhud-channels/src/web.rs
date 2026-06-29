//! Web channel transport — approval and notification via built-in web server.
//!
//! Supports send-only (notification pages) and two-way (approval forms).
//! Approval requests render HTML forms; responses are collected via POST.

use crate::channel::Channel;
use crate::error::ChannelError;
use crate::message::{InboundMessage, OutboundMessage, OutboundMessageKind};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Web channel that queues messages for the web approval interface.
///
/// Outbound messages are stored in a ring buffer to be served by the
/// web server's `/approval` routes. Inbound responses (user clicks
/// approve/deny) are pushed back via `push_response()`.
pub struct WebChannel {
    outbox: Mutex<VecDeque<OutboundMessage>>,
    inbox: Mutex<VecDeque<InboundMessage>>,
}

impl WebChannel {
    pub fn new() -> Self {
        Self {
            outbox: Mutex::new(VecDeque::new()),
            inbox: Mutex::new(VecDeque::new()),
        }
    }

    /// Pop the oldest pending outbound message (for web UI rendering).
    pub fn pop_outbox(&self) -> Option<OutboundMessage> {
        self.outbox.lock().unwrap().pop_front()
    }

    /// Push a user response into the channel (from web form POST).
    pub fn push_response(&self, msg: InboundMessage) {
        self.inbox.lock().unwrap().push_back(msg);
    }

    /// Number of pending approval requests.
    pub fn pending_count(&self) -> usize {
        self.outbox.lock().unwrap().len()
    }
}

impl Default for WebChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Channel for WebChannel {
    fn name(&self) -> &str {
        "web"
    }

    async fn send(&self, msg: &OutboundMessage) -> Result<(), ChannelError> {
        self.outbox.lock().unwrap().push_back(msg.clone());
        Ok(())
    }

    async fn poll(&self) -> Result<Vec<InboundMessage>, ChannelError> {
        let mut inbox = self.inbox.lock().unwrap();
        let msgs: Vec<InboundMessage> = inbox.drain(..).collect();
        Ok(msgs)
    }

    fn supports_receive(&self) -> bool {
        true
    }
}
