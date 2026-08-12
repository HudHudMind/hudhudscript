//! Non-streaming Provider trait methods for OpenAI-compatible providers.

use crate::provider::{
    error::ProviderError,
    registry::estimate_tokens,
    request::{LLMRequest, LLMResponse, TokenUsage},
    traits::Provider,
    types::{ProviderInfo, TokenUsageStats},
};
use crate::providers::http_client::{is_local_url, send_with_retry};
use crate::providers::openai_compatible::construct::OpenAICompatibleProvider;
use serde_json::json;
use tokio::io::AsyncBufReadExt;
use tracing::{debug, warn};

#[async_trait::async_trait]
impl Provider for OpenAICompatibleProvider {
    async fn call(&self, request: LLMRequest) -> Result<LLMResponse, ProviderError> {
        let estimated_tokens = estimate_tokens(&request.prompt);
        self.check_budget(estimated_tokens)?;

        let mut messages = Vec::new();
        if let Some(sys) = &request.system_prompt {
            messages.push(json!({ "role": "system", "content": sys }));
        }
        messages.push(json!({ "role": "user", "content": request.prompt }));

        let mut api_request = json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": request.temperature.or(self.config.temperature).unwrap_or(0.7),
            "max_tokens": request.max_tokens.or(self.config.max_tokens).unwrap_or(2000),
        });

        // Attach tools if provided (OpenAI function calling format)
        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                let tools_json: Vec<serde_json::Value> = tools
                    .iter()
                    .map(|t| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": t.name,
                                "description": t.description,
                                "parameters": t.parameters,
                            }
                        })
                    })
                    .collect();
                api_request["tools"] = json!(tools_json);
                api_request["tool_choice"] = json!("auto");
            }
        }

        let url = format!("{}/chat/completions", self.base_url);

        debug!(
            "[HudHud] → Provider call: {:?} | model: {} | url: {}",
            self.config.provider_type, self.config.model, url
        );

        let no_auth = matches!(
            self.config.provider_type,
            crate::provider::ProviderType::Http
        ) || is_local_url(&url);
        let api_key = self.resolve_api_key(no_auth)?;

        let timeout_secs = request.timeout_secs.or(self.config.timeout_secs).unwrap_or(crate::provider::types::DEFAULT_PROVIDER_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        let build_req = || {
            let mut req = self
                .client
                .post(&url)
                .timeout(timeout_duration)
                .header("Content-Type", "application/json")
                .json(&api_request);
            if let Some(ref key) = api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            req
        };

        let response = send_with_retry(build_req(), build_req()).await?;

        let status = response.status();
        debug!("[HudHud] ← Response status: {}", status);

        if !status.is_success() {
            let err = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            warn!("[HudHud] ✗ API error: {}", err);
            return Err(ProviderError::ApiError(format!(
                "{:?} API error ({}): {}",
                self.config.provider_type, status, err
            )));
        }

        let api_response: serde_json::Value = response.json().await?;

        let finish_reason = api_response["choices"][0]["finish_reason"]
            .as_str()
            .unwrap_or("stop")
            .to_string();

        // Parse tool calls if present
        let tool_calls_raw = api_response["choices"][0]["message"]["tool_calls"].as_array();
        let tool_calls = tool_calls_raw
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|tc| {
                        let id = tc["id"].as_str()?.to_string();
                        let name = tc["function"]["name"].as_str()?.to_string();
                        let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                        // LLM tool-call arguments are sometimes malformed JSON.
                        // Log the parse failure but include the raw string in a
                        // wrapper object so callers don't lose the data.
                        let arguments = serde_json::from_str(args_str).unwrap_or_else(|e| {
                            tracing::warn!(
                                "Tool call '{}' has invalid JSON arguments: {} — wrapping raw text",
                                name,
                                e
                            );
                            json!({ "_raw_arguments": args_str, "_parse_error": e.to_string() })
                        });
                        Some(crate::provider::LLMToolCall {
                            id,
                            name,
                            arguments,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|v: &Vec<_>| !v.is_empty());

        let content = if tool_calls.is_some() {
            // LLM wants to call tools — content may be empty
            api_response["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string()
        } else {
            api_response["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| ProviderError::ApiError("No content in response".to_string()))?
                .to_string()
        };

        let usage = &api_response["usage"];
        let tokens_used = TokenUsage {
            prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as usize,
            completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as usize,
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as usize,
        };

        self.token_tracker
            .write()
            .await
            .record(tokens_used.prompt_tokens, tokens_used.completion_tokens);

        Ok(LLMResponse {
            content,
            tokens_used,
            model: self.config.model.clone(),
            finish_reason,
            tool_calls,
        })
    }

    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: format!("{:?}", self.config.provider_type),
            model: self.config.model.clone(),
            provider_type: self.config.provider_type.clone(),
        }
    }

    async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
        // Ollama Cloud uses /api/tags instead of /v1/models
        let is_ollama_cloud = self.base_url.contains("ollama.com");
        let url = if is_ollama_cloud {
            "https://ollama.com/api/tags".to_string()
        } else {
            format!("{}/models", self.base_url)
        };
        debug!("[HudHud] → list_models: {}", url);

        let no_auth = matches!(
            self.config.provider_type,
            crate::provider::ProviderType::Http
        ) || is_local_url(&url);
        let api_key = self.resolve_api_key(no_auth)?;

        let timeout_secs = self.config.timeout_secs.unwrap_or(crate::provider::types::DEFAULT_PROVIDER_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        let build_req = || {
            let mut req = self
                .client
                .get(&url)
                .timeout(timeout_duration)
                .header("Content-Type", "application/json");
            if let Some(ref key) = api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            req
        };

        let response = send_with_retry(build_req(), build_req()).await?;
        let status = response.status();
        if !status.is_success() {
            let err = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(ProviderError::ApiError(format!(
                "list_models failed ({}): {}",
                status, err
            )));
        }

        let body: serde_json::Value = response.json().await?;

        // Standard OpenAI format: { "data": [{ "id": "model-name" }, ...] }
        // Ollama format: { "models": [{ "name": "llama3.2" }, ...] }
        let models = if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            data.iter()
                .filter_map(|m| {
                    m.get("id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        } else if let Some(models) = body.get("models").and_then(|m| m.as_array()) {
            models
                .iter()
                .filter_map(|m| {
                    m.get("name")
                        .or_else(|| m.get("id"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        } else {
            vec![]
        };

        debug!("[HudHud] ← {} models found", models.len());
        Ok(models)
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

    async fn get_usage_stats(&self) -> TokenUsageStats {
        self.token_tracker.read().await.get_stats()
    }

    async fn stream_call(
        &self,
        request: LLMRequest,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<LLMResponse, ProviderError> {
        let estimated_tokens = estimate_tokens(&request.prompt);
        self.check_budget(estimated_tokens)?;

        let mut messages = Vec::new();
        if let Some(sys) = &request.system_prompt {
            messages.push(json!({ "role": "system", "content": sys }));
        }
        messages.push(json!({ "role": "user", "content": request.prompt }));

        let api_request = json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": request.temperature.or(self.config.temperature).unwrap_or(0.7),
            "max_tokens": request.max_tokens.or(self.config.max_tokens).unwrap_or(2000),
            "stream": true,
        });

        let url = format!("{}/chat/completions", self.base_url);

        let no_auth = matches!(
            self.config.provider_type,
            crate::provider::ProviderType::Http
        ) || is_local_url(&url);
        let api_key = self.resolve_api_key(no_auth)?;

        let timeout_secs = request.timeout_secs.or(self.config.timeout_secs).unwrap_or(crate::provider::types::DEFAULT_PROVIDER_TIMEOUT_SECS);
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);

        let mut req = self
            .client
            .post(&url)
            .timeout(timeout_duration)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&api_request);

        if let Some(ref key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        // Streaming does not retry — mid-stream retries would produce duplicate content
        let response = req.send().await?;
        let status = response.status();
        if !status.is_success() {
            let err = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(ProviderError::ApiError(format!(
                "stream_call failed ({}): {}",
                status, err
            )));
        }

        let mut full_content = String::new();
        let mut total_tokens = 0usize;
        let mut prompt_tokens = 0usize;

        // Read SSE lines
        let bytes = response.bytes_stream();
        use futures_util::TryStreamExt;
        use tokio_util::io::StreamReader;

        let stream = bytes.map_err(std::io::Error::other);
        let reader = StreamReader::new(stream);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim().to_string();
            if line.is_empty() || line == "data: [DONE]" {
                continue;
            }
            let data = line.strip_prefix("data: ").unwrap_or(&line);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                // Extract delta content — copy to owned String to avoid borrow issues
                let delta_owned = json["choices"][0]["delta"]["content"]
                    .as_str()
                    .map(|s| s.to_string());
                if let Some(delta) = delta_owned {
                    if !delta.is_empty() {
                        on_chunk(delta.clone());
                        full_content.push_str(&delta);
                    }
                }
                // Extract usage if present (some providers send at end)
                if let Some(usage) = json.get("usage") {
                    total_tokens = usage["total_tokens"].as_u64().unwrap_or(0) as usize;
                    prompt_tokens = usage["prompt_tokens"].as_u64().unwrap_or(0) as usize;
                }
            }
        }

        self.token_tracker.write().await.record(prompt_tokens, total_tokens);

        Ok(LLMResponse {
            content: full_content.clone(),
            tokens_used: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens,
            },
            model: self.config.model.clone(),
            finish_reason: "stop".to_string(),
            tool_calls: None,
        })
    }
}
