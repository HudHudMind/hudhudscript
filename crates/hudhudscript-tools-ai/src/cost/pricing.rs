use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported AI providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Ollama,
    DeepSeek,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::OpenAI => write!(f, "OpenAI"),
            Provider::Anthropic => write!(f, "Anthropic"),
            Provider::Ollama => write!(f, "Ollama"),
            Provider::DeepSeek => write!(f, "DeepSeek"),
        }
    }
}

/// Per-model pricing expressed in USD per 1 000 tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Provider that offers this model.
    pub provider: Provider,
    /// Model identifier (e.g. `"gpt-4o"`, `"claude-3-opus"`).
    pub model: String,
    /// Cost per 1 000 *input* tokens (USD).
    pub input_cost_per_1k: f64,
    /// Cost per 1 000 *output* tokens (USD).
    pub output_cost_per_1k: f64,
}

/// Build the default pricing table.
///
/// Prices are approximate list prices as of early 2026. Users may override
/// them via [`CostTracker::set_model_pricing`].
pub fn default_pricing() -> HashMap<String, ModelPricing> {
    let entries = vec![
        ModelPricing {
            provider: Provider::OpenAI,
            model: "gpt-4o".into(),
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.015,
        },
        ModelPricing {
            provider: Provider::OpenAI,
            model: "gpt-4o-mini".into(),
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0006,
        },
        ModelPricing {
            provider: Provider::OpenAI,
            model: "gpt-4-turbo".into(),
            input_cost_per_1k: 0.01,
            output_cost_per_1k: 0.03,
        },
        ModelPricing {
            provider: Provider::Anthropic,
            model: "claude-3-opus".into(),
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
        },
        ModelPricing {
            provider: Provider::Anthropic,
            model: "claude-3-sonnet".into(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
        ModelPricing {
            provider: Provider::Anthropic,
            model: "claude-3-haiku".into(),
            input_cost_per_1k: 0.00025,
            output_cost_per_1k: 0.00125,
        },
        ModelPricing {
            provider: Provider::Ollama,
            model: "ollama-local".into(),
            input_cost_per_1k: 0.0,
            output_cost_per_1k: 0.0,
        },
        ModelPricing {
            provider: Provider::DeepSeek,
            model: "deepseek-chat".into(),
            input_cost_per_1k: 0.00014,
            output_cost_per_1k: 0.00028,
        },
        ModelPricing {
            provider: Provider::DeepSeek,
            model: "deepseek-coder".into(),
            input_cost_per_1k: 0.00014,
            output_cost_per_1k: 0.00028,
        },
    ];

    entries.into_iter().map(|p| (p.model.clone(), p)).collect()
}
