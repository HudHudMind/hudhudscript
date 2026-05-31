use std::path::Path;
use tracing::{debug, warn};

use crate::context::estimate_tokens;

use super::{ConversationError, Message, Role};

/// Persistent representation for save/load (excludes runtime-only fields).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConversationData {
    messages: Vec<Message>,
    system_prompt: Option<String>,
    max_tokens: usize,
    model: String,
}

/// Manages a multi-turn conversation: message history, context window
/// truncation, persistence, and tool-use loop support.
#[derive(Debug, Clone)]
pub struct Conversation {
    /// Ordered message history.
    pub(crate) messages: Vec<Message>,
    /// Optional system prompt (always kept as the first logical message).
    pub(crate) system_prompt: Option<String>,
    /// Maximum token budget for the conversation context window.
    pub(crate) max_tokens: usize,
    /// Model identifier (e.g. `"gpt-4o"`, `"claude-3-sonnet"`).
    pub(crate) model: String,
}

impl Conversation {
    /// Create a new empty conversation.
    pub fn new(model: impl Into<String>, max_tokens: usize) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            max_tokens,
            model: model.into(),
        }
    }

    /// Create a conversation with a system prompt.
    pub fn with_system_prompt(
        model: impl Into<String>,
        max_tokens: usize,
        system_prompt: impl Into<String>,
    ) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: Some(system_prompt.into()),
            max_tokens,
            model: model.into(),
        }
    }

    /// The model identifier for this conversation.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// The configured maximum token budget.
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// The system prompt, if any.
    pub fn system_prompt(&self) -> Option<&str> {
        self.system_prompt.as_deref()
    }

    /// Set or replace the system prompt.
    pub fn set_system_prompt(&mut self, prompt: impl Into<String>) {
        self.system_prompt = Some(prompt.into());
    }

    /// Number of messages in the history (excluding the system prompt).
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Return a reference to all messages.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Return the last message, or `None` if the conversation is empty.
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Add a generic message.
    pub fn add_message(&mut self, message: Message) {
        debug!(role = %message.role, tokens = message.estimated_tokens(), "Adding message to conversation");
        self.messages.push(message);
    }

    /// Add a user message.
    pub fn add_user(&mut self, content: impl Into<String>) {
        self.add_message(Message::new(Role::User, content));
    }

    /// Add an assistant message.
    pub fn add_assistant(&mut self, content: impl Into<String>) {
        self.add_message(Message::new(Role::Assistant, content));
    }

    /// Add a system message (inline, not the conversation-level system prompt).
    pub fn add_system(&mut self, content: impl Into<String>) {
        self.add_message(Message::new(Role::System, content));
    }

    /// Add a tool result message.
    pub fn add_tool_result(&mut self, tool_call_id: impl Into<String>, content: impl Into<String>) {
        self.add_message(Message::tool_result(tool_call_id, content));
    }

    /// Add an assistant message that requests tool calls.
    pub fn add_assistant_with_tools(
        &mut self,
        content: impl Into<String>,
        tool_calls: Vec<super::ToolCall>,
    ) {
        self.add_message(Message::assistant_with_tools(content, tool_calls));
    }

    /// Estimate the total token count of the conversation, including the
    /// system prompt.
    pub fn total_tokens(&self) -> usize {
        let system_tokens = self
            .system_prompt
            .as_ref()
            .map(|s| 4 + estimate_tokens(s))
            .unwrap_or(0);
        let message_tokens: usize = self.messages.iter().map(|m| m.estimated_tokens()).sum();
        system_tokens + message_tokens
    }

    /// Remove the oldest non-system messages until the total token count
    /// fits within `max_tokens`.
    ///
    /// System-role messages in the history and the conversation-level system
    /// prompt are never removed. Returns the number of messages removed.
    pub fn truncate_to_fit(&mut self, max_tokens: usize) -> usize {
        let mut removed = 0;

        while self.total_tokens() > max_tokens && !self.messages.is_empty() {
            if let Some(idx) = self.messages.iter().position(|m| m.role != Role::System) {
                let msg = &self.messages[idx];
                debug!(
                    role = %msg.role,
                    tokens = msg.estimated_tokens(),
                    "Truncating message to fit context window"
                );
                self.messages.remove(idx);
                removed += 1;
            } else {
                warn!("Cannot truncate further — only system messages remain");
                break;
            }
        }

        if removed > 0 {
            debug!(
                removed,
                remaining = self.messages.len(),
                total_tokens = self.total_tokens(),
                max_tokens,
                "Conversation truncated to fit context window"
            );
        }

        removed
    }

    /// Clear all messages (system prompt is preserved).
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Format the conversation for an API call: system prompt first (if any),
    /// followed by all messages. Returns a JSON-serializable array.
    pub fn messages_for_api(&self) -> Vec<serde_json::Value> {
        let mut api_messages = Vec::new();

        if let Some(ref system) = self.system_prompt {
            api_messages.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }

        for msg in &self.messages {
            let mut obj = serde_json::json!({
                "role": msg.role.to_string(),
                "content": msg.content
            });

            if let Some(ref tool_call_id) = msg.tool_call_id {
                obj["tool_call_id"] = serde_json::json!(tool_call_id);
            }

            if let Some(ref tool_calls) = msg.tool_calls {
                let calls: Vec<serde_json::Value> = tool_calls
                    .iter()
                    .map(|tc| {
                        serde_json::json!({
                            "id": tc.id,
                            "type": "function",
                            "function": {
                                "name": tc.name,
                                "arguments": tc.arguments
                            }
                        })
                    })
                    .collect();
                obj["tool_calls"] = serde_json::json!(calls);
            }

            api_messages.push(obj);
        }

        api_messages
    }

    /// Save the conversation to a JSON file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConversationError> {
        let data = ConversationData {
            messages: self.messages.clone(),
            system_prompt: self.system_prompt.clone(),
            max_tokens: self.max_tokens,
            model: self.model.clone(),
        };
        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load a conversation from a JSON file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConversationError> {
        let json = std::fs::read_to_string(path)?;
        let data: ConversationData = serde_json::from_str(&json)?;
        Ok(Self {
            messages: data.messages,
            system_prompt: data.system_prompt,
            max_tokens: data.max_tokens,
            model: data.model,
        })
    }
}
