//! Tests for tokenomics::types — Budget, TokenUsageRecord, OptimizationStrategy, Prediction,
//! ModelMetadata, FederatedUpdate, Reward, TimeSeriesPoint, Metrics

use chrono::Utc;
use hudhudscript_tokenomics::types::*;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Budget
// ---------------------------------------------------------------------------

#[test]
fn test_budget_new() {
    let budget = Budget::new("user1".into(), 1000);
    assert_eq!(budget.total, 1000);
    assert_eq!(budget.used, 0);
    assert_eq!(budget.remaining, 1000);
    assert_eq!(budget.user_id, "user1");
    assert!(!budget.id.is_nil());
    assert!(budget.expires_at.is_none());
}

#[test]
fn test_budget_consume_success() {
    let mut budget = Budget::new("u".into(), 1000);
    assert!(budget.consume(500).is_ok());
    assert_eq!(budget.used, 500);
    assert_eq!(budget.remaining, 500);
}

#[test]
fn test_budget_consume_exact_remaining() {
    let mut budget = Budget::new("u".into(), 100);
    assert!(budget.consume(100).is_ok());
    assert_eq!(budget.used, 100);
    assert_eq!(budget.remaining, 0);
}

#[test]
fn test_budget_consume_insufficient() {
    let mut budget = Budget::new("u".into(), 100);
    let err = budget.consume(200);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("Insufficient budget"));
}

#[test]
fn test_budget_consume_multiple() {
    let mut budget = Budget::new("u".into(), 1000);
    budget.consume(300).unwrap();
    budget.consume(200).unwrap();
    assert_eq!(budget.used, 500);
    assert_eq!(budget.remaining, 500);
}

#[test]
fn test_budget_refill() {
    let mut budget = Budget::new("u".into(), 1000);
    budget.consume(500).unwrap();
    budget.refill(300);
    assert_eq!(budget.total, 1300);
    assert_eq!(budget.remaining, 800);
    assert_eq!(budget.used, 500);
}

#[test]
fn test_budget_refill_zero() {
    let mut budget = Budget::new("u".into(), 100);
    budget.refill(0);
    assert_eq!(budget.total, 100);
    assert_eq!(budget.remaining, 100);
}

#[test]
fn test_budget_usage_percentage() {
    let mut budget = Budget::new("u".into(), 1000);
    budget.consume(250).unwrap();
    assert_eq!(budget.usage_percentage(), 25.0);
}

#[test]
fn test_budget_usage_percentage_zero_total() {
    let budget = Budget::new("u".into(), 0);
    assert_eq!(budget.usage_percentage(), 0.0);
}

#[test]
fn test_budget_usage_percentage_full() {
    let mut budget = Budget::new("u".into(), 200);
    budget.consume(200).unwrap();
    assert_eq!(budget.usage_percentage(), 100.0);
}

#[test]
fn test_budget_updated_at_changes() {
    let mut budget = Budget::new("u".into(), 1000);
    let before = budget.updated_at;
    std::thread::sleep(std::time::Duration::from_millis(2));
    budget.consume(1).unwrap();
    assert!(budget.updated_at >= before);
}

// ---------------------------------------------------------------------------
// TokenUsageRecord
// ---------------------------------------------------------------------------

#[test]
fn test_token_usage_new() {
    let usage = TokenUsageRecord::new("user1".into(), "compile".into(), 500);
    assert_eq!(usage.user_id, "user1");
    assert_eq!(usage.operation, "compile");
    assert_eq!(usage.tokens_used, 500);
    assert!(!usage.id.is_nil());
    assert!(usage.session_id.is_none());
}

#[test]
fn test_token_usage_metadata_default() {
    let usage = TokenUsageRecord::new("u".into(), "op".into(), 10);
    assert_eq!(usage.metadata, serde_json::json!({}));
}

// ---------------------------------------------------------------------------
// OptimizationStrategy
// ---------------------------------------------------------------------------

#[test]
fn test_strategy_default_is_balanced() {
    let strategy = OptimizationStrategy::default();
    assert!(matches!(strategy, OptimizationStrategy::Balanced));
}

#[test]
fn test_strategy_conservative() {
    let _s = OptimizationStrategy::Conservative;
}

#[test]
fn test_strategy_aggressive() {
    let _s = OptimizationStrategy::Aggressive;
}

#[test]
fn test_strategy_custom() {
    let s = OptimizationStrategy::Custom {
        performance_weight: 0.8,
        cost_weight: 0.2,
    };
    match s {
        OptimizationStrategy::Custom {
            performance_weight,
            cost_weight,
        } => {
            assert_eq!(performance_weight, 0.8);
            assert_eq!(cost_weight, 0.2);
        }
        _ => panic!("Expected Custom variant"),
    }
}

// ---------------------------------------------------------------------------
// Prediction
// ---------------------------------------------------------------------------

#[test]
fn test_prediction_fields() {
    let pred = Prediction {
        predicted_usage: 5000,
        confidence: 0.95,
        horizon_seconds: 3600,
        timestamp: Utc::now(),
        model_version: "v1.0".into(),
    };
    assert_eq!(pred.predicted_usage, 5000);
    assert_eq!(pred.confidence, 0.95);
    assert_eq!(pred.horizon_seconds, 3600);
    assert_eq!(pred.model_version, "v1.0");
}

// ---------------------------------------------------------------------------
// ModelMetadata
// ---------------------------------------------------------------------------

#[test]
fn test_model_metadata_fields() {
    let meta = ModelMetadata {
        name: "forecaster".into(),
        version: "2.0".into(),
        trained_at: Utc::now(),
        accuracy: 0.92,
        samples_count: 10000,
    };
    assert_eq!(meta.name, "forecaster");
    assert_eq!(meta.version, "2.0");
    assert_eq!(meta.accuracy, 0.92);
    assert_eq!(meta.samples_count, 10000);
}

// ---------------------------------------------------------------------------
// FederatedUpdate
// ---------------------------------------------------------------------------

#[test]
fn test_federated_update_fields() {
    let update = FederatedUpdate {
        id: Uuid::new_v4(),
        model_version: "1.0".into(),
        gradients: vec![0.1, -0.2, 0.3],
        sample_count: 100,
        timestamp: Utc::now(),
    };
    assert_eq!(update.gradients.len(), 3);
    assert_eq!(update.sample_count, 100);
    assert_eq!(update.model_version, "1.0");
}

// ---------------------------------------------------------------------------
// Reward
// ---------------------------------------------------------------------------

#[test]
fn test_reward_fields() {
    let id = Uuid::new_v4();
    let reward = Reward {
        action_id: id,
        reward_value: 5.0,
        feedback: Some("good".into()),
        timestamp: Utc::now(),
    };
    assert_eq!(reward.action_id, id);
    assert_eq!(reward.reward_value, 5.0);
    assert_eq!(reward.feedback, Some("good".into()));
}

#[test]
fn test_reward_no_feedback() {
    let reward = Reward {
        action_id: Uuid::new_v4(),
        reward_value: -1.0,
        feedback: None,
        timestamp: Utc::now(),
    };
    assert!(reward.feedback.is_none());
    assert_eq!(reward.reward_value, -1.0);
}

// ---------------------------------------------------------------------------
// TimeSeriesPoint
// ---------------------------------------------------------------------------

#[test]
fn test_time_series_point_fields() {
    let pt = TimeSeriesPoint {
        timestamp: Utc::now(),
        value: 42.5,
        metadata: serde_json::json!({"source": "test"}),
    };
    assert_eq!(pt.value, 42.5);
    assert_eq!(pt.metadata["source"], "test");
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

#[test]
fn test_metrics_fields() {
    let m = Metrics {
        total_users: 100,
        total_usage: 500_000,
        average_usage: 5000.0,
        peak_usage: 20_000,
        prediction_accuracy: 0.88,
        timestamp: Utc::now(),
    };
    assert_eq!(m.total_users, 100);
    assert_eq!(m.total_usage, 500_000);
    assert_eq!(m.average_usage, 5000.0);
    assert_eq!(m.peak_usage, 20_000);
    assert_eq!(m.prediction_accuracy, 0.88);
}
