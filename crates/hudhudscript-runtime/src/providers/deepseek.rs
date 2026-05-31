use crate::provider::ProviderError;
use crate::providers::http_client::{shared_http_client, send_with_retry};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DeepSeekProvider {
    api_key: String,
    base_url: String,
    /// Pre-built HTTP client with timeout; reused across calls.
    client: Client,
}

#[derive(Debug, Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekResponse {
    /// Populated by the DeepSeek API but not used directly; kept for deserialization.
    #[serde(rename = "id")]
    _id: String,
    /// Populated by the DeepSeek API but not used directly; kept for deserialization.
    #[serde(rename = "object")]
    _object: String,
    /// Populated by the DeepSeek API but not used directly; kept for deserialization.
    #[serde(rename = "created")]
    _created: u64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    /// Populated by the DeepSeek API but not used directly; kept for deserialization.
    #[serde(rename = "index")]
    _index: u32,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl DeepSeekProvider {
    /// Construct a new DeepSeek provider.
    ///
    /// Returns an error if the underlying HTTP client cannot be built
    /// (e.g., system TLS / DNS misconfiguration). Previously this silently
    /// fell back to `Client::new()`, losing the configured timeout/retry
    /// settings — fixed in v0.4.47.9.
    pub fn new(api_key: String) -> std::result::Result<Self, ProviderError> {
        let client = shared_http_client().map_err(|e| {
            ProviderError::InvalidConfig(format!(
                "Failed to build HTTP client for DeepSeek provider: {}",
                e
            ))
        })?;
        Ok(Self {
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
            client,
        })
    }

    /// Convenience constructor for backward compatibility — panics on failure.
    /// New code should use [`DeepSeekProvider::new`] which returns a Result.
    pub fn new_or_panic(api_key: String) -> Self {
        Self::new(api_key).expect("DeepSeekProvider::new_or_panic: failed to build HTTP client")
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub async fn call(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<DeepSeekCallResponse, ProviderError> {
        let request = DeepSeekRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature,
            max_tokens,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: false,
        };

        let url = format!("{}/chat/completions", self.base_url);
        let build_req = || {
            self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
        };
        let response = send_with_retry(build_req(), build_req())
            .await
            .map_err(|e| ProviderError::NetworkError(format!("DeepSeek request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(ProviderError::ApiError(format!(
                "DeepSeek API error {}: {}",
                status, error_text
            )));
        }

        let deepseek_response: DeepSeekResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::SerializationError(format!("DeepSeek JSON: {}", e)))?;

        Ok(DeepSeekCallResponse {
            content: deepseek_response.choices[0].message.content.clone(),
            model: deepseek_response.model,
            tokens_used: deepseek_response.usage.total_tokens,
            prompt_tokens: deepseek_response.usage.prompt_tokens,
            completion_tokens: deepseek_response.usage.completion_tokens,
            finish_reason: deepseek_response.choices[0].finish_reason.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeepSeekCallResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: u32,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub finish_reason: String,
}
