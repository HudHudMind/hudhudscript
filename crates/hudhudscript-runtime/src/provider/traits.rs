//! Provider trait and function-call types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::provider::{
    error::ProviderError,
    request::{LLMRequest, LLMResponse},
    types::{ProviderInfo, TokenUsageStats},
};

// ---------------------------------------------------------------------------
// Issue #114 — LLM Function Calling Abstraction
// ---------------------------------------------------------------------------

/// Strongly-typed representation of a single function/tool call requested by
/// the model.  This unifies the ad-hoc `LLMToolCall` struct with a richer
/// lifecycle that tracks the call from request through result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Unique call identifier (matches the id returned by the provider)
    pub id: String,
    /// Name of the tool / function to invoke
    pub name: String,
    /// Arguments as a JSON object (matches the tool's parameter schema)
    pub arguments: serde_json::Value,
    /// Result type hint — helps the dispatcher pick the right deserializer
    pub result_type: FunctionCallResultType,
}

/// Expected result type for a function call
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FunctionCallResultType {
    #[default]
    Json,
    Text,
    Binary,
}

/// Outcome of executing a `FunctionCall`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallResult {
    /// Mirrors `FunctionCall::id`
    pub id: String,
    /// Mirrors `FunctionCall::name`
    pub name: String,
    /// Serialised result ready to feed back into the LLM conversation
    pub output: serde_json::Value,
    /// Whether the call succeeded
    pub success: bool,
    /// Optional error message when `success == false`
    pub error: Option<String>,
}

impl FunctionCallResult {
    /// Construct a successful result
    pub fn ok(id: impl Into<String>, name: impl Into<String>, output: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            output,
            success: true,
            error: None,
        }
    }

    /// Construct a failed result
    pub fn err(id: impl Into<String>, name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            output: serde_json::Value::Null,
            success: false,
            error: Some(error.into()),
        }
    }

    /// Convert to a `ToolCallResult` for backward-compat with the provider API
    pub fn as_tool_call_result(&self) -> crate::provider::tool::ToolCallResult {
        crate::provider::tool::ToolCallResult {
            tool_call_id: self.id.clone(),
            name: self.name.clone(),
            content: if self.success {
                self.output.to_string()
            } else {
                format!("Error: {}", self.error.as_deref().unwrap_or("unknown"))
            },
        }
    }
}

/// Provider trait - all providers must implement this
#[async_trait]
pub trait Provider: Send + Sync {
    /// Call the LLM with a request
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError>;

    /// Stream the LLM response chunk by chunk, calling `on_chunk` for each token
    async fn stream_call(
        &self,
        request: LLMRequest,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<LLMResponse, ProviderError> {
        // Default: fall back to regular call, emit full response as one chunk
        let response = self.call(request).await?;
        on_chunk(response.content.clone());
        Ok(response)
    }

    /// List available models from the provider's API
    async fn list_models(&self) -> Result<Vec<String>, ProviderError>;

    /// Get provider information
    fn info(&self) -> ProviderInfo;

    /// Check if request is within token budget
    fn check_budget(&self, tokens: usize) -> Result<(), ProviderError>;

    /// Get current token usage statistics
    async fn get_usage_stats(&self) -> TokenUsageStats;
}
