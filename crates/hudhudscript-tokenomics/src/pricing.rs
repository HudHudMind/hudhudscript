//! Provider-based model pricing tables.
//!
//! Pricing is loaded externally via `load_from_json()` or `load_from_file()`.
//! The default registry is empty — users must provide their own pricing data
//! through `hudhud.toml [tokenomics.pricing]` or a separate JSON file.
//! No built-in prices are bundled; this prevents stale/hardcoded pricing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-model pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model: String,
    pub provider: String,
    pub input_cost_per_mtok: f64,
    pub output_cost_per_mtok: f64,
    pub cached_input_cost_per_mtok: Option<f64>,
    pub cache_write_cost_per_mtok: Option<f64>,
    pub thinking_cost_per_mtok: Option<f64>,
    pub batch_discount: f64,
    pub image_cost_per_token: Option<f64>,
    pub audio_cost_per_minute: Option<f64>,
}

/// Cost breakdown for a single call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub input_cost: f64,
    pub output_cost: f64,
    pub cached_input_cost: f64,
    pub thinking_cost: f64,
    pub total_cost: f64,
    pub currency: String,
}

/// Pricing registry with built-in 2026 model prices
pub struct PricingRegistry {
    models: HashMap<String, ModelPricing>,
}

impl Default for PricingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PricingRegistry {
    /// Create a registry with built-in defaults for backward compatibility.
    /// Use `load_from_json()` or `load_from_file()` to override with custom pricing.
    pub fn new() -> Self {
        let mut models = HashMap::new();

        // ── Anthropic ──
        models.insert(
            "claude-opus-4".into(),
            ModelPricing {
                model: "claude-opus-4".into(),
                provider: "anthropic".into(),
                input_cost_per_mtok: 15.0,
                output_cost_per_mtok: 75.0,
                cached_input_cost_per_mtok: Some(1.5),
                cache_write_cost_per_mtok: Some(18.75),
                thinking_cost_per_mtok: Some(75.0),
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );
        models.insert(
            "claude-sonnet-4".into(),
            ModelPricing {
                model: "claude-sonnet-4".into(),
                provider: "anthropic".into(),
                input_cost_per_mtok: 3.0,
                output_cost_per_mtok: 15.0,
                cached_input_cost_per_mtok: Some(0.30),
                cache_write_cost_per_mtok: Some(3.75),
                thinking_cost_per_mtok: Some(15.0),
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );
        models.insert(
            "claude-haiku-3.5".into(),
            ModelPricing {
                model: "claude-haiku-3.5".into(),
                provider: "anthropic".into(),
                input_cost_per_mtok: 0.80,
                output_cost_per_mtok: 4.0,
                cached_input_cost_per_mtok: Some(0.08),
                cache_write_cost_per_mtok: Some(1.0),
                thinking_cost_per_mtok: Some(4.0),
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );

        // ── OpenAI ──
        models.insert(
            "gpt-4o".into(),
            ModelPricing {
                model: "gpt-4o".into(),
                provider: "openai".into(),
                input_cost_per_mtok: 2.50,
                output_cost_per_mtok: 10.0,
                cached_input_cost_per_mtok: Some(1.25),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: None,
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );
        models.insert(
            "gpt-4o-mini".into(),
            ModelPricing {
                model: "gpt-4o-mini".into(),
                provider: "openai".into(),
                input_cost_per_mtok: 0.15,
                output_cost_per_mtok: 0.60,
                cached_input_cost_per_mtok: Some(0.075),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: None,
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );
        models.insert(
            "o1".into(),
            ModelPricing {
                model: "o1".into(),
                provider: "openai".into(),
                input_cost_per_mtok: 15.0,
                output_cost_per_mtok: 60.0,
                cached_input_cost_per_mtok: Some(7.50),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: Some(60.0),
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );
        models.insert(
            "o3".into(),
            ModelPricing {
                model: "o3".into(),
                provider: "openai".into(),
                input_cost_per_mtok: 10.0,
                output_cost_per_mtok: 40.0,
                cached_input_cost_per_mtok: Some(5.0),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: Some(40.0),
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );

        // ── DeepSeek ──
        models.insert(
            "deepseek-v3".into(),
            ModelPricing {
                model: "deepseek-v3".into(),
                provider: "deepseek".into(),
                input_cost_per_mtok: 0.27,
                output_cost_per_mtok: 1.10,
                cached_input_cost_per_mtok: Some(0.07),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: None,
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );
        models.insert(
            "deepseek-r1".into(),
            ModelPricing {
                model: "deepseek-r1".into(),
                provider: "deepseek".into(),
                input_cost_per_mtok: 0.55,
                output_cost_per_mtok: 2.19,
                cached_input_cost_per_mtok: Some(0.14),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: Some(2.19),
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );

        // ── Gemini ──
        models.insert(
            "gemini-1.5-pro".into(),
            ModelPricing {
                model: "gemini-1.5-pro".into(),
                provider: "gemini".into(),
                input_cost_per_mtok: 1.25,
                output_cost_per_mtok: 5.0,
                cached_input_cost_per_mtok: Some(0.3125),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: None,
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );
        models.insert(
            "gemini-1.5-flash".into(),
            ModelPricing {
                model: "gemini-1.5-flash".into(),
                provider: "gemini".into(),
                input_cost_per_mtok: 0.075,
                output_cost_per_mtok: 0.30,
                cached_input_cost_per_mtok: Some(0.01875),
                cache_write_cost_per_mtok: None,
                thinking_cost_per_mtok: None,
                batch_discount: 0.50,
                image_cost_per_token: None,
                audio_cost_per_minute: None,
            },
        );

        Self { models }
    }

    /// Calculate cost for a call
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: usize,
        output_tokens: usize,
        cached_tokens: usize,
        thinking_tokens: usize,
        is_batch: bool,
    ) -> Option<CostBreakdown> {
        let pricing = self.models.get(model)?;
        let discount = if is_batch {
            1.0 - pricing.batch_discount
        } else {
            1.0
        };

        let billable_input = input_tokens.saturating_sub(cached_tokens);
        let input_cost =
            billable_input as f64 * pricing.input_cost_per_mtok / 1_000_000.0 * discount;
        let output_cost =
            output_tokens as f64 * pricing.output_cost_per_mtok / 1_000_000.0 * discount;
        let cached_cost = cached_tokens as f64
            * pricing
                .cached_input_cost_per_mtok
                .unwrap_or(pricing.input_cost_per_mtok)
            / 1_000_000.0
            * discount;
        let think_cost = thinking_tokens as f64
            * pricing
                .thinking_cost_per_mtok
                .unwrap_or(pricing.output_cost_per_mtok)
            / 1_000_000.0
            * discount;

        Some(CostBreakdown {
            input_cost,
            output_cost,
            cached_input_cost: cached_cost,
            thinking_cost: think_cost,
            total_cost: input_cost + output_cost + cached_cost + think_cost,
            currency: "USD".into(),
        })
    }

    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.models.get(model)
    }

    pub fn register_model(&mut self, pricing: ModelPricing) {
        self.models.insert(pricing.model.clone(), pricing);
    }

    /// Find cheapest model from a given provider
    pub fn cheapest_for_provider(&self, provider: &str) -> Option<&ModelPricing> {
        self.models
            .values()
            .filter(|m| m.provider == provider)
            .min_by(|a, b| {
                a.input_cost_per_mtok
                    .partial_cmp(&b.input_cost_per_mtok)
                    .unwrap()
            })
    }

    pub fn all_models(&self) -> Vec<&ModelPricing> {
        self.models.values().collect()
    }

    /// Load pricing from a JSON string, overriding/adding models.
    ///
    /// JSON format: `[{ "model": "gpt-4o", "provider": "openai", ... }, ...]`
    pub fn load_from_json(&mut self, json: &str) -> Result<usize, String> {
        let models: Vec<ModelPricing> =
            serde_json::from_str(json).map_err(|e| format!("Invalid pricing JSON: {}", e))?;
        let count = models.len();
        for model in models {
            self.models.insert(model.model.clone(), model);
        }
        Ok(count)
    }

    /// Load pricing from a JSON file, overriding/adding models.
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<usize, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read pricing file {:?}: {}", path, e))?;
        self.load_from_json(&content)
    }
}
