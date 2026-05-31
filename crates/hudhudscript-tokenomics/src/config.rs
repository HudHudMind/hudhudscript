//! Configuration for tokenomics system — parsed from `[tokenomics]` in hudhud.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Helper defaults ──────────────────────────────────────────────────────────

fn default_true() -> bool {
    true
}
fn default_strategy() -> String {
    "balanced".to_string()
}
fn default_budget_amount() -> u64 {
    100_000
}
fn default_min_threshold() -> u64 {
    10_000
}
fn default_retrain_interval() -> u64 {
    3600
}
fn default_max_per_call() -> usize {
    4_000
}
fn default_max_per_day() -> usize {
    100_000
}
fn default_max_per_month() -> usize {
    3_000_000
}
fn default_alert_threshold() -> f64 {
    0.80
}
fn default_thinking_budget() -> usize {
    4096
}
fn default_min_prefix_tokens() -> usize {
    1024
}
fn default_cache_strategy() -> String {
    "exact".to_string()
}
fn default_ttl() -> u64 {
    300
}
fn default_max_entries() -> usize {
    1000
}
fn default_semantic_threshold() -> f64 {
    0.95
}
fn default_forecasting_method() -> String {
    "holt".to_string()
}
fn default_horizon() -> u64 {
    24
}
fn default_alert_action() -> String {
    "log".to_string()
}
fn default_depleted_action() -> String {
    "block".to_string()
}
fn default_flush_interval() -> u64 {
    30
}
fn default_max_batch_size() -> usize {
    100
}

fn default_thinking_tiers() -> Vec<ThinkingTier> {
    vec![
        ThinkingTier {
            name: "minimal".into(),
            tokens: 1024,
        },
        ThinkingTier {
            name: "standard".into(),
            tokens: 4096,
        },
        ThinkingTier {
            name: "deep".into(),
            tokens: 16384,
        },
        ThinkingTier {
            name: "maximum".into(),
            tokens: 65536,
        },
    ]
}

// ── Main config ──────────────────────────────────────────────────────────────

/// Tokenomics configuration — all fields optional with sensible defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenomicsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_strategy")]
    pub strategy: String,

    #[serde(default)]
    pub budget: BudgetConfig,
    #[serde(default)]
    pub pricing: HashMap<String, PricingConfig>,
    #[serde(default)]
    pub prompt_caching: PromptCachingConfig,
    #[serde(default)]
    pub cache: ResponseCacheConfig,
    #[serde(default)]
    pub optimization: OptimizationConfig,
    #[serde(default)]
    pub forecasting: ForecastingConfig,
    #[serde(default)]
    pub alerts: AlertsConfig,
    #[serde(default)]
    pub attribution: AttributionConfig,
    #[serde(default)]
    pub batch: BatchConfig,

    // ── Legacy fields (backward compatibility) ───────────────────────────
    #[serde(default = "default_budget_amount")]
    pub default_budget: u64,
    #[serde(default = "default_min_threshold")]
    pub min_threshold: u64,
    #[serde(default)]
    pub ml_enabled: bool,
    #[serde(default)]
    pub federated_learning: bool,
    #[serde(default)]
    pub reinforcement_learning: bool,
    #[serde(default = "default_retrain_interval")]
    pub retrain_interval: u64,
    #[serde(default)]
    pub redis_url: Option<String>,
    #[serde(default)]
    pub postgres_url: Option<String>,
}

impl Default for TokenomicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: default_strategy(),
            budget: BudgetConfig::default(),
            pricing: HashMap::new(),
            prompt_caching: PromptCachingConfig::default(),
            cache: ResponseCacheConfig::default(),
            optimization: OptimizationConfig::default(),
            forecasting: ForecastingConfig::default(),
            alerts: AlertsConfig::default(),
            attribution: AttributionConfig::default(),
            batch: BatchConfig::default(),
            default_budget: 100_000,
            min_threshold: 10_000,
            ml_enabled: true,
            federated_learning: false,
            reinforcement_learning: false,
            retrain_interval: 3600,
            redis_url: None,
            postgres_url: None,
        }
    }
}

// ── Sub-configs ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    #[serde(default = "default_max_per_call")]
    pub max_tokens_per_call: usize,
    #[serde(default = "default_max_per_day")]
    pub max_tokens_per_day: usize,
    #[serde(default = "default_max_per_month")]
    pub max_tokens_per_month: usize,
    #[serde(default = "default_alert_threshold")]
    pub alert_threshold: f64,
    #[serde(default = "default_thinking_budget")]
    pub thinking_budget_default: usize,
    #[serde(default = "default_thinking_tiers")]
    pub thinking_budget_tiers: Vec<ThinkingTier>,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_tokens_per_call: 4_000,
            max_tokens_per_day: 100_000,
            max_tokens_per_month: 3_000_000,
            alert_threshold: 0.80,
            thinking_budget_default: 4096,
            thinking_budget_tiers: default_thinking_tiers(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingTier {
    pub name: String,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PricingConfig {
    #[serde(default)]
    pub input_cost_per_1k: f64,
    #[serde(default)]
    pub output_cost_per_1k: f64,
    #[serde(default)]
    pub cached_input_cost_per_1k: f64,
    #[serde(default)]
    pub cache_write_cost_per_1k: f64,
    #[serde(default)]
    pub thinking_cost_per_1k: f64,
    #[serde(default)]
    pub batch_discount: f64,
    #[serde(default)]
    pub image_cost_per_token: f64,
    #[serde(default)]
    pub audio_cost_per_minute: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCachingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub auto_breakpoints: bool,
    #[serde(default = "default_min_prefix_tokens")]
    pub min_prefix_tokens: usize,
    #[serde(default)]
    pub reorder_for_cache: bool,
}

impl Default for PromptCachingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_breakpoints: false,
            min_prefix_tokens: 1024,
            reorder_for_cache: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseCacheConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_cache_strategy")]
    pub strategy: String,
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_semantic_threshold")]
    pub semantic_threshold: f64,
}

impl Default for ResponseCacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: "exact".into(),
            ttl_seconds: 300,
            max_entries: 1000,
            semantic_threshold: 0.95,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizationConfig {
    #[serde(default)]
    pub prompt_compression: bool,
    #[serde(default)]
    pub model_fallback: bool,
    #[serde(default)]
    pub fallback_model: Option<String>,
    #[serde(default)]
    pub cascade_routing: bool,
    #[serde(default)]
    pub batch_eligible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_forecasting_method")]
    pub method: String,
    #[serde(default = "default_horizon")]
    pub horizon_hours: u64,
    #[serde(default)]
    pub anomaly_detection: bool,
}

impl Default for ForecastingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            method: "holt".into(),
            horizon_hours: 24,
            anomaly_detection: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsConfig {
    #[serde(default = "default_alert_action")]
    pub on_warning: String,
    #[serde(default = "default_alert_action")]
    pub on_critical: String,
    #[serde(default = "default_depleted_action")]
    pub on_depleted: String,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self {
            on_warning: "log".into(),
            on_critical: "log".into(),
            on_depleted: "block".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttributionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub auto_promote: bool,
    #[serde(default = "default_flush_interval")]
    pub flush_interval_seconds: u64,
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_promote: false,
            flush_interval_seconds: 30,
            max_batch_size: 100,
        }
    }
}
