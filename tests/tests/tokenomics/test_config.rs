//! Public API tests for `hudhudscript_tokenomics::config`.
//! Tests cover TokenomicsConfig and all public sub-config structs.

use hudhudscript_tokenomics::config::*;

// ── Default construction: TokenomicsConfig ───────────────────────────

#[test]
fn test_default_config_enabled() {
    let config = TokenomicsConfig::default();
    assert!(config.enabled);
}

#[test]
fn test_default_config_strategy() {
    let config = TokenomicsConfig::default();
    assert_eq!(config.strategy, "balanced");
}

#[test]
fn test_default_config_budget_limits() {
    let config = TokenomicsConfig::default();
    assert_eq!(config.budget.max_tokens_per_call, 4_000);
    assert_eq!(config.budget.max_tokens_per_day, 100_000);
    assert_eq!(config.budget.max_tokens_per_month, 3_000_000);
    assert_eq!(config.budget.alert_threshold, 0.80);
    assert_eq!(config.budget.thinking_budget_default, 4096);
}

#[test]
fn test_default_thinking_tiers_count() {
    let config = TokenomicsConfig::default();
    assert_eq!(config.budget.thinking_budget_tiers.len(), 4);
}

#[test]
fn test_default_thinking_tiers_values() {
    let tiers = &TokenomicsConfig::default().budget.thinking_budget_tiers;
    assert_eq!(tiers[0].name, "minimal");
    assert_eq!(tiers[0].tokens, 1024);
    assert_eq!(tiers[1].name, "standard");
    assert_eq!(tiers[1].tokens, 4096);
    assert_eq!(tiers[2].name, "deep");
    assert_eq!(tiers[2].tokens, 16384);
    assert_eq!(tiers[3].name, "maximum");
    assert_eq!(tiers[3].tokens, 65536);
}

#[test]
fn test_default_prompt_caching() {
    let config = TokenomicsConfig::default();
    assert!(!config.prompt_caching.enabled);
    assert!(!config.prompt_caching.auto_breakpoints);
    assert_eq!(config.prompt_caching.min_prefix_tokens, 1024);
    assert!(!config.prompt_caching.reorder_for_cache);
}

#[test]
fn test_default_response_cache() {
    let config = TokenomicsConfig::default();
    assert!(!config.cache.enabled);
    assert_eq!(config.cache.strategy, "exact");
    assert_eq!(config.cache.ttl_seconds, 300);
    assert_eq!(config.cache.max_entries, 1000);
    assert_eq!(config.cache.semantic_threshold, 0.95);
}

#[test]
fn test_default_optimization() {
    let config = TokenomicsConfig::default();
    assert!(!config.optimization.prompt_compression);
    assert!(!config.optimization.model_fallback);
    assert!(config.optimization.fallback_model.is_none());
    assert!(!config.optimization.cascade_routing);
    assert!(!config.optimization.batch_eligible);
}

#[test]
fn test_default_forecasting() {
    let config = TokenomicsConfig::default();
    assert!(!config.forecasting.enabled);
    assert_eq!(config.forecasting.method, "holt");
    assert_eq!(config.forecasting.horizon_hours, 24);
    assert!(!config.forecasting.anomaly_detection);
}

#[test]
fn test_default_alerts() {
    let config = TokenomicsConfig::default();
    assert_eq!(config.alerts.on_warning, "log");
    assert_eq!(config.alerts.on_critical, "log");
    assert_eq!(config.alerts.on_depleted, "block");
}

#[test]
fn test_default_attribution() {
    let config = TokenomicsConfig::default();
    assert!(!config.attribution.enabled);
    assert!(config.attribution.tags.is_empty());
}

#[test]
fn test_default_batch() {
    let config = TokenomicsConfig::default();
    assert!(!config.batch.enabled);
    assert!(!config.batch.auto_promote);
    assert_eq!(config.batch.flush_interval_seconds, 30);
    assert_eq!(config.batch.max_batch_size, 100);
}

#[test]
fn test_default_legacy_fields() {
    let config = TokenomicsConfig::default();
    assert_eq!(config.default_budget, 100_000);
    assert_eq!(config.min_threshold, 10_000);
    assert!(config.ml_enabled);
    assert!(!config.federated_learning);
    assert!(!config.reinforcement_learning);
    assert_eq!(config.retrain_interval, 3600);
    assert!(config.redis_url.is_none());
    assert!(config.postgres_url.is_none());
}

// ── TOML: empty string triggers all serde defaults ───────────────────

#[test]
fn test_deserialize_empty_toml() {
    let config: TokenomicsConfig = toml::from_str("").unwrap();
    assert!(config.enabled);
    assert_eq!(config.strategy, "balanced");
    assert_eq!(config.default_budget, 100_000);
    assert_eq!(config.min_threshold, 10_000);
    assert_eq!(config.retrain_interval, 3600);
    assert_eq!(config.budget.max_tokens_per_call, 4_000);
    assert_eq!(config.budget.max_tokens_per_day, 100_000);
    assert_eq!(config.budget.max_tokens_per_month, 3_000_000);
    assert_eq!(config.budget.alert_threshold, 0.80);
    assert_eq!(config.budget.thinking_budget_default, 4096);
    assert_eq!(config.budget.thinking_budget_tiers.len(), 4);
    assert_eq!(config.prompt_caching.min_prefix_tokens, 1024);
    assert_eq!(config.cache.strategy, "exact");
    assert_eq!(config.cache.ttl_seconds, 300);
    assert_eq!(config.cache.max_entries, 1000);
    assert_eq!(config.cache.semantic_threshold, 0.95);
    assert_eq!(config.forecasting.method, "holt");
    assert_eq!(config.forecasting.horizon_hours, 24);
    assert_eq!(config.alerts.on_warning, "log");
    assert_eq!(config.alerts.on_critical, "log");
    assert_eq!(config.alerts.on_depleted, "block");
    assert_eq!(config.batch.flush_interval_seconds, 30);
    assert_eq!(config.batch.max_batch_size, 100);
    assert!(!config.federated_learning);
    assert!(!config.reinforcement_learning);
    assert!(config.redis_url.is_none());
    assert!(config.postgres_url.is_none());
}

// ── TOML: minimal ────────────────────────────────────────────────────

#[test]
fn test_parse_minimal_toml_conservative() {
    let toml_str = r#"enabled = true
strategy = "conservative""#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert!(config.enabled);
    assert_eq!(config.strategy, "conservative");
    assert_eq!(config.budget.max_tokens_per_day, 100_000);
}

#[test]
fn test_parse_disabled() {
    let config: TokenomicsConfig = toml::from_str("enabled = false").unwrap();
    assert!(!config.enabled);
}

#[test]
fn test_parse_strategy_aggressive() {
    let config: TokenomicsConfig = toml::from_str(r#"strategy = "aggressive""#).unwrap();
    assert_eq!(config.strategy, "aggressive");
}

// ── TOML: full config ─────────────────────────────────────────────────

#[test]
fn test_parse_full_toml() {
    let toml_str = r#"
        enabled = true
        strategy = "aggressive"

        [budget]
        max_tokens_per_call = 8000
        max_tokens_per_day = 200000
        max_tokens_per_month = 5000000
        alert_threshold = 0.75
        thinking_budget_default = 8192

        [[budget.thinking_budget_tiers]]
        name = "minimal"
        tokens = 512

        [[budget.thinking_budget_tiers]]
        name = "deep"
        tokens = 32768

        [pricing.anthropic]
        input_cost_per_1k = 0.015
        output_cost_per_1k = 0.075

        [prompt_caching]
        enabled = true
        auto_breakpoints = true
        min_prefix_tokens = 2048
        reorder_for_cache = true

        [cache]
        enabled = true
        strategy = "hybrid"
        ttl_seconds = 600
        max_entries = 5000
        semantic_threshold = 0.92

        [optimization]
        prompt_compression = true
        model_fallback = true
        fallback_model = "claude-haiku-3.5"
        cascade_routing = true

        [forecasting]
        enabled = true
        method = "prophet"
        horizon_hours = 48
        anomaly_detection = true

        [alerts]
        on_warning = "notify"
        on_critical = "pause"
        on_depleted = "block"

        [attribution]
        enabled = true
        tags = ["team", "project"]

        [batch]
        enabled = true
        auto_promote = true
        flush_interval_seconds = 60
        max_batch_size = 200
    "#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.strategy, "aggressive");
    assert_eq!(config.budget.max_tokens_per_call, 8000);
    assert_eq!(config.budget.max_tokens_per_day, 200000);
    assert_eq!(config.budget.max_tokens_per_month, 5000000);
    assert_eq!(config.budget.alert_threshold, 0.75);
    assert_eq!(config.budget.thinking_budget_default, 8192);
    assert_eq!(config.budget.thinking_budget_tiers.len(), 2);
    assert_eq!(config.pricing.len(), 1);
    assert!(config.pricing.contains_key("anthropic"));
    assert!(config.prompt_caching.enabled);
    assert!(config.prompt_caching.auto_breakpoints);
    assert_eq!(config.prompt_caching.min_prefix_tokens, 2048);
    assert!(config.prompt_caching.reorder_for_cache);
    assert!(config.cache.enabled);
    assert_eq!(config.cache.strategy, "hybrid");
    assert_eq!(config.cache.ttl_seconds, 600);
    assert_eq!(config.cache.max_entries, 5000);
    assert_eq!(config.cache.semantic_threshold, 0.92);
    assert!(config.optimization.prompt_compression);
    assert!(config.optimization.model_fallback);
    assert_eq!(
        config.optimization.fallback_model.as_deref(),
        Some("claude-haiku-3.5")
    );
    assert!(config.optimization.cascade_routing);
    assert!(config.forecasting.enabled);
    assert_eq!(config.forecasting.method, "prophet");
    assert_eq!(config.forecasting.horizon_hours, 48);
    assert!(config.forecasting.anomaly_detection);
    assert_eq!(config.alerts.on_warning, "notify");
    assert_eq!(config.alerts.on_critical, "pause");
    assert_eq!(config.alerts.on_depleted, "block");
    assert!(config.attribution.enabled);
    assert_eq!(config.attribution.tags, vec!["team", "project"]);
    assert!(config.batch.enabled);
    assert!(config.batch.auto_promote);
    assert_eq!(config.batch.flush_interval_seconds, 60);
    assert_eq!(config.batch.max_batch_size, 200);
}

// ── TOML: legacy compatibility ────────────────────────────────────────

#[test]
fn test_legacy_compat() {
    let toml_str = r#"
        default_budget = 50000
        min_threshold = 5000
        ml_enabled = true
        retrain_interval = 7200
    "#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.default_budget, 50000);
    assert_eq!(config.min_threshold, 5000);
    assert!(config.ml_enabled);
    assert_eq!(config.retrain_interval, 7200);
    // top-level defaults still apply
    assert!(config.enabled);
    assert_eq!(config.strategy, "balanced");
}

#[test]
fn test_legacy_database_urls() {
    let toml_str = r#"
        redis_url = "redis://localhost:6379"
        postgres_url = "postgres://localhost/db"
    "#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.redis_url.as_deref(), Some("redis://localhost:6379"));
    assert_eq!(
        config.postgres_url.as_deref(),
        Some("postgres://localhost/db")
    );
}

#[test]
fn test_legacy_fl_and_rl_flags() {
    let toml_str = r#"
        federated_learning = true
        reinforcement_learning = true
    "#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert!(config.federated_learning);
    assert!(config.reinforcement_learning);
}

// ── TOML: partial sub-struct — remaining fields use serde defaults ────

#[test]
fn test_partial_budget_uses_defaults() {
    let toml_str = r#"[budget]
max_tokens_per_call = 9999"#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.budget.max_tokens_per_call, 9999);
    assert_eq!(config.budget.max_tokens_per_day, 100_000);
    assert_eq!(config.budget.max_tokens_per_month, 3_000_000);
    assert_eq!(config.budget.alert_threshold, 0.80);
    assert_eq!(config.budget.thinking_budget_default, 4096);
    assert_eq!(config.budget.thinking_budget_tiers.len(), 4);
}

#[test]
fn test_partial_cache_uses_defaults() {
    let toml_str = r#"[cache]
enabled = true"#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert!(config.cache.enabled);
    assert_eq!(config.cache.strategy, "exact");
    assert_eq!(config.cache.ttl_seconds, 300);
    assert_eq!(config.cache.max_entries, 1000);
}

#[test]
fn test_partial_forecasting_uses_defaults() {
    let toml_str = r#"[forecasting]
enabled = true"#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert!(config.forecasting.enabled);
    assert_eq!(config.forecasting.method, "holt");
    assert_eq!(config.forecasting.horizon_hours, 24);
}

#[test]
fn test_partial_alerts_uses_defaults() {
    let toml_str = r#"[alerts]
on_warning = "notify""#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.alerts.on_warning, "notify");
    assert_eq!(config.alerts.on_critical, "log");
    assert_eq!(config.alerts.on_depleted, "block");
}

#[test]
fn test_partial_batch_uses_defaults() {
    let toml_str = r#"[batch]
enabled = true"#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert!(config.batch.enabled);
    assert_eq!(config.batch.flush_interval_seconds, 30);
    assert_eq!(config.batch.max_batch_size, 100);
}

// ── Sub-config Default impls ──────────────────────────────────────────

#[test]
fn test_budget_config_default() {
    let bc = BudgetConfig::default();
    assert_eq!(bc.max_tokens_per_call, 4_000);
    assert_eq!(bc.max_tokens_per_day, 100_000);
    assert_eq!(bc.max_tokens_per_month, 3_000_000);
    assert_eq!(bc.alert_threshold, 0.80);
    assert_eq!(bc.thinking_budget_tiers.len(), 4);
}

#[test]
fn test_prompt_caching_config_default() {
    let pc = PromptCachingConfig::default();
    assert!(!pc.enabled);
    assert_eq!(pc.min_prefix_tokens, 1024);
    assert!(!pc.auto_breakpoints);
    assert!(!pc.reorder_for_cache);
}

#[test]
fn test_response_cache_config_default() {
    let rc = ResponseCacheConfig::default();
    assert!(!rc.enabled);
    assert_eq!(rc.strategy, "exact");
    assert_eq!(rc.ttl_seconds, 300);
    assert_eq!(rc.max_entries, 1000);
    assert_eq!(rc.semantic_threshold, 0.95);
}

#[test]
fn test_optimization_config_default() {
    let oc = OptimizationConfig::default();
    assert!(!oc.prompt_compression);
    assert!(!oc.model_fallback);
    assert!(oc.fallback_model.is_none());
    assert!(!oc.cascade_routing);
    assert!(!oc.batch_eligible);
}

#[test]
fn test_forecasting_config_default() {
    let fc = ForecastingConfig::default();
    assert!(!fc.enabled);
    assert_eq!(fc.method, "holt");
    assert_eq!(fc.horizon_hours, 24);
    assert!(!fc.anomaly_detection);
}

#[test]
fn test_alerts_config_default() {
    let ac = AlertsConfig::default();
    assert_eq!(ac.on_warning, "log");
    assert_eq!(ac.on_critical, "log");
    assert_eq!(ac.on_depleted, "block");
}

#[test]
fn test_attribution_config_default() {
    let ac = AttributionConfig::default();
    assert!(!ac.enabled);
    assert!(ac.tags.is_empty());
}

#[test]
fn test_batch_config_default() {
    let bc = BatchConfig::default();
    assert!(!bc.enabled);
    assert!(!bc.auto_promote);
    assert_eq!(bc.flush_interval_seconds, 30);
    assert_eq!(bc.max_batch_size, 100);
}

#[test]
fn test_pricing_config_default() {
    let pc = PricingConfig::default();
    assert_eq!(pc.input_cost_per_1k, 0.0);
    assert_eq!(pc.output_cost_per_1k, 0.0);
    assert_eq!(pc.cached_input_cost_per_1k, 0.0);
    assert_eq!(pc.cache_write_cost_per_1k, 0.0);
    assert_eq!(pc.thinking_cost_per_1k, 0.0);
    assert_eq!(pc.batch_discount, 0.0);
    assert_eq!(pc.image_cost_per_token, 0.0);
    assert_eq!(pc.audio_cost_per_minute, 0.0);
}

// ── Clone / Debug derives ────────────────────────────────────────────

#[test]
fn test_config_clone() {
    let config = TokenomicsConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.strategy, config.strategy);
    assert_eq!(cloned.default_budget, config.default_budget);
}

#[test]
fn test_budget_config_clone() {
    let bc = BudgetConfig::default();
    let cloned = bc.clone();
    assert_eq!(cloned.max_tokens_per_call, bc.max_tokens_per_call);
}

#[test]
fn test_thinking_tier_fields() {
    let tier = ThinkingTier {
        name: "custom".to_string(),
        tokens: 8192,
    };
    assert_eq!(tier.name, "custom");
    assert_eq!(tier.tokens, 8192);
}

// ── Pricing map ───────────────────────────────────────────────────────

#[test]
fn test_pricing_map_empty_by_default() {
    let config = TokenomicsConfig::default();
    assert!(config.pricing.is_empty());
}

#[test]
fn test_pricing_map_multiple_providers() {
    let toml_str = r#"
        [pricing.anthropic]
        input_cost_per_1k = 0.015

        [pricing.openai]
        input_cost_per_1k = 0.010
    "#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.pricing.len(), 2);
    assert!(config.pricing.contains_key("anthropic"));
    assert!(config.pricing.contains_key("openai"));
}
