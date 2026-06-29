//! Channel message types.
//!
//! `OutboundMessage` is what the script sends to a channel.
//! `InboundMessage` is what the channel delivers back (poll/receive).

/// Kind of outbound message — determines rendering hints.
#[derive(Debug, Clone)]
pub enum OutboundMessageKind {
    /// Informational notification.
    Info,
    /// Alert / warning notification (from tokenomics alert system).
    Alert,
    /// Approval request — requires a yes/no/abort response.
    ApprovalRequest {
        /// Unique approval request ID.
        id: String,
        /// Short code describing the action (e.g. "exec.sudo", "budget.override").
        code: String,
    },
}

/// Message sent from the system to a channel.
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    /// Plain-text message body.
    pub text: String,
    /// Message kind for rendering / routing.
    pub kind: OutboundMessageKind,
    /// Optional action buttons (for approval requests).
    pub buttons: Vec<String>,
}

impl OutboundMessage {
    /// Create a simple info message.
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: OutboundMessageKind::Info,
            buttons: Vec::new(),
        }
    }

    /// Create an approval request message.
    pub fn approval(id: String, code: String, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: OutboundMessageKind::ApprovalRequest { id, code },
            buttons: vec!["approve".into(), "deny".into()],
        }
    }
}

/// Message received from a channel (e.g., user reply on Telegram).
///
/// **Important:** This struct carries only `String` values, never `Value16`.
/// This ensures the polling thread (which may run outside the VM's GC arena)
/// does not create safety issues with GC-managed objects (GCv2 C7).
#[derive(Debug, Clone)]
pub struct InboundMessage {
    /// Channel name that delivered this message.
    pub channel: String,
    /// Sender identifier (Telegram chat_id, Slack user_id, etc.).
    pub sender_id: String,
    /// Plain-text message body.
    pub text: String,
    /// If this is a reply to an approval request, the request ID.
    pub reply_to_request: Option<String>,
}
