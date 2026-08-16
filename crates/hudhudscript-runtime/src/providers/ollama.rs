//! Ollama Provider Implementation
//!
//! This module provides integration with Ollama for running local LLM models.

use crate::provider::{
    estimate_tokens, LLMRequest, LLMResponse, Provider, ProviderConfig, ProviderError,
    ProviderInfo, TokenTracker, TokenUsage, TokenUsageStats,
};
use crate::providers::http_client::{send_with_retry, shared_http_client};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Ollama provider for local models
pub struct OllamaProvider {
    config: ProviderConfig,
    client: Client,
    tracker: Arc<Mutex<TokenTracker>>,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        let client = shared_http_client()?;
        let tracker = Arc::new(Mutex::new(TokenTracker::new()));

        Ok(Self {
            config,
            client,
            tracker,
        })
    }

    /// Get API endpoint
    pub fn get_endpoint(&self) -> String {
        self.config
            .endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:11434/api/generate".to_string())
    }

    pub fn info(&self) -> crate::provider::ProviderInfo {
        crate::provider::ProviderInfo {
            name: "ollama".to_string(),
            model: self.config.model.clone(),
            provider_type: crate::provider::ProviderType::Ollama,
        }
    }
}

/// Ollama API request format
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
}

/// Ollama API response format
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    response: String,
    done: bool,
    /// Populated by the Ollama API but not used directly; kept for deserialization.
    #[serde(default, rename = "total_duration")]
    _total_duration: u64,
    #[serde(default)]
    prompt_eval_count: usize,
    #[serde(default)]
    eval_count: usize,
}

#[async_trait::async_trait]
impl Provider for OllamaProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        // Estimate tokens and check budget
        let estimated_tokens = estimate_tokens(&request.prompt);
        self.check_budget(estimated_tokens)?;

        // Prepare request
        let ollama_request = OllamaRequest {
            model: self.config.model.clone(),
            prompt: request.prompt.clone(),
            system: request.system_prompt.clone(),
            temperature: request.temperature.or(self.config.temperature),
            stream: false,
            options: request
                .max_tokens
                .or(self.config.max_tokens)
                .map(|max_tokens| OllamaOptions {
                    num_predict: Some(max_tokens),
                }),
        };

        // Make API call (retry once on 5xx)
        let endpoint = self.get_endpoint();
        let timeout_secs = request
            .timeout_secs
            .or(self.config.timeout_secs)
            .unwrap_or(crate::provider::types::DEFAULT_PROVIDER_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        let build_req = || {
            self.client
                .post(&endpoint)
                .timeout(timeout_duration)
                .json(&ollama_request)
        };
        let response = send_with_retry(build_req(), build_req()).await?;

        // Check for errors
        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(ProviderError::ApiError(format!(
                "Ollama API error: {}",
                error_text
            )));
        }

        // Parse response
        let ollama_response: OllamaResponse = response.json().await?;

        // Calculate token usage
        let prompt_tokens = ollama_response.prompt_eval_count;
        let completion_tokens = ollama_response.eval_count;
        let total_tokens = prompt_tokens + completion_tokens;

        // Record token usage
        self.tracker
            .lock()
            .await
            .record(prompt_tokens, completion_tokens);

        Ok(LLMResponse {
            content: ollama_response.response,
            tokens_used: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            },
            model: ollama_response.model,
            finish_reason: if ollama_response.done {
                "stop".to_string()
            } else {
                "length".to_string()
            },
            tool_calls: None,
        })
    }

    fn info(&self) -> ProviderInfo {
        OllamaProvider::info(self)
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
            estimated_cost: 0.0, // Ollama is free (local)
            last_reset: tracker.last_reset(),
        }
    }
}
