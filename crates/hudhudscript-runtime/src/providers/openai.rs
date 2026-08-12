//! OpenAI Provider Implementation

use crate::provider::*;
use crate::providers::http_client::{shared_http_client, send_with_retry};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

/// OpenAI provider for GPT models
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: Client,
    token_tracker: Arc<RwLock<TokenTracker>>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: ProviderConfig) -> Result<Self, ProviderError> {
        // Validate configuration
        if config.provider_type != ProviderType::OpenAI {
            return Err(ProviderError::InvalidConfig(
                "Provider type must be OpenAI".to_string(),
            ));
        }

        if config.api_key.is_none() {
            return Err(ProviderError::InvalidConfig(
                "API key is required for OpenAI provider".to_string(),
            ));
        }

        Ok(Self {
            config,
            client: shared_http_client()?,
            token_tracker: Arc::new(RwLock::new(TokenTracker::new())),
        })
    }

    /// Optimize prompt with mnemonics
    pub fn optimize_with_mnemonics(&self, request: &LLMRequest) -> Result<String, ProviderError> {
        if !request.optimize || request.mnemonics.is_none() {
            return Ok(request.prompt.clone());
        }

        let mnemonics = match request.mnemonics.as_ref() {
            Some(m) => m,
            None => return Ok(request.prompt.clone()),
        };
        let mut optimized = request.prompt.clone();

        // Replace verbose instructions with mnemonics
        for (mnemonic, expansion) in mnemonics {
            optimized = optimized.replace(expansion, mnemonic);
        }

        // Add mnemonic dictionary to system prompt
        let dictionary = mnemonics
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!("{}\n\n[Mnemonics: {}]", optimized, dictionary))
    }

    /// Build messages array for OpenAI API (public for testing)
    pub fn build_messages(
        &self,
        request: &LLMRequest,
    ) -> Result<Vec<serde_json::Value>, ProviderError> {
        let mut messages = Vec::new();

        // Add system prompt if provided
        if let Some(system_prompt) = &request.system_prompt {
            messages.push(json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        // Add mnemonic dictionary to system prompt if optimizing
        if request.optimize {
            if let Some(mnemonics) = request.mnemonics.as_ref() {
                let dictionary = mnemonics
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                messages.push(json!({
                    "role": "system",
                    "content": format!("Mnemonic Dictionary:\n{}", dictionary)
                }));
            }
        }

        // Add user prompt (optimized if requested)
        let prompt = self.optimize_with_mnemonics(request)?;
        messages.push(json!({
            "role": "user",
            "content": prompt
        }));

        Ok(messages)
    }

    /// Return provider information.
    pub fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "OpenAI".to_string(),
            model: self.config.model.clone(),
            provider_type: ProviderType::OpenAI,
        }
    }

    /// Check token budget.
    pub fn check_budget(&self, tokens: usize) -> Result<(), ProviderError> {
        if let Some(budget) = &self.config.budget {
            // Use blocking read for sync context
            let tracker = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { self.token_tracker.read().await })
            });

            if tokens > budget.max_tokens_per_call {
                return Err(ProviderError::BudgetExceeded {
                    limit: budget.max_tokens_per_call,
                    requested: tokens,
                });
            }

            let daily_usage = tracker.daily_usage();
            if daily_usage + tokens > budget.max_tokens_per_day {
                return Err(ProviderError::DailyBudgetExceeded {
                    limit: budget.max_tokens_per_day,
                    current: daily_usage,
                });
            }

            let usage_ratio = (daily_usage + tokens) as f64 / budget.max_tokens_per_day as f64;
            if usage_ratio > budget.alert_threshold {
                warn!(
                    "Token budget alert: {}% used",
                    (usage_ratio * 100.0) as usize
                );
            }
        }
        Ok(())
    }

    /// Get current usage statistics.
    pub async fn get_usage_stats(&self) -> TokenUsageStats {
        self.token_tracker.read().await.get_stats()
    }
}

#[async_trait::async_trait]
impl Provider for OpenAIProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        // Estimate tokens and check budget
        let estimated_tokens = estimate_tokens(&request.prompt);
        self.check_budget(estimated_tokens)?;

        // Build messages
        let messages = self.build_messages(&request)?;

        // Build API request
        let api_request = json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": request.temperature.or(self.config.temperature).unwrap_or(0.7),
            "max_tokens": request.max_tokens.or(self.config.max_tokens).unwrap_or(2000),
        });

        // Call OpenAI API (retry once on 5xx)
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            ProviderError::NotConfigured("OpenAI API key is required".to_string())
        })?;
        let timeout_secs = request.timeout_secs.or(self.config.timeout_secs).unwrap_or(crate::provider::types::DEFAULT_PROVIDER_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);
        
        let build_req =
            || {
                self.client
                    .post(std::env::var("OPENAI_API_BASE").unwrap_or_else(|_| {
                        "https://api.openai.com/v1/chat/completions".to_string()
                    }))
                    .timeout(timeout_duration)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&api_request)
            };
        let response = send_with_retry(build_req(), build_req()).await?;

        // Check for HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(ProviderError::ApiError(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        // Parse response
        let api_response: serde_json::Value = response.json().await?;

        // Extract content (may be empty when only tool calls are returned)
        let content = api_response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Extract token usage
        let usage = &api_response["usage"];
        let tokens_used = TokenUsage {
            prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as usize,
            completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as usize,
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as usize,
        };

        // Track token usage
        self.token_tracker
            .write()
            .await
            .record(tokens_used.prompt_tokens, tokens_used.completion_tokens);

        // Extract finish reason
        let finish_reason = api_response["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("stop")
            .to_string();

        // Extract tool calls if any (multi-turn tool calling)
        let tool_calls = api_response["choices"][0]["message"]["tool_calls"]
            .as_array()
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|tc| {
                        let id = tc["id"].as_str()?.to_string();
                        let name = tc["function"]["name"].as_str()?.to_string();
                        // Arguments come as JSON string per OpenAI API
                        let args_str = tc["function"]["arguments"].as_str()?;
                        let arguments =
                            serde_json::from_str(args_str).unwrap_or(serde_json::Value::Null);
                        Some(crate::provider::LLMToolCall {
                            id,
                            name,
                            arguments,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|v| !v.is_empty());

        Ok(LLMResponse {
            content,
            tokens_used,
            model: self.config.model.clone(),
            finish_reason,
            tool_calls,
        })
    }

    fn info(&self) -> ProviderInfo {
        OpenAIProvider::info(self)
    }

    fn check_budget(&self, tokens: usize) -> Result<(), ProviderError> {
        OpenAIProvider::check_budget(self, tokens)
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        Ok(vec![])
    }

    async fn get_usage_stats(&self) -> TokenUsageStats {
        OpenAIProvider::get_usage_stats(self).await
    }
}
