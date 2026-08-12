//! Provider data types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const DEFAULT_PROVIDER_TIMEOUT_SECS: u64 = 120;

/// Provider type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    /// OpenAI (GPT-4, GPT-4o, etc.)
    OpenAI,
    /// Anthropic (Claude)
    Anthropic,
    /// Ollama (local models)
    Ollama,
    /// DeepSeek (deepseek-chat, deepseek-reasoner)
    DeepSeek,
    /// Google Gemini
    Gemini,
    /// Mistral AI
    Mistral,
    /// Groq (fast inference)
    Groq,
    /// Cohere
    Cohere,
    /// Together AI
    Together,
    /// xAI (Grok)
    XAI,
    /// OpenRouter (multi-provider gateway)
    OpenRouter,
    /// Custom HTTP endpoint (OpenAI-compatible)
    Http,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider type
    pub provider_type: ProviderType,

    /// Model name
    pub model: String,

    /// API key (optional, for authenticated providers)
    pub api_key: Option<String>,

    /// Endpoint URL (optional, for custom endpoints)
    pub endpoint: Option<String>,

    /// Default temperature
    pub temperature: Option<f64>,

    /// Default max tokens
    pub max_tokens: Option<usize>,

    /// Token budget configuration
    pub budget: Option<TokenBudget>,

    /// Timeout in seconds
    pub timeout_secs: Option<u64>,

    /// Extra provider-specific configuration
    pub extra: HashMap<String, serde_json::Value>,
}

/// Token budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    /// Maximum tokens per single call
    pub max_tokens_per_call: usize,

    /// Maximum tokens per day
    pub max_tokens_per_day: usize,

    /// Alert threshold (0.0-1.0)
    pub alert_threshold: f64,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_tokens_per_call: 4000,
            max_tokens_per_day: 100000,
            alert_threshold: 0.8,
        }
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageStats {
    /// Total tokens used today
    pub daily_usage: usize,

    /// Total tokens used this month
    pub monthly_usage: usize,

    /// Total cost in USD (estimated)
    pub estimated_cost: f64,

    /// Last reset time
    pub last_reset: std::time::SystemTime,
}

/// Provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider name
    pub name: String,

    /// Model name
    pub model: String,

    /// Provider type
    pub provider_type: ProviderType,
}

#[cfg(test)]
mod provider_timeout_tests {
    use super::*;

    fn get_effective_timeout(req: Option<u64>, cfg: Option<u64>) -> u64 {
        req.or(cfg).unwrap_or(DEFAULT_PROVIDER_TIMEOUT_SECS)
    }

    #[test]
    fn test_provider_timeout_effective_logic() {
        assert_eq!(get_effective_timeout(Some(300), Some(120)), 300);
        assert_eq!(get_effective_timeout(None, Some(300)), 300);
        assert_eq!(get_effective_timeout(Some(180), None), 180);
        assert_eq!(get_effective_timeout(None, None), DEFAULT_PROVIDER_TIMEOUT_SECS);
    }
}
