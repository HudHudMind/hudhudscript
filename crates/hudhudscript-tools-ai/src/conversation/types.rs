use serde::{Deserialize, Serialize};

use crate::context::estimate_tokens;

/// The role of a message participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

/// Represents a tool/function call requested by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call (used to correlate results).
    pub id: String,
    /// Name of the tool/function to invoke.
    pub name: String,
    /// JSON-encoded arguments for the tool call.
    pub arguments: String,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message author.
    pub role: Role,
    /// Text content of the message.
    pub content: String,
    /// For tool-result messages: the id of the tool call this responds to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// For assistant messages: tool calls the model wants to make.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Unix timestamp (seconds) when this message was created.
    pub timestamp: u64,
}

impl Message {
    /// Create a new message with the current timestamp.
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
            timestamp: super::unix_now(),
        }
    }

    /// Create a tool-result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
            tool_calls: None,
            timestamp: super::unix_now(),
        }
    }

    /// Create an assistant message that includes tool calls.
    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_call_id: None,
            tool_calls: Some(tool_calls),
            timestamp: super::unix_now(),
        }
    }

    /// Estimate the token count of this message's content.
    pub fn estimated_tokens(&self) -> usize {
        let content_tokens = estimate_tokens(&self.content);
        let tool_tokens = self
            .tool_calls
            .as_ref()
            .map(|calls| {
                calls
                    .iter()
                    .map(|c| estimate_tokens(&c.name) + estimate_tokens(&c.arguments))
                    .sum::<usize>()
            })
            .unwrap_or(0);
        4 + content_tokens + tool_tokens
    }
}
