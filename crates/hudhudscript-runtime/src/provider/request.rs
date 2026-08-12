//! LLM request / response types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::provider::tool::ToolDefinition;

/// LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    /// User prompt
    pub prompt: String,

    /// System prompt (optional)
    pub system_prompt: Option<String>,

    /// Temperature override
    pub temperature: Option<f64>,

    /// Max tokens override
    pub max_tokens: Option<usize>,

    /// Mnemonic dictionary for token optimization
    pub mnemonics: Option<HashMap<String, String>>,

    /// Enable automatic token optimization
    pub optimize: bool,

    /// Tools available to the LLM (tool calling / function calling)
    pub tools: Option<Vec<ToolDefinition>>,

    /// Request-specific timeout override
    pub timeout_secs: Option<u64>,
}

/// A single tool call requested by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// Response content
    pub content: String,

    /// Token usage information
    pub tokens_used: TokenUsage,

    /// Model that generated the response
    pub model: String,

    /// Finish reason (stop, length, tool_calls, etc.)
    pub finish_reason: String,

    /// Tool calls requested by the LLM (if any)
    pub tool_calls: Option<Vec<LLMToolCall>>,
}

/// Token usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens: usize,

    /// Tokens in the completion
    pub completion_tokens: usize,

    /// Total tokens used
    pub total_tokens: usize,
}
