//! Tool definitions for LLM function calling

use serde::{Deserialize, Serialize};

/// Tool definition — sent to LLM as available function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (must be unique, snake_case)
    pub name: String,
    /// Human-readable description — LLM uses this to decide when to call
    pub description: String,
    /// JSON Schema for parameters
    pub parameters: serde_json::Value,
}

/// Tool call result — returned from tool execution back to LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_call_id: String,
    pub name: String,
    pub content: String,
}
