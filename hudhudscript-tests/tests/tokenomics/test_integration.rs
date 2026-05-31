//! Public API tests for `hudhudscript_tokenomics::integration`.
//! Tests cover TokenomicsManager and create_shared_manager.

use hudhudscript_tokenomics::config::TokenomicsConfig;
use hudhudscript_tokenomics::enforcement::EnforcementDecision;
use hudhudscript_tokenomics::integration::{create_shared_manager, TokenomicsManager};
use hudhudscript_tokenomics::streaming::StreamAction;

fn test_config() -> TokenomicsConfig {
    let mut config = TokenomicsConfig::default();
    config.attribution.enabled = true;
    config
}

fn disabled_config() -> TokenomicsConfig {
    let mut config = test_config();
    config.enabled = false;
    config
}

// ── Construction ─────────────────────────────────────────────────────

#[test]
fn test_manager_creation() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert!(mgr.is_enabled());
}

#[test]
fn test_manager_has_pricing_for_known_model() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert!(mgr.pricing().get_pricing("claude-sonnet-4").is_some());
}

#[test]
fn test_manager_initial_daily_usage_zero() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert_eq!(mgr.usage_summary().daily_usage, 0);
}

#[test]
fn test_manager_initial_events_zero() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert_eq!(mgr.total_events(), 0);
    assert_eq!(mgr.total_cost(), 0.0);
}

#[test]
fn test_manager_initial_alerts_empty() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert_eq!(mgr.alerts().len(), 0);
}

// ── Disabled manager ──────────────────────────────────────────────────

#[test]
fn test_disabled_manager_is_not_enabled() {
    let mgr = TokenomicsManager::from_config(disabled_config());
    assert!(!mgr.is_enabled());
}

#[test]
fn test_disabled_pre_call_always_allowed() {
    let mgr = TokenomicsManager::from_config(disabled_config());
    assert_eq!(mgr.pre_call_check(999999), EnforcementDecision::Allowed);
}

#[test]
fn test_disabled_post_call_returns_none() {
    let mgr = TokenomicsManager::from_config(disabled_config());
    let cost = mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    assert!(cost.is_none());
}

#[test]
fn test_disabled_post_call_does_not_track_usage() {
    let mgr = TokenomicsManager::from_config(disabled_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    assert_eq!(mgr.usage_summary().daily_usage, 0);
}

// ── pre_call_check ────────────────────────────────────────────────────

#[test]
fn test_pre_call_allowed_within_limit() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert_eq!(mgr.pre_call_check(1000), EnforcementDecision::Allowed);
}

#[test]
fn test_pre_call_blocked_exceeds_per_call_limit() {
    let mgr = TokenomicsManager::from_config(test_config());
    // default per-call limit is 4000
    match mgr.pre_call_check(5000) {
        EnforcementDecision::Blocked(_) => {}
        other => panic!("Expected Blocked, got {:?}", other),
    }
}

#[test]
fn test_pre_call_at_exact_limit_is_allowed() {
    let mgr = TokenomicsManager::from_config(test_config());
    // exactly 4000 should be allowed (not over limit)
    assert_eq!(mgr.pre_call_check(4000), EnforcementDecision::Allowed);
}

#[test]
fn test_pre_call_zero_tokens_allowed() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert_eq!(mgr.pre_call_check(0), EnforcementDecision::Allowed);
}

// ── post_call_record ──────────────────────────────────────────────────

#[test]
fn test_post_call_records_cost_for_known_model() {
    let mgr = TokenomicsManager::from_config(test_config());
    let cost = mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        Some("user1"),
        Some("sess1"),
        Some("chat"),
    );
    assert!(cost.is_some());
    assert!(cost.unwrap().total_cost > 0.0);
}

#[test]
fn test_post_call_unknown_model_returns_none() {
    let mgr = TokenomicsManager::from_config(test_config());
    let cost = mgr.post_call_record(
        "nonexistent-model-9000",
        "unknown",
        1000,
        500,
        0,
        0,
        false,
        Some("user1"),
        None,
        Some("chat"),
    );
    assert!(cost.is_none());
}

#[test]
fn test_post_call_tracks_usage_even_without_pricing() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "nonexistent",
        "unk",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    // enforcer records input + output + thinking = 1500
    assert_eq!(mgr.usage_summary().daily_usage, 1500);
}

#[test]
fn test_post_call_unknown_model_no_attribution_event() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "nonexistent",
        "unk",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    assert_eq!(mgr.total_events(), 0);
}

#[test]
fn test_post_call_usage_accumulates() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        500,
        250,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    // 1500 + 750 = 2250
    assert_eq!(mgr.usage_summary().daily_usage, 2250);
}

// ── Attribution tracking ──────────────────────────────────────────────

#[test]
fn test_attribution_tracking_two_calls() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        Some("alice"),
        None,
        Some("chat"),
    );
    mgr.post_call_record(
        "gpt-4o",
        "openai",
        2000,
        1000,
        0,
        0,
        false,
        Some("bob"),
        None,
        Some("search"),
    );
    assert_eq!(mgr.total_events(), 2);
    assert!(mgr.total_cost() > 0.0);
}

#[test]
fn test_cost_by_feature() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        Some("chat"),
    );
    mgr.post_call_record(
        "gpt-4o",
        "openai",
        2000,
        1000,
        0,
        0,
        false,
        None,
        None,
        Some("search"),
    );
    let by_feature = mgr.cost_by_feature();
    assert!(by_feature.contains_key("chat"));
    assert!(by_feature.contains_key("search"));
}

#[test]
fn test_cost_by_user() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        Some("alice"),
        None,
        None,
    );
    mgr.post_call_record(
        "gpt-4o",
        "openai",
        2000,
        1000,
        0,
        0,
        false,
        Some("bob"),
        None,
        None,
    );
    let by_user = mgr.cost_by_user();
    assert!(by_user.contains_key("alice"));
    assert!(by_user.contains_key("bob"));
}

#[test]
fn test_cost_by_model() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    let by_model = mgr.cost_by_model();
    assert!(by_model.contains_key("claude-sonnet-4"));
}

#[test]
fn test_disabled_attribution_not_recorded() {
    let mut config = test_config();
    config.attribution.enabled = false;
    let mgr = TokenomicsManager::from_config(config);
    let cost = mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        Some("alice"),
        None,
        Some("chat"),
    );
    // cost is still returned even without attribution recording
    assert!(cost.is_some());
    assert_eq!(mgr.total_events(), 0);
    assert_eq!(mgr.total_cost(), 0.0);
    assert!(mgr.cost_by_feature().is_empty());
    assert!(mgr.cost_by_user().is_empty());
}

// ── Usage summary ──────────────────────────────────────────────────────

#[test]
fn test_usage_summary_after_one_call() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        1000,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    // input + output + thinking = 1500
    assert_eq!(mgr.usage_summary().daily_usage, 1500);
}

#[test]
fn test_usage_summary_with_thinking_tokens() {
    let mgr = TokenomicsManager::from_config(test_config());
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        500,
        250,
        0,
        100,
        false,
        None,
        None,
        None,
    );
    // 500 + 250 + 100 = 850
    assert_eq!(mgr.usage_summary().daily_usage, 850);
}

// ── Streaming ─────────────────────────────────────────────────────────

#[test]
fn test_stream_counter_continue_on_small_chunk() {
    let mgr = TokenomicsManager::from_config(test_config());
    let mut counter = mgr.create_stream_counter();
    let action = counter.process_chunk("hello world");
    assert_eq!(action, StreamAction::Continue);
}

// ── Shared manager ────────────────────────────────────────────────────

#[test]
fn test_shared_manager_creation() {
    let shared = create_shared_manager(test_config());
    assert!(shared.is_enabled());
}

#[test]
fn test_shared_manager_clone_shares_state() {
    let shared = create_shared_manager(test_config());
    let cloned = shared.clone();
    assert!(cloned.is_enabled());
    // both refs point to the same manager
    assert_eq!(
        cloned.usage_summary().daily_usage,
        shared.usage_summary().daily_usage
    );
}

// ── Config accessor ───────────────────────────────────────────────────

#[test]
fn test_config_accessor() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert!(mgr.config().enabled);
    assert!(mgr.config().attribution.enabled);
}

#[test]
fn test_config_accessor_strategy() {
    let mgr = TokenomicsManager::from_config(test_config());
    assert_eq!(mgr.config().strategy, "balanced");
}

// ── Comprehensive behavioral tests ──────────────────────────────────

/// Verify that repeated calls exhaust the daily budget and trigger blocking.
/// This tests the full lifecycle: allowed -> warning -> blocked.
#[test]
fn test_budget_exhaustion_lifecycle_allowed_then_warning_then_blocked() {
    let mut config = test_config();
    // Use small daily limit so we can exhaust it quickly
    config.budget.max_tokens_per_call = 5000;
    config.budget.max_tokens_per_day = 10000;
    config.budget.max_tokens_per_month = 1_000_000;
    config.budget.alert_threshold = 0.50; // warn at 50%
    config.alerts.on_warning = "log".to_string();
    config.alerts.on_critical = "log".to_string();
    config.alerts.on_depleted = "block".to_string();
    let mgr = TokenomicsManager::from_config(config);

    // Phase 1: small request is allowed, no alerts yet
    assert_eq!(mgr.pre_call_check(1000), EnforcementDecision::Allowed);
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        500,
        500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    assert_eq!(mgr.usage_summary().daily_usage, 1000);
    assert_eq!(mgr.alerts().len(), 0);

    // Phase 2: push past 50% threshold -> warning on next check
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        2500,
        2500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    assert_eq!(mgr.usage_summary().daily_usage, 6000); // 60% of 10000
    match mgr.pre_call_check(100) {
        EnforcementDecision::Warning(msg) => {
            assert!(msg.contains("warning") || msg.contains("Warning") || msg.len() > 0);
        }
        other => panic!(
            "Expected Warning after exceeding 50% threshold, got {:?}",
            other
        ),
    }

    // Phase 3: push past daily limit -> blocked
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        2500,
        2500,
        0,
        0,
        false,
        None,
        None,
        None,
    );
    assert_eq!(mgr.usage_summary().daily_usage, 11000); // over 10000
    match mgr.pre_call_check(1000) {
        EnforcementDecision::Blocked(msg) => {
            assert!(msg.contains("Daily") || msg.contains("daily"));
        }
        other => panic!(
            "Expected Blocked after daily limit exceeded, got {:?}",
            other
        ),
    }
}

/// Verify that cost attribution correctly accumulates across multiple models
/// and that per-model/per-user/per-feature breakdowns are accurate.
#[test]
fn test_cost_attribution_accuracy_across_models_and_users() {
    let mgr = TokenomicsManager::from_config(test_config());

    // Record calls for different models, users, and features
    let cost_sonnet = mgr
        .post_call_record(
            "claude-sonnet-4",
            "anthropic",
            1000,
            500,
            0,
            0,
            false,
            Some("alice"),
            Some("s1"),
            Some("chat"),
        )
        .unwrap();

    let cost_opus = mgr
        .post_call_record(
            "claude-opus-4",
            "anthropic",
            1000,
            500,
            0,
            0,
            false,
            Some("bob"),
            Some("s2"),
            Some("search"),
        )
        .unwrap();

    let cost_haiku = mgr
        .post_call_record(
            "claude-haiku-3.5",
            "anthropic",
            1000,
            500,
            0,
            0,
            false,
            Some("alice"),
            Some("s1"),
            Some("chat"),
        )
        .unwrap();

    // Total cost should be the sum of individual costs
    let total = mgr.total_cost();
    let expected_total = cost_sonnet.total_cost + cost_opus.total_cost + cost_haiku.total_cost;
    assert!(
        (total - expected_total).abs() < 1e-10,
        "Total cost {} should equal sum of individual costs {}",
        total,
        expected_total
    );

    // Opus should be more expensive than Sonnet for the same token counts
    assert!(
        cost_opus.total_cost > cost_sonnet.total_cost,
        "Opus ({}) should cost more than Sonnet ({})",
        cost_opus.total_cost,
        cost_sonnet.total_cost
    );

    // Haiku should be cheapest
    assert!(
        cost_haiku.total_cost < cost_sonnet.total_cost,
        "Haiku ({}) should cost less than Sonnet ({})",
        cost_haiku.total_cost,
        cost_sonnet.total_cost
    );

    // Per-user: alice has 2 calls, bob has 1
    let by_user = mgr.cost_by_user();
    let alice_cost = by_user["alice"];
    let bob_cost = by_user["bob"];
    assert!((alice_cost - (cost_sonnet.total_cost + cost_haiku.total_cost)).abs() < 1e-10);
    assert!((bob_cost - cost_opus.total_cost).abs() < 1e-10);

    // Per-feature: chat has 2 calls, search has 1
    let by_feature = mgr.cost_by_feature();
    assert!((by_feature["chat"] - (cost_sonnet.total_cost + cost_haiku.total_cost)).abs() < 1e-10);
    assert!((by_feature["search"] - cost_opus.total_cost).abs() < 1e-10);

    // Per-model: each model has exactly 1 call
    let by_model = mgr.cost_by_model();
    assert!((by_model["claude-sonnet-4"] - cost_sonnet.total_cost).abs() < 1e-10);
    assert!((by_model["claude-opus-4"] - cost_opus.total_cost).abs() < 1e-10);
    assert!((by_model["claude-haiku-3.5"] - cost_haiku.total_cost).abs() < 1e-10);

    assert_eq!(mgr.total_events(), 3);
}

/// Verify that the stream counter created by the manager respects the
/// per-call budget from config and produces correct Cancel actions.
#[test]
fn test_stream_counter_respects_config_budget_and_cancels() {
    let mut config = test_config();
    config.budget.max_tokens_per_call = 20; // very tight budget
    let mgr = TokenomicsManager::from_config(config);

    let mut counter = mgr.create_stream_counter();

    // Small chunk: should continue
    let action1 = counter.process_chunk("hello");
    assert_eq!(action1, StreamAction::Continue);
    assert!(counter.current_count() > 0);
    assert!(!counter.is_cancelled());

    // Large chunk that exceeds 20-token budget
    let action2 = counter.process_chunk(&"a".repeat(200)); // ~50 tokens
    match action2 {
        StreamAction::Cancel {
            estimated_tokens,
            budget,
        } => {
            assert_eq!(budget, 20);
            assert!(estimated_tokens > 20);
        }
        StreamAction::Warning { .. } => {
            // Warning is also acceptable if at threshold but not over
            // Push further to get Cancel
            let action3 = counter.process_chunk(&"b".repeat(200));
            assert!(matches!(action3, StreamAction::Cancel { .. }));
        }
        StreamAction::Continue => {
            panic!("Expected Cancel or Warning for large chunk on tight budget")
        }
    }

    // Once cancelled, stays cancelled
    assert!(counter.is_cancelled());
    let action_after = counter.process_chunk("more");
    assert!(matches!(action_after, StreamAction::Cancel { .. }));
}
