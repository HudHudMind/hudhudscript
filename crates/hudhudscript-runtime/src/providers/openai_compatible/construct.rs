//! OpenAI-compatible provider construction and helpers.

use crate::provider::{
    error::ProviderError, registry::TokenTracker, types::ProviderConfig, ProviderType,
};
use crate::providers::http_client::shared_http_client;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::providers::openai_compatible::defaults::get_provider_defaults;

/// Generic OpenAI-compatible provider
pub struct OpenAICompatibleProvider {
    pub(crate) config: ProviderConfig,
    pub(crate) base_url: String,
    pub(crate) client: Client,
    pub(crate) token_tracker: Arc<RwLock<TokenTracker>>,
}

impl OpenAICompatibleProvider {
    pub fn new(config: ProviderConfig, base_url: String) -> Result<Self, ProviderError> {
        // Local providers (Ollama local, LM Studio) and Http type don't need an API key
        // Ollama Cloud (ollama.com) does require an API key
        let is_local_or_no_auth = matches!(config.provider_type, ProviderType::Http)
            || (matches!(config.provider_type, ProviderType::Ollama)
                && crate::providers::http_client::is_local_url(&base_url));
        if config.api_key.is_none() && !is_local_or_no_auth {
            return Err(ProviderError::InvalidConfig(format!(
                "API key is required for {:?} provider",
                config.provider_type
            )));
        }
        Ok(Self {
            config,
            base_url,
            client: shared_http_client()?,
            token_tracker: Arc::new(RwLock::new(TokenTracker::new())),
        })
    }

    /// Create from provider name — looks up defaults automatically
    pub fn from_name(
        name: &str,
        api_key: String,
        model_override: Option<String>,
        timeout_secs: Option<u64>,
        endpoint_override: Option<String>,
    ) -> Result<Self, ProviderError> {
        let defaults = get_provider_defaults(name)
            .ok_or_else(|| ProviderError::InvalidConfig(format!("Unknown provider: {}", name)))?;

        let base_url = endpoint_override.unwrap_or_else(|| defaults.base_url.to_string());

        let config = ProviderConfig {
            provider_type: defaults.provider_type,
            model: model_override.unwrap_or_else(|| defaults.default_model.to_string()),
            api_key: Some(api_key),
            endpoint: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
            budget: None,
            timeout_secs,
            extra: std::collections::HashMap::new(),
        };

        Ok(Self {
            config,
            base_url,
            client: shared_http_client()?,
            token_tracker: Arc::new(RwLock::new(TokenTracker::new())),
        })
    }

    /// Resolve the API key, returning `None` for local/no-auth endpoints.
    /// Returns an error if authentication is required but no key is configured.
    pub(crate) fn resolve_api_key(&self, no_auth: bool) -> Result<Option<String>, ProviderError> {
        match &self.config.api_key {
            Some(key) if !key.is_empty() => Ok(Some(key.clone())),
            Some(_) if no_auth => Ok(None),
            Some(_) => Err(ProviderError::ApiError(format!(
                "API key is empty for provider {:?}. Set the required environment variable.",
                self.config.provider_type
            ))),
            None if no_auth => Ok(None),
            None => Err(ProviderError::ApiError(format!(
                "No API key configured for provider {:?}. Set the required environment variable.",
                self.config.provider_type
            ))),
        }
    }
}
