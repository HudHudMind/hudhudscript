//! Public API tests for `hudhudscript_tokenomics::optimizer`.
//! Tests cover TokenomicsEngine and UsageStatistics via only public methods.

use hudhudscript_tokenomics::optimizer::{TokenomicsEngine, UsageStatistics};
use hudhudscript_tokenomics::types::OptimizationStrategy;

// ── Engine construction ──────────────────────────────────────────────

#[tokio::test]
async fn test_engine_new_balanced() {
    let _e = TokenomicsEngine::new(OptimizationStrategy::Balanced);
}

#[tokio::test]
async fn test_engine_new_conservative() {
    let _e = TokenomicsEngine::new(OptimizationStrategy::Conservative);
}

#[tokio::test]
async fn test_engine_new_aggressive() {
    let _e = TokenomicsEngine::new(OptimizationStrategy::Aggressive);
}

#[tokio::test]
async fn test_engine_new_custom() {
    let _e = TokenomicsEngine::new(OptimizationStrategy::Custom {
        performance_weight: 0.7,
        cost_weight: 0.3,
    });
}

// ── Budget: create ───────────────────────────────────────────────────

#[tokio::test]
async fn test_get_or_create_budget_new_user() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let budget = engine.get_or_create_budget("user1", 10000).await.unwrap();
    assert_eq!(budget.total, 10000);
    assert_eq!(budget.remaining, 10000);
    assert_eq!(budget.used, 0);
    assert_eq!(budget.user_id, "user1");
}

#[tokio::test]
async fn test_get_or_create_budget_idempotent() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let b1 = engine.get_or_create_budget("user1", 5000).await.unwrap();
    let b2 = engine.get_or_create_budget("user1", 9999).await.unwrap();
    // second call returns existing budget, not a new one
    assert_eq!(b1.id, b2.id);
    assert_eq!(b2.total, 5000);
    assert_eq!(b2.remaining, 5000);
}

#[tokio::test]
async fn test_get_or_create_budget_multiple_users() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let b1 = engine.get_or_create_budget("alice", 1000).await.unwrap();
    let b2 = engine.get_or_create_budget("bob", 2000).await.unwrap();
    assert_ne!(b1.id, b2.id);
    assert_eq!(b1.total, 1000);
    assert_eq!(b2.total, 2000);
}

// ── Token consumption ────────────────────────────────────────────────

#[tokio::test]
async fn test_consume_tokens_basic() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 10000).await.unwrap();
    engine
        .consume_tokens("user1", 1000, "test_op")
        .await
        .unwrap();
    let budget = engine.get_or_create_budget("user1", 10000).await.unwrap();
    assert_eq!(budget.remaining, 9000);
    assert_eq!(budget.used, 1000);
}

#[tokio::test]
async fn test_consume_tokens_exact_remaining() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 500).await.unwrap();
    engine.consume_tokens("user1", 500, "drain").await.unwrap();
    let budget = engine.get_or_create_budget("user1", 500).await.unwrap();
    assert_eq!(budget.remaining, 0);
    assert_eq!(budget.used, 500);
    assert_eq!(budget.total, 500);
}

#[tokio::test]
async fn test_consume_tokens_multiple_operations() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 1000).await.unwrap();
    engine.consume_tokens("user1", 100, "op1").await.unwrap();
    engine.consume_tokens("user1", 250, "op2").await.unwrap();
    engine.consume_tokens("user1", 50, "op3").await.unwrap();
    let budget = engine.get_or_create_budget("user1", 1000).await.unwrap();
    assert_eq!(budget.used, 400);
    assert_eq!(budget.remaining, 600);
}

#[tokio::test]
async fn test_consume_tokens_insufficient_budget() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 100).await.unwrap();
    let result = engine.consume_tokens("user1", 200, "test").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_consume_tokens_nonexistent_user() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let result = engine.consume_tokens("nobody", 100, "test").await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Budget not found: nobody"));
}

#[tokio::test]
async fn test_consume_tokens_zero_amount() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 1000).await.unwrap();
    engine.consume_tokens("user1", 0, "noop").await.unwrap();
    let budget = engine.get_or_create_budget("user1", 1000).await.unwrap();
    assert_eq!(budget.remaining, 1000);
}

#[tokio::test]
async fn test_consume_tokens_one_over_budget_fails() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 999).await.unwrap();
    assert!(engine.consume_tokens("user1", 1000, "big").await.is_err());
}

// ── Multi-user isolation ─────────────────────────────────────────────

#[tokio::test]
async fn test_multi_user_budget_isolation() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("alice", 5000).await.unwrap();
    engine.get_or_create_budget("bob", 3000).await.unwrap();
    engine.consume_tokens("alice", 2000, "op").await.unwrap();
    let alice = engine.get_or_create_budget("alice", 5000).await.unwrap();
    let bob = engine.get_or_create_budget("bob", 3000).await.unwrap();
    assert_eq!(alice.remaining, 3000);
    assert_eq!(bob.remaining, 3000);
}

#[tokio::test]
async fn test_multi_user_statistics_isolation() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("alice", 10000).await.unwrap();
    engine.get_or_create_budget("bob", 10000).await.unwrap();
    engine.consume_tokens("alice", 100, "a").await.unwrap();
    engine.consume_tokens("alice", 200, "b").await.unwrap();
    engine.consume_tokens("bob", 500, "c").await.unwrap();
    let alice_stats = engine.get_statistics("alice").await.unwrap();
    let bob_stats = engine.get_statistics("bob").await.unwrap();
    assert_eq!(alice_stats.total_usage, 300);
    assert_eq!(alice_stats.operation_count, 2);
    assert_eq!(bob_stats.total_usage, 500);
    assert_eq!(bob_stats.operation_count, 1);
}

// ── Prediction: fallback path ────────────────────────────────────────

#[tokio::test]
async fn test_predict_usage_cold_start_fallback() {
    // < 10 data points → fallback with confidence 0.5
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 10000).await.unwrap();
    engine.consume_tokens("user1", 100, "op1").await.unwrap();
    engine.consume_tokens("user1", 200, "op2").await.unwrap();
    engine.consume_tokens("user1", 300, "op3").await.unwrap();
    let prediction = engine.predict_usage("user1", 3600).await.unwrap();
    assert_eq!(prediction.confidence, 0.5);
    assert_eq!(prediction.model_version, "fallback-v1");
    assert_eq!(prediction.horizon_seconds, 3600);
}

#[tokio::test]
async fn test_predict_usage_fallback_linear_extrapolation() {
    // avg = (100+200+300+400)/4 = 250; horizon 7200 => 250 * 7200/3600 = 500
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 100000).await.unwrap();
    engine.consume_tokens("user1", 100, "a").await.unwrap();
    engine.consume_tokens("user1", 200, "b").await.unwrap();
    engine.consume_tokens("user1", 300, "c").await.unwrap();
    engine.consume_tokens("user1", 400, "d").await.unwrap();
    let prediction = engine.predict_usage("user1", 7200).await.unwrap();
    assert_eq!(prediction.predicted_usage, 500);
    assert_eq!(prediction.confidence, 0.5);
}

#[tokio::test]
async fn test_predict_usage_no_history_default_avg() {
    // no history → fallback uses 1000.0 default; horizon 3600 → predicted 1000
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let prediction = engine.predict_usage("ghost_user", 3600).await.unwrap();
    assert_eq!(prediction.predicted_usage, 1000);
    assert_eq!(prediction.confidence, 0.5);
}

#[tokio::test]
async fn test_predict_usage_fallback_half_horizon() {
    // horizon 1800 → predicted = (1000.0 * 1800/3600) as u64 = 500
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let prediction = engine.predict_usage("ghost", 1800).await.unwrap();
    assert_eq!(prediction.predicted_usage, 500);
}

// ── Prediction: ML path (>= 10 data points) ──────────────────────────

#[tokio::test]
async fn test_predict_usage_ml_path() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 500000).await.unwrap();
    for i in 0..10 {
        engine
            .consume_tokens("user1", 100 + i * 10, &format!("op{}", i))
            .await
            .unwrap();
    }
    let prediction = engine.predict_usage("user1", 3600).await.unwrap();
    assert_eq!(prediction.confidence, 0.0); // untrained model
    assert_eq!(prediction.model_version, "0.1.0");
    assert_eq!(prediction.horizon_seconds, 3600);
}

#[tokio::test]
async fn test_predict_usage_ml_path_longer_horizon() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("u1", 1_000_000).await.unwrap();
    for i in 0..12 {
        engine
            .consume_tokens("u1", 50 + i * 5, &format!("op{}", i))
            .await
            .unwrap();
    }
    let pred = engine.predict_usage("u1", 7200).await.unwrap();
    assert_eq!(pred.model_version, "0.1.0");
    assert_eq!(pred.horizon_seconds, 7200);
}

// ── Optimization strategies ──────────────────────────────────────────

#[tokio::test]
async fn test_optimize_allocation_conservative() {
    // no history → fallback predicted=1000; conservative = 1000 * 0.8 = 800
    let engine = TokenomicsEngine::new(OptimizationStrategy::Conservative);
    let allocation = engine.optimize_allocation("user1").await.unwrap();
    assert_eq!(allocation, 800);
}

#[tokio::test]
async fn test_optimize_allocation_balanced() {
    // balanced = 100% of predicted = 1000
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let allocation = engine.optimize_allocation("user1").await.unwrap();
    assert_eq!(allocation, 1000);
}

#[tokio::test]
async fn test_optimize_allocation_aggressive() {
    // aggressive = 1000 * 1.2 = 1200
    let engine = TokenomicsEngine::new(OptimizationStrategy::Aggressive);
    let allocation = engine.optimize_allocation("user1").await.unwrap();
    assert_eq!(allocation, 1200);
}

#[tokio::test]
async fn test_optimize_allocation_custom_perf_heavy() {
    // perf=3, cost=1 → factor=0.75 → multiplier=0.8+0.3=1.1 → 1100
    let engine = TokenomicsEngine::new(OptimizationStrategy::Custom {
        performance_weight: 3.0,
        cost_weight: 1.0,
    });
    let allocation = engine.optimize_allocation("user1").await.unwrap();
    assert_eq!(allocation, 1100);
}

#[tokio::test]
async fn test_optimize_allocation_custom_cost_heavy() {
    // perf=1, cost=3 → factor=0.25 → multiplier=0.8+0.1=0.9 → 900
    let engine = TokenomicsEngine::new(OptimizationStrategy::Custom {
        performance_weight: 1.0,
        cost_weight: 3.0,
    });
    let allocation = engine.optimize_allocation("user1").await.unwrap();
    assert_eq!(allocation, 900);
}

#[tokio::test]
async fn test_optimize_allocation_custom_equal_weights() {
    // perf=1, cost=1 → factor=0.5 → multiplier=0.8+0.2=1.0 → 1000
    let engine = TokenomicsEngine::new(OptimizationStrategy::Custom {
        performance_weight: 1.0,
        cost_weight: 1.0,
    });
    let allocation = engine.optimize_allocation("user1").await.unwrap();
    assert_eq!(allocation, 1000);
}

// ── Statistics ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_statistics_with_usage() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 10000).await.unwrap();
    engine.consume_tokens("user1", 100, "op1").await.unwrap();
    engine.consume_tokens("user1", 300, "op2").await.unwrap();
    engine.consume_tokens("user1", 200, "op3").await.unwrap();
    let stats = engine.get_statistics("user1").await.unwrap();
    assert_eq!(stats.total_usage, 600);
    assert_eq!(stats.average_usage, 200.0);
    assert_eq!(stats.operation_count, 3);
    let budget = stats.current_budget.unwrap();
    assert_eq!(budget.remaining, 9400);
}

#[tokio::test]
async fn test_get_statistics_empty() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let stats = engine.get_statistics("nonexistent").await.unwrap();
    assert_eq!(stats.total_usage, 0);
    assert_eq!(stats.average_usage, 0.0);
    assert_eq!(stats.operation_count, 0);
    assert!(stats.current_budget.is_none());
}

#[tokio::test]
async fn test_get_statistics_with_budget_no_usage() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 5000).await.unwrap();
    let stats = engine.get_statistics("user1").await.unwrap();
    assert_eq!(stats.total_usage, 0);
    assert_eq!(stats.operation_count, 0);
    assert!(stats.current_budget.is_some());
    assert_eq!(stats.current_budget.unwrap().total, 5000);
}

#[tokio::test]
async fn test_usage_statistics_fields_are_public() {
    // Verify UsageStatistics public field access compiles
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("u", 100).await.unwrap();
    engine.consume_tokens("u", 50, "op").await.unwrap();
    let stats: UsageStatistics = engine.get_statistics("u").await.unwrap();
    let _total: u64 = stats.total_usage;
    let _avg: f64 = stats.average_usage;
    let _count: usize = stats.operation_count;
}

// ── Model training ───────────────────────────────────────────────────

#[tokio::test]
async fn test_train_model_insufficient_data() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 100000).await.unwrap();
    for i in 0..5 {
        engine
            .consume_tokens("user1", 100, &format!("op{}", i))
            .await
            .unwrap();
    }
    let result = engine.train_model().await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Cold start: insufficient training data"));
}

#[tokio::test]
async fn test_train_model_with_sufficient_data() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine
        .get_or_create_budget("user1", 10000000)
        .await
        .unwrap();
    for i in 0..110 {
        engine
            .consume_tokens("user1", 100 + (i % 50), &format!("op{}", i))
            .await
            .unwrap();
    }
    let result = engine.train_model().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_train_model_many_users_few_per_user() {
    // 100+ total entries but no user has >= 10 → ColdStart
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    for u in 0..20 {
        let uid = format!("user{}", u);
        engine.get_or_create_budget(&uid, 100000).await.unwrap();
        for i in 0..5 {
            engine
                .consume_tokens(&uid, 100, &format!("op{}", i))
                .await
                .unwrap();
        }
    }
    let result = engine.train_model().await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Cold start: insufficient training data"));
}

// ── Retraining flag ──────────────────────────────────────────────────

#[tokio::test]
async fn test_needs_retraining_fresh_engine() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    assert!(engine.needs_retraining().await);
}

#[tokio::test]
async fn test_needs_retraining_after_successful_train() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine
        .get_or_create_budget("user1", 10_000_000)
        .await
        .unwrap();
    for i in 0..110 {
        engine
            .consume_tokens("user1", 100 + (i % 50), &format!("op{}", i))
            .await
            .unwrap();
    }
    engine.train_model().await.unwrap();
    // after training the model should no longer need retraining
    assert!(!engine.needs_retraining().await);
}

// ── Comprehensive behavioral tests ──────────────────────────────────

/// Test that consuming tokens until budget is exactly zero succeeds,
/// but any further consumption fails — verifying boundary behavior.
#[tokio::test]
async fn test_budget_drain_to_zero_then_reject() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    engine.get_or_create_budget("user1", 1000).await.unwrap();

    // Drain in multiple steps
    engine.consume_tokens("user1", 300, "step1").await.unwrap();
    engine.consume_tokens("user1", 300, "step2").await.unwrap();
    engine.consume_tokens("user1", 400, "step3").await.unwrap();

    // Budget should be exactly zero
    let budget = engine.get_or_create_budget("user1", 1000).await.unwrap();
    assert_eq!(budget.remaining, 0);
    assert_eq!(budget.used, 1000);

    // Even consuming 1 token should fail now
    let result = engine.consume_tokens("user1", 1, "overflow").await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Insufficient") || err_msg.contains("budget"),
        "Error should mention insufficient budget, got: {}",
        err_msg
    );

    // Statistics should reflect all 3 successful operations
    let stats = engine.get_statistics("user1").await.unwrap();
    assert_eq!(stats.operation_count, 3);
    assert_eq!(stats.total_usage, 1000);
    assert_eq!(stats.average_usage, 1000.0 / 3.0);
}

/// Test that optimization allocation changes based on actual usage history,
/// not just cold-start defaults. Verifies the strategy multiplier applies
/// to the predicted usage derived from real consumption patterns.
#[tokio::test]
async fn test_optimize_allocation_reflects_actual_usage_history() {
    let engine = TokenomicsEngine::new(OptimizationStrategy::Conservative);
    engine
        .get_or_create_budget("user1", 1_000_000)
        .await
        .unwrap();

    // Cold start allocation (no history) = 1000 * 0.8 = 800
    let cold_allocation = engine.optimize_allocation("user1").await.unwrap();
    assert_eq!(cold_allocation, 800);

    // Now create usage history with small amounts (avg ~50)
    for i in 0..4 {
        engine
            .consume_tokens("user1", 40 + i * 5, &format!("op{}", i))
            .await
            .unwrap();
    }

    // Fallback path (< 10 data points) uses avg * horizon/3600
    // avg = (40+45+50+55)/4 = 47.5; horizon=3600 → predicted=47 (as u64)
    // conservative = 47 * 0.8 = 37 (as u64)
    let allocation_with_history = engine.optimize_allocation("user1").await.unwrap();
    // The allocation should be much smaller than the cold-start 800
    // because actual usage is small
    assert!(
        allocation_with_history < cold_allocation,
        "Allocation with small usage history ({}) should be less than cold start ({})",
        allocation_with_history,
        cold_allocation
    );
}

/// Test that predictions scale linearly with horizon for the fallback path,
/// and that different strategies produce correctly ordered allocations.
#[tokio::test]
async fn test_strategy_ordering_conservative_lt_balanced_lt_aggressive() {
    // For the same user with no history, allocations should be ordered:
    // Conservative < Balanced < Aggressive
    let conservative = TokenomicsEngine::new(OptimizationStrategy::Conservative);
    let balanced = TokenomicsEngine::new(OptimizationStrategy::Balanced);
    let aggressive = TokenomicsEngine::new(OptimizationStrategy::Aggressive);

    let alloc_c = conservative.optimize_allocation("u").await.unwrap();
    let alloc_b = balanced.optimize_allocation("u").await.unwrap();
    let alloc_a = aggressive.optimize_allocation("u").await.unwrap();

    assert!(
        alloc_c < alloc_b,
        "Conservative ({}) should be less than Balanced ({})",
        alloc_c,
        alloc_b
    );
    assert!(
        alloc_b < alloc_a,
        "Balanced ({}) should be less than Aggressive ({})",
        alloc_b,
        alloc_a
    );

    // Verify exact ratios: conservative=0.8, balanced=1.0, aggressive=1.2
    assert_eq!(alloc_c as f64 / alloc_b as f64, 0.8);
    assert_eq!(alloc_a as f64 / alloc_b as f64, 1.2);
}
