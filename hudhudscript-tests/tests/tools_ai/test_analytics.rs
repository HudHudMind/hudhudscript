//! Tests extracted from hudhudscript-tools-ai/src/analytics.rs

use hudhudscript_tools_ai::{Analytics, BudgetConfig, CostTracker, Provider};

fn tracker_with_data() -> CostTracker {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    tracker.record_usage("gpt-4o", 1000, 500).unwrap();
    tracker.record_usage("gpt-4o", 2000, 1000).unwrap();
    tracker.record_usage("claude-3-haiku", 500, 200).unwrap();
    tracker.record_usage("deepseek-chat", 3000, 1500).unwrap();
    tracker
}

#[test]
fn test_summary() {
    let analytics = Analytics::new(tracker_with_data());
    let summary = analytics.summary();

    assert_eq!(summary.total_requests, 4);
    assert_eq!(summary.total_input_tokens, 1000 + 2000 + 500 + 3000);
    assert_eq!(summary.total_output_tokens, 500 + 1000 + 200 + 1500);
    assert!(summary.total_cost_usd > 0.0);
    assert!(summary.avg_cost_per_request > 0.0);
}

#[test]
fn test_by_provider() {
    let analytics = Analytics::new(tracker_with_data());
    let providers = analytics.by_provider();

    // Should have 3 providers: OpenAI, Anthropic, DeepSeek
    assert_eq!(providers.len(), 3);

    let openai = providers
        .iter()
        .find(|p| p.provider == Provider::OpenAI)
        .unwrap();
    assert_eq!(openai.requests, 2);
}

#[test]
fn test_by_model() {
    let analytics = Analytics::new(tracker_with_data());
    let models = analytics.by_model();

    assert_eq!(models.len(), 3); // gpt-4o, claude-3-haiku, deepseek-chat

    let gpt4o = models.iter().find(|m| m.model == "gpt-4o").unwrap();
    assert_eq!(gpt4o.requests, 2);
    assert_eq!(gpt4o.input_tokens, 3000);
}

#[test]
fn test_report_json() {
    let analytics = Analytics::new(tracker_with_data());
    let json = analytics.report_json().unwrap();
    assert!(json.contains("total_requests"));
    assert!(json.contains("by_provider"));
    assert!(json.contains("by_model"));
}

#[test]
fn test_empty_tracker() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    let analytics = Analytics::new(tracker);
    let summary = analytics.summary();

    assert_eq!(summary.total_requests, 0);
    assert_eq!(summary.total_tokens, 0);
    assert!((summary.total_cost_usd - 0.0).abs() < 1e-9);
    assert!((summary.avg_cost_per_request - 0.0).abs() < 1e-9);
}

#[test]
fn test_top_models_by_cost() {
    let analytics = Analytics::new(tracker_with_data());
    let top = analytics.top_models_by_cost(2);
    assert_eq!(top.len(), 2);
    // Most expensive should come first (already sorted)
    assert!(top[0].cost_usd >= top[1].cost_usd);
}

#[test]
fn test_cost_share_sums_to_one() {
    let analytics = Analytics::new(tracker_with_data());
    let providers = analytics.by_provider();
    let total_share: f64 = providers.iter().map(|p| p.cost_share).sum();
    assert!((total_share - 1.0).abs() < 1e-9);
}

#[test]
fn test_history_in_range_all() {
    let analytics = Analytics::new(tracker_with_data());
    let all = analytics.history_in_range(0, u64::MAX);
    assert_eq!(all.len(), 4);
}

#[test]
fn test_history_in_range_none() {
    let analytics = Analytics::new(tracker_with_data());
    // Timestamps far in the past — no results
    let none = analytics.history_in_range(0, 1);
    assert_eq!(none.len(), 0);
}

#[test]
fn test_top_models_by_cost_more_than_available() {
    let analytics = Analytics::new(tracker_with_data());
    let top = analytics.top_models_by_cost(100);
    assert_eq!(top.len(), 3); // only 3 distinct models
}

#[test]
fn test_empty_analytics_by_provider() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    let analytics = Analytics::new(tracker);
    let providers = analytics.by_provider();
    assert!(providers.is_empty());
}

#[test]
fn test_empty_analytics_by_model() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    let analytics = Analytics::new(tracker);
    let models = analytics.by_model();
    assert!(models.is_empty());
}

#[test]
fn test_report_struct() {
    let analytics = Analytics::new(tracker_with_data());
    let report = analytics.report();
    assert!(report.generated_at > 0);
    assert_eq!(report.summary.total_requests, 4);
    assert!(!report.by_provider.is_empty());
    assert!(!report.by_model.is_empty());
}
