//! Tests for tokenomics::bandit — ContextualBandit, BanditContext, BanditReward, ModelArm

use hudhudscript_tokenomics::bandit::*;

fn test_context() -> BanditContext {
    BanditContext {
        task_type: "chat".into(),
        complexity: "simple".into(),
    }
}

// ---------------------------------------------------------------------------
// with_defaults — construction
// ---------------------------------------------------------------------------

#[test]
fn test_with_defaults_arm_count() {
    let bandit = ContextualBandit::with_defaults();
    assert_eq!(bandit.arm_count(), 6);
}

#[test]
fn test_with_defaults_total_selections_zero() {
    let bandit = ContextualBandit::with_defaults();
    assert_eq!(bandit.total_selections(), 0);
}

#[test]
fn test_with_defaults_initial_estimates_uniform() {
    let bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    let estimates = bandit.model_estimates(&ctx);
    assert_eq!(estimates.len(), 6);
    for (_, est) in &estimates {
        assert_eq!(*est, 0.5);
    }
}

// ---------------------------------------------------------------------------
// custom construction
// ---------------------------------------------------------------------------

#[test]
fn test_custom_arms() {
    let arms = vec![
        ModelArm {
            model: "m1".into(),
            provider: "p".into(),
            cost_per_1k: 0.01,
        },
        ModelArm {
            model: "m2".into(),
            provider: "p".into(),
            cost_per_1k: 0.02,
        },
    ];
    let bandit = ContextualBandit::new(arms, 0.3, 0.6, 0.1);
    assert_eq!(bandit.arm_count(), 2);
}

// ---------------------------------------------------------------------------
// select
// ---------------------------------------------------------------------------

#[test]
fn test_select_returns_arm() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    let arm = bandit.select(&ctx);
    assert!(!arm.model.is_empty());
    assert!(!arm.provider.is_empty());
}

#[test]
fn test_select_increments_total() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);
    assert_eq!(bandit.total_selections(), 1);
    bandit.select(&ctx);
    assert_eq!(bandit.total_selections(), 2);
}

#[test]
fn test_select_multiple_contexts() {
    let mut bandit = ContextualBandit::with_defaults();
    let simple = BanditContext {
        task_type: "chat".into(),
        complexity: "simple".into(),
    };
    let hard = BanditContext {
        task_type: "code".into(),
        complexity: "hard".into(),
    };
    bandit.select(&simple);
    bandit.select(&hard);
    assert_eq!(bandit.total_selections(), 2);
}

// ---------------------------------------------------------------------------
// update — shifts preferences
// ---------------------------------------------------------------------------

#[test]
fn test_update_shifts_preference() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);

    for _ in 0..50 {
        bandit.update(
            &ctx,
            "claude-haiku-3.5",
            &BanditReward {
                quality_score: 0.95,
                cost_usd: 0.001,
                latency_ms: 200,
            },
        );
        bandit.update(
            &ctx,
            "claude-opus-4",
            &BanditReward {
                quality_score: 0.3,
                cost_usd: 0.10,
                latency_ms: 5000,
            },
        );
    }

    let estimates = bandit.model_estimates(&ctx);
    let haiku = estimates
        .iter()
        .find(|(m, _)| m == "claude-haiku-3.5")
        .unwrap()
        .1;
    let opus = estimates
        .iter()
        .find(|(m, _)| m == "claude-opus-4")
        .unwrap()
        .1;
    assert!(haiku > opus);
}

#[test]
fn test_update_good_reward_increases_estimate() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);

    let good = BanditReward {
        quality_score: 0.95,
        cost_usd: 0.001,
        latency_ms: 100,
    };
    for _ in 0..100 {
        bandit.update(&ctx, "claude-haiku-3.5", &good);
    }

    let estimates = bandit.model_estimates(&ctx);
    let haiku = estimates
        .iter()
        .find(|(m, _)| m == "claude-haiku-3.5")
        .unwrap()
        .1;
    assert!(haiku > 0.7);
}

#[test]
fn test_update_bad_reward_decreases_estimate() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);

    let bad = BanditReward {
        quality_score: 0.1,
        cost_usd: 0.10,
        latency_ms: 8000,
    };
    for _ in 0..100 {
        bandit.update(&ctx, "claude-opus-4", &bad);
    }

    let estimates = bandit.model_estimates(&ctx);
    let opus = estimates
        .iter()
        .find(|(m, _)| m == "claude-opus-4")
        .unwrap()
        .1;
    assert!(opus < 0.4);
}

#[test]
fn test_untouched_arm_stays_at_prior() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);

    let good = BanditReward {
        quality_score: 0.95,
        cost_usd: 0.001,
        latency_ms: 100,
    };
    for _ in 0..100 {
        bandit.update(&ctx, "claude-haiku-3.5", &good);
    }

    let estimates = bandit.model_estimates(&ctx);
    let sonnet = estimates
        .iter()
        .find(|(m, _)| m == "claude-sonnet-4")
        .unwrap()
        .1;
    assert!((sonnet - 0.5).abs() < 0.01);
}

// ---------------------------------------------------------------------------
// model_estimates
// ---------------------------------------------------------------------------

#[test]
fn test_model_estimates_unknown_context() {
    let bandit = ContextualBandit::with_defaults();
    let ctx = BanditContext {
        task_type: "unknown".into(),
        complexity: "unknown".into(),
    };
    let estimates = bandit.model_estimates(&ctx);
    for (_, est) in &estimates {
        assert!((*est - 0.5).abs() < 0.01);
    }
}

#[test]
fn test_model_estimates_sorted_descending() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);

    bandit.update(
        &ctx,
        "claude-haiku-3.5",
        &BanditReward {
            quality_score: 0.95,
            cost_usd: 0.001,
            latency_ms: 100,
        },
    );

    let estimates = bandit.model_estimates(&ctx);
    for w in estimates.windows(2) {
        assert!(w[0].1 >= w[1].1);
    }
}

#[test]
fn test_model_estimates_length_equals_arms() {
    let bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    assert_eq!(bandit.model_estimates(&ctx).len(), 6);
}

// ---------------------------------------------------------------------------
// observations
// ---------------------------------------------------------------------------

#[test]
fn test_observations_new_context() {
    let bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    assert_eq!(bandit.observations(&ctx), 0.0);
}

#[test]
fn test_observations_after_update() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);
    bandit.update(
        &ctx,
        "claude-haiku-3.5",
        &BanditReward {
            quality_score: 0.8,
            cost_usd: 0.001,
            latency_ms: 100,
        },
    );
    let obs = bandit.observations(&ctx);
    assert!(obs > 0.0);
}

#[test]
fn test_observations_accumulate() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);

    let reward = BanditReward {
        quality_score: 0.8,
        cost_usd: 0.001,
        latency_ms: 100,
    };
    bandit.update(&ctx, "claude-haiku-3.5", &reward);
    let obs1 = bandit.observations(&ctx);
    bandit.update(&ctx, "claude-haiku-3.5", &reward);
    let obs2 = bandit.observations(&ctx);
    assert!(obs2 > obs1);
}

// ---------------------------------------------------------------------------
// total_selections / arm_count
// ---------------------------------------------------------------------------

#[test]
fn test_total_selections_tracks_correctly() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    for _ in 0..10 {
        bandit.select(&ctx);
    }
    assert_eq!(bandit.total_selections(), 10);
}

#[test]
fn test_arm_count_custom() {
    let arms = vec![ModelArm {
        model: "a".into(),
        provider: "p".into(),
        cost_per_1k: 0.01,
    }];
    let bandit = ContextualBandit::new(arms, 0.3, 0.6, 0.1);
    assert_eq!(bandit.arm_count(), 1);
}

// ---------------------------------------------------------------------------
// ModelArm fields
// ---------------------------------------------------------------------------

#[test]
fn test_model_arm_fields() {
    let arm = ModelArm {
        model: "gpt-4o".into(),
        provider: "openai".into(),
        cost_per_1k: 0.00625,
    };
    assert_eq!(arm.model, "gpt-4o");
    assert_eq!(arm.provider, "openai");
    assert_eq!(arm.cost_per_1k, 0.00625);
}

// ---------------------------------------------------------------------------
// BanditReward fields
// ---------------------------------------------------------------------------

#[test]
fn test_bandit_reward_fields() {
    let reward = BanditReward {
        quality_score: 0.9,
        cost_usd: 0.05,
        latency_ms: 500,
    };
    assert_eq!(reward.quality_score, 0.9);
    assert_eq!(reward.cost_usd, 0.05);
    assert_eq!(reward.latency_ms, 500);
}

// ---------------------------------------------------------------------------
// Ordering after many updates
// ---------------------------------------------------------------------------

#[test]
fn test_ordering_after_rewards() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = test_context();
    bandit.select(&ctx);

    let good = BanditReward {
        quality_score: 0.95,
        cost_usd: 0.001,
        latency_ms: 100,
    };
    let bad = BanditReward {
        quality_score: 0.1,
        cost_usd: 0.10,
        latency_ms: 8000,
    };

    for _ in 0..100 {
        bandit.update(&ctx, "claude-haiku-3.5", &good);
        bandit.update(&ctx, "claude-opus-4", &bad);
    }

    let estimates = bandit.model_estimates(&ctx);
    let haiku = estimates
        .iter()
        .find(|(m, _)| m == "claude-haiku-3.5")
        .unwrap()
        .1;
    let sonnet = estimates
        .iter()
        .find(|(m, _)| m == "claude-sonnet-4")
        .unwrap()
        .1;
    let opus = estimates
        .iter()
        .find(|(m, _)| m == "claude-opus-4")
        .unwrap()
        .1;
    assert!(haiku > sonnet);
    assert!(sonnet > opus);
}
