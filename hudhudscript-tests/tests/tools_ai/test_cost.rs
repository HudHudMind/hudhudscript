//! Tests extracted from hudhudscript-tools-ai/src/cost.rs
//! Skipped (already in tools_ai_test_lib.rs): test_record_usage_accumulates,
//! test_budget_exceeded, test_remaining_budget, test_reset,
//! test_cost_by_provider, test_cost_by_model

use hudhudscript_tools_ai::cost::default_pricing;
use hudhudscript_tools_ai::{BudgetConfig, CostError, CostTracker, ModelPricing, Provider};

#[test]
fn test_default_pricing_populated() {
    let pricing = default_pricing();
    assert!(pricing.contains_key("gpt-4o"));
    assert!(pricing.contains_key("claude-3-opus"));
    assert!(pricing.contains_key("ollama-local"));
    assert!(pricing.contains_key("deepseek-chat"));
}

#[test]
fn test_calculate_cost() {
    let tracker = CostTracker::with_defaults();
    // gpt-4o: input 0.005/1k, output 0.015/1k
    let cost = tracker.calculate_cost("gpt-4o", 1000, 500).unwrap();
    // 1000 * 0.005/1000 + 500 * 0.015/1000 = 0.005 + 0.0075 = 0.0125
    assert!((cost - 0.0125).abs() < 1e-9);
}

#[test]
fn test_ollama_is_free() {
    let tracker = CostTracker::with_defaults();
    let cost = tracker.calculate_cost("ollama-local", 10000, 5000).unwrap();
    assert!((cost - 0.0).abs() < 1e-9);
}

#[test]
fn test_unknown_model_error() {
    let tracker = CostTracker::with_defaults();
    assert!(tracker.calculate_cost("no-such-model", 1, 1).is_err());
}

#[test]
fn test_record_usage_from_text() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    let usage = tracker
        .record_usage_from_text("gpt-4o", "Hello world", "Response text here")
        .unwrap();
    assert!(usage.input_tokens > 0);
    assert!(usage.output_tokens > 0);
}

#[test]
fn test_provider_display() {
    assert_eq!(format!("{}", Provider::OpenAI), "OpenAI");
    assert_eq!(format!("{}", Provider::Anthropic), "Anthropic");
    assert_eq!(format!("{}", Provider::Ollama), "Ollama");
    assert_eq!(format!("{}", Provider::DeepSeek), "DeepSeek");
}

#[test]
fn test_count_tokens_static() {
    assert_eq!(CostTracker::count_tokens(""), 0);
    assert_eq!(CostTracker::count_tokens("hello"), 1);
    assert_eq!(CostTracker::count_tokens(&"a".repeat(40)), 10);
}

#[test]
fn test_total_tokens() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    tracker.record_usage("gpt-4o", 1000, 500).unwrap();
    tracker.record_usage("gpt-4o", 2000, 1000).unwrap();
    assert_eq!(tracker.total_tokens(), 1000 + 500 + 2000 + 1000);
}

#[test]
fn test_get_pricing() {
    let tracker = CostTracker::with_defaults();
    let pricing = tracker.get_pricing("gpt-4o").unwrap();
    assert_eq!(pricing.provider, Provider::OpenAI);
    assert!(pricing.input_cost_per_1k > 0.0);

    assert!(tracker.get_pricing("nonexistent").is_none());
}

#[test]
fn test_set_budget_resets_alerts() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 1.0,
        alert_thresholds: vec![0.5],
    });
    // Record enough to trigger the 50% alert
    tracker.record_usage("gpt-4o", 10000, 5000).unwrap();

    // Now set a higher budget — alerts should be reset
    tracker.set_budget(BudgetConfig {
        hard_limit_usd: 1000.0,
        alert_thresholds: vec![0.5, 0.8],
    });
    // Can now record without hitting budget
    assert!(tracker.record_usage("gpt-4o", 1000, 500).is_ok());
}

#[test]
fn test_budget_config_default() {
    let config = BudgetConfig::default();
    assert_eq!(config.hard_limit_usd, 10.0);
    assert_eq!(config.alert_thresholds, vec![0.5, 0.8, 0.95]);
}

#[test]
fn test_cost_error_display() {
    let e = CostError::UnknownProvider("test".to_string());
    assert!(format!("{}", e).contains("Unknown provider: test"));

    let e = CostError::UnknownModel("test".to_string());
    assert!(format!("{}", e).contains("Unknown model: test"));

    let e = CostError::BudgetExceeded {
        spent: 1.5,
        limit: 1.0,
    };
    assert!(format!("{}", e).contains("Budget exceeded"));
}

#[test]
fn test_set_model_pricing() {
    let tracker = CostTracker::with_defaults();
    tracker.set_model_pricing(ModelPricing {
        provider: Provider::OpenAI,
        model: "custom-model".into(),
        input_cost_per_1k: 0.1,
        output_cost_per_1k: 0.2,
    });
    let cost = tracker.calculate_cost("custom-model", 1000, 1000).unwrap();
    assert!((cost - 0.3).abs() < 1e-9);
}
