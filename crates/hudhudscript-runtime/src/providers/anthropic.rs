//! Anthropic (Claude) Provider Implementation
//!
//! This module provides integration with Anthropic's Claude models.

use crate::provider::{
    estimate_tokens, LLMRequest, LLMResponse, Provider, ProviderConfig, ProviderError,
    ProviderInfo, TokenTracker, TokenUsage, TokenUsageStats,
};
use crate::providers::http_client::{shared_http_client, send_with_retry};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Anthropic provider for Claude models
pub struct AnthropicProvider {
    config: ProviderConfig,
    client: Client,
    tracker: Arc<Mutex<TokenTracker>>,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        if config.api_key.is_none() {
            return Err(ProviderError::InvalidConfig(
                "Anthropic API key is required".to_string(),
            ));
        }

        let client = shared_http_client()?;
        let tracker = Arc::new(Mutex::new(TokenTracker::new()));

        Ok(Self {
            config,
            client,
            tracker,
        })
    }

    /// Get API key from config or environment
    fn get_api_key(&self) -> Result<String, ProviderError> {
        self.config
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| {
                ProviderError::InvalidConfig(
                    "Anthropic API key not found in config or ANTHROPIC_API_KEY env var"
                        .to_string(),
                )
            })
    }

    /// Return provider information (inherent — accessible without importing `Provider` trait).
    pub fn info(&self) -> crate::provider::ProviderInfo {
        crate::provider::ProviderInfo {
            name: "anthropic".to_string(),
            model: self.config.model.clone(),
            provider_type: crate::provider::ProviderType::Anthropic,
        }
    }

    /// Get API endpoint
    fn get_endpoint(&self) -> String {
        self.config
            .endpoint
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_BASE").ok())
            .unwrap_or_else(|| "https://api.anthropic.com/v1/messages".to_string())
    }
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    /// Populated by the Anthropic API but not used directly; kept for deserialization.
    #[serde(rename = "id")]
    _id: String,
    /// Populated by the Anthropic API but not used directly; kept for deserialization.
    #[serde(rename = "type")]
    _response_type: String,
    /// Populated by the Anthropic API but not used directly; kept for deserialization.
    #[serde(rename = "role")]
    _role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
    /// For tool_use blocks: tool call identifier
    #[serde(default)]
    id: Option<String>,
    /// For tool_use blocks: tool name
    #[serde(default)]
    name: Option<String>,
    /// For tool_use blocks: tool input parameters
    #[serde(default)]
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

#[async_trait::async_trait]
impl Provider for AnthropicProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        // Estimate tokens and check budget
        let estimated_tokens = estimate_tokens(&request.prompt);
        self.check_budget(estimated_tokens)?;

        // Get API key
        let api_key = self.get_api_key()?;

        // Prepare request
        let anthropic_request = AnthropicRequest {
            model: self.config.model.clone(),
            max_tokens: request
                .max_tokens
                .or(self.config.max_tokens)
                .unwrap_or(1024),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            system: request.system_prompt.clone(),
            temperature: request.temperature.or(self.config.temperature),
        };

        // Make API call (retry once on 5xx)
        let endpoint = self.get_endpoint();
        let build_req = || {
            self.client
                .post(&endpoint)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&anthropic_request)
        };
        let response = send_with_retry(build_req(), build_req()).await?;

        // Check for errors
        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(ProviderError::ApiError(format!(
                "Anthropic API error: {}",
                error_text
            )));
        }

        // Parse response
        let anthropic_response: AnthropicResponse = response.json().await?;

        // Extract text content and tool_use blocks separately
        let mut text_parts = Vec::new();
        let mut tool_calls_vec = Vec::new();

        for c in anthropic_response.content {
            match c.content_type.as_str() {
                "text" => text_parts.push(c.text),
                "tool_use" => {
                    if let (Some(id), Some(name)) = (c.id, c.name) {
                        tool_calls_vec.push(crate::provider::LLMToolCall {
                            id,
                            name,
                            arguments: c.input.unwrap_or(serde_json::Value::Null),
                        });
                    }
                }
                _ => {}
            }
        }

        let content = text_parts.join("\n");
        let tool_calls = if tool_calls_vec.is_empty() {
            None
        } else {
            Some(tool_calls_vec)
        };

        // Record token usage
        let total_tokens =
            anthropic_response.usage.input_tokens + anthropic_response.usage.output_tokens;
        self.tracker.lock().await.record(
            anthropic_response.usage.input_tokens,
            anthropic_response.usage.output_tokens,
        );

        Ok(LLMResponse {
            content,
            tokens_used: TokenUsage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens,
            },
            model: anthropic_response.model,
            finish_reason: anthropic_response
                .stop_reason
                .unwrap_or_else(|| "stop".to_string()),
            tool_calls,
        })
    }

    fn info(&self) -> ProviderInfo {
        AnthropicProvider::info(self)
    }

    fn check_budget(&self, tokens: usize) -> Result<(), ProviderError> {
        if let Some(budget) = &self.config.budget {
            if tokens > budget.max_tokens_per_call {
                return Err(ProviderError::BudgetExceeded {
                    limit: budget.max_tokens_per_call,
                    requested: tokens,
                });
            }
        }
        Ok(())
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(vec![])
    }

    async fn get_usage_stats(&self) -> TokenUsageStats {
        let tracker = self.tracker.lock().await;
        TokenUsageStats {
            daily_usage: tracker.daily_usage(),
            monthly_usage: tracker.monthly_usage(),
            estimated_cost: (tracker.monthly_usage() as f64) * 0.00001, // Rough estimate
            last_reset: tracker.last_reset(),
        }
    }
}
