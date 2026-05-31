//! Tests for tokenomics lib — root constants and re-exported types

use hudhudscript_tokenomics::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

#[test]
fn test_version_not_empty() {
    assert!(!VERSION.is_empty());
}

#[test]
fn test_default_budget_constant() {
    assert_eq!(DEFAULT_BUDGET, 100_000);
}

#[test]
fn test_min_budget_threshold_constant() {
    assert_eq!(MIN_BUDGET_THRESHOLD, 10_000);
}

#[test]
fn test_max_prediction_horizon_constant() {
    assert_eq!(MAX_PREDICTION_HORIZON, 86_400);
}

#[test]
fn test_constants_relationship() {
    assert!(DEFAULT_BUDGET > MIN_BUDGET_THRESHOLD);
    assert!(MAX_PREDICTION_HORIZON > 0);
}

#[test]
fn test_default_budget_above_threshold() {
    assert!(DEFAULT_BUDGET >= MIN_BUDGET_THRESHOLD * 2);
}

// ---------------------------------------------------------------------------
// Re-exported types
// ---------------------------------------------------------------------------

#[test]
fn test_budget_reexport() {
    let budget = Budget::new("user1".into(), DEFAULT_BUDGET);
    assert_eq!(budget.total, DEFAULT_BUDGET);
    assert_eq!(budget.remaining, DEFAULT_BUDGET);
    assert_eq!(budget.used, 0);
}

#[test]
fn test_budget_consume_via_reexport() {
    let mut budget = Budget::new("user1".into(), 1000);
    budget.consume(500).unwrap();
    assert_eq!(budget.remaining, 500);
}

#[test]
fn test_optimization_strategy_default() {
    let strategy = OptimizationStrategy::default();
    assert!(matches!(strategy, OptimizationStrategy::Balanced));
}

#[test]
fn test_optimization_strategy_conservative() {
    let _c = OptimizationStrategy::Conservative;
}

#[test]
fn test_optimization_strategy_balanced() {
    let _b = OptimizationStrategy::Balanced;
}

#[test]
fn test_optimization_strategy_aggressive() {
    let _a = OptimizationStrategy::Aggressive;
}

#[test]
fn test_optimization_strategy_custom() {
    let _custom = OptimizationStrategy::Custom {
        performance_weight: 1.0,
        cost_weight: 1.0,
    };
}

#[test]
fn test_token_usage_reexport() {
    let usage = TokenUsageRecord::new("u".into(), "op".into(), 100);
    assert_eq!(usage.tokens_used, 100);
}

#[test]
fn test_prediction_reexport() {
    let pred = Prediction {
        predicted_usage: 1000,
        confidence: 0.9,
        horizon_seconds: 3600,
        timestamp: chrono::Utc::now(),
        model_version: "v1".into(),
    };
    assert_eq!(pred.predicted_usage, 1000);
}
