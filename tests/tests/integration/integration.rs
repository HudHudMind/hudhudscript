//! Integration tests for the tokenomics system
//!
//! These tests verify that all subsystems work together correctly.

use chrono::Utc;
use hudhudscript_tokenomics::bandit::{BanditContext, BanditReward, ContextualBandit};
use hudhudscript_tokenomics::batch::{BatchQueue, DispatchMode};
use hudhudscript_tokenomics::cascade::CascadeRouter;
use hudhudscript_tokenomics::cli;
use hudhudscript_tokenomics::compression::{CompressionLevel, PromptCompressor};
use hudhudscript_tokenomics::config::TokenomicsConfig;
use hudhudscript_tokenomics::enforcement::EnforcementDecision;
use hudhudscript_tokenomics::integration::TokenomicsManager;
use hudhudscript_tokenomics::pricing::PricingRegistry;
use hudhudscript_tokenomics::response_cache::{CacheStrategy, CachedResponse, ResponseCache};
use hudhudscript_tokenomics::streaming::{StreamAction, StreamingTokenCounter};
use hudhudscript_tokenomics::thinking::{TaskComplexity, ThinkingBudgetController};
use hudhudscript_tokenomics::tool_tracking::{ToolCallTracker, ToolCallType};

// ─── Config Parse ────────────────────────────────────────────────────────────

#[test]
fn test_full_config_parse_and_manager_creation() {
    let toml_str = r#"
        enabled = true
        strategy = "balanced"

        [budget]
        max_tokens_per_call = 8000
        max_tokens_per_day = 200000
        max_tokens_per_month = 5000000

        [alerts]
        on_warning = "log"
        on_critical = "notify"
        on_depleted = "block"

        [attribution]
        enabled = true
    "#;
    let config: TokenomicsConfig = toml::from_str(toml_str).unwrap();
    let mgr = TokenomicsManager::from_config(config);
    assert!(mgr.is_enabled());
}

// ─── Budget Enforcement Flow ─────────────────────────────────────────────────

#[test]
fn test_budget_enforcement_per_call() {
    let mut config = TokenomicsConfig::default();
    config.budget.max_tokens_per_call = 4000;
    let mgr = TokenomicsManager::from_config(config);

    assert_eq!(mgr.pre_call_check(3000), EnforcementDecision::Allowed);
    match mgr.pre_call_check(5000) {
        EnforcementDecision::Blocked(msg) => assert!(msg.contains("per-call")),
        other => panic!("Expected blocked, got {:?}", other),
    }
}

#[test]
fn test_budget_enforcement_daily_accumulation() {
    let mut config = TokenomicsConfig::default();
    config.budget.max_tokens_per_call = 50000;
    config.budget.max_tokens_per_day = 10000;
    config.alerts.on_depleted = "block".into();
    let mgr = TokenomicsManager::from_config(config);

    // Record usage that fills daily budget
    mgr.post_call_record(
        "claude-sonnet-4",
        "anthropic",
        5000,
        4500,
        0,
        0,
        false,
        None,
        None,
        None,
    );

    // Next request should be blocked (9500 + 2000 > 10000)
    match mgr.pre_call_check(2000) {
        EnforcementDecision::Blocked(_) => {}
        other => panic!("Expected blocked, got {:?}", other),
    }
}

// ─── Pricing + Cost Calculation ──────────────────────────────────────────────

#[test]
fn test_pricing_all_providers() {
    let reg = PricingRegistry::new();
    // All major models should exist
    assert!(reg.get_pricing("claude-opus-4").is_some());
    assert!(reg.get_pricing("claude-sonnet-4").is_some());
    assert!(reg.get_pricing("claude-haiku-3.5").is_some());
    assert!(reg.get_pricing("gpt-4o").is_some());
    assert!(reg.get_pricing("gpt-4o-mini").is_some());
    assert!(reg.get_pricing("o1").is_some());
    assert!(reg.get_pricing("o3").is_some());
    assert!(reg.get_pricing("deepseek-v3").is_some());
    assert!(reg.get_pricing("deepseek-r1").is_some());
    assert!(reg.get_pricing("gemini-1.5-pro").is_some());
    assert!(reg.get_pricing("gemini-1.5-flash").is_some());
}

#[test]
fn test_cost_with_caching_and_thinking() {
    let reg = PricingRegistry::new();
    // Opus: 10K input, 2K output, 5K cached, 8K thinking
    let cost = reg
        .calculate_cost("claude-opus-4", 10000, 2000, 5000, 8000, false)
        .unwrap();
    assert!(cost.total_cost > 0.0);
    assert!(cost.thinking_cost > 0.0);
    assert!(cost.cached_input_cost > 0.0);

    // Batch should be cheaper
    let batch_cost = reg
        .calculate_cost("claude-opus-4", 10000, 2000, 5000, 8000, true)
        .unwrap();
    assert!(batch_cost.total_cost < cost.total_cost);
}

// ─── Cascade Routing ─────────────────────────────────────────────────────────

#[test]
fn test_cascade_routes_by_complexity() {
    let router = CascadeRouter::with_defaults();
    let simple = router.route("Hi", None);
    let hard = router.route(
        "Prove the Riemann hypothesis using analytic continuation",
        None,
    );
    assert_ne!(simple.tier_name, hard.tier_name);
}

#[test]
fn test_cascade_budget_downgrade() {
    let mut router = CascadeRouter::with_defaults();
    router.set_budget_remaining(0.05);
    let decision = router.route("Prove the Riemann hypothesis", None);
    assert!(decision.reason.contains("Downgraded"));
}

// ─── Bandit Learns ───────────────────────────────────────────────────────────

#[test]
fn test_bandit_learns_from_rewards() {
    let mut bandit = ContextualBandit::with_defaults();
    let ctx = BanditContext {
        task_type: "chat".into(),
        complexity: "simple".into(),
    };

    for _ in 0..30 {
        bandit.select(&ctx);
        bandit.update(
            &ctx,
            "gpt-4o-mini",
            &BanditReward {
                quality_score: 0.9,
                cost_usd: 0.0001,
                latency_ms: 50,
            },
        );
        bandit.update(
            &ctx,
            "claude-opus-4",
            &BanditReward {
                quality_score: 0.5,
                cost_usd: 0.10,
                latency_ms: 5000,
            },
        );
    }

    let estimates = bandit.model_estimates(&ctx);
    let mini_est = estimates
        .iter()
        .find(|(m, _)| m == "gpt-4o-mini")
        .unwrap()
        .1;
    let opus_est = estimates
        .iter()
        .find(|(m, _)| m == "claude-opus-4")
        .unwrap()
        .1;
    assert!(mini_est > opus_est);
}

// ─── Response Cache ──────────────────────────────────────────────────────────

#[test]
fn test_cache_exact_and_semantic() {
    let mut cache = ResponseCache::new(100, CacheStrategy::Hybrid, 0.90);

    let resp = CachedResponse {
        content: "Answer".into(),
        model: "test".into(),
        tokens_used: 100,
        cached_at: Utc::now(),
        ttl_seconds: 300,
        hit_count: 0,
        estimated_cost_usd: 0.05,
    };

    // Exact match
    let key1 = ResponseCache::cache_key("gpt-4o", "hello", "", 0.0, 0);
    cache.put(key1, resp.clone());
    assert!(cache.get(key1, None).is_some());
    let key2 = ResponseCache::cache_key("gpt-4o", "different", "", 0.0, 0);
    assert!(cache.get(key2, None).is_none());

    // Semantic match
    let emb1 = vec![1.0, 0.0, 0.0];
    let emb2 = vec![0.98, 0.1, 0.0];
    let key3 = ResponseCache::cache_key("m", "q1", "", 0.0, 0);
    cache.put_with_embedding(key3, resp, emb1);
    assert!(cache.get_semantic(&emb2).is_some());
}

// ─── Streaming Budget ────────────────────────────────────────────────────────

#[test]
fn test_streaming_cancel_on_budget() {
    let mut counter = StreamingTokenCounter::new(10);
    let action = counter.process_chunk(&"x".repeat(100));
    assert!(matches!(action, StreamAction::Cancel { .. }));
    assert!(counter.is_cancelled());
}

// ─── Batch Queue ─────────────────────────────────────────────────────────────

#[test]
fn test_batch_dispatch_classification() {
    let q = BatchQueue::new(100, 30, true);
    assert_eq!(
        q.classify_dispatch(true, false, false),
        DispatchMode::Stream
    );
    assert_eq!(
        q.classify_dispatch(false, true, false),
        DispatchMode::Stream
    );
    assert_eq!(
        q.classify_dispatch(false, false, false),
        DispatchMode::Batch
    ); // auto_promote
}

// ─── Prompt Compression ──────────────────────────────────────────────────────

#[test]
fn test_compression_pipeline() {
    let compressor = PromptCompressor::new(CompressionLevel::Medium);
    let input =
        "The quick brown fox jumps over the lazy dog and then the fox ran away very quickly";
    let result = compressor.compress(input);
    assert!(result.compressed_tokens <= result.original_tokens);
    assert!(
        result.reduction_pct > 0.0,
        "medium compression on redundant text should achieve some reduction",
    );
    // Verify compressed content is non-empty and shorter than original
    assert!(
        !result.compressed.is_empty(),
        "compressed output must not be empty"
    );
    assert!(
        result.compressed.len() <= input.len(),
        "compressed output ({} bytes) should not exceed original ({} bytes)",
        result.compressed.len(),
        input.len(),
    );
    // Original token count should reflect actual input
    assert!(
        result.original_tokens > 0,
        "original tokens must be positive"
    );
}

// ─── Thinking Budget ─────────────────────────────────────────────────────────

#[test]
fn test_thinking_tiers_with_classification() {
    let ctrl = ThinkingBudgetController::with_defaults();

    // Verify all tiers have monotonically increasing budgets
    let simple = ctrl.budget_for_complexity(TaskComplexity::Simple);
    let medium = ctrl.budget_for_complexity(TaskComplexity::Medium);
    let hard = ctrl.budget_for_complexity(TaskComplexity::Hard);
    let expert = ctrl.budget_for_complexity(TaskComplexity::Expert);
    assert_eq!(simple, 1024);
    assert_eq!(expert, 65536);
    assert!(simple < medium, "simple < medium");
    assert!(medium < hard, "medium < hard");
    assert!(hard < expert, "hard < expert");

    // Integration: classify a prompt then look up its budget
    let complexity = ctrl.classify_task(
        "Prove the Riemann hypothesis using analytic continuation and the functional equation",
        false,
        true, // has_math
    );
    let budget = ctrl.budget_for_complexity(complexity);
    // A math prompt (has_math=true) should classify as Hard or Expert
    assert!(
        budget >= hard,
        "math prompt should classify at Hard or higher, got budget {}",
        budget,
    );
}

// ─── Tool Tracking ───────────────────────────────────────────────────────────

#[test]
fn test_tool_tracking_aggregation() {
    let mut tracker = ToolCallTracker::new();
    tracker.set_cost_rate("search".into(), 0.001);
    tracker.record_call(
        ToolCallType::Mcp,
        "search",
        Some("github"),
        100,
        50,
        500,
        true,
    );
    tracker.record_call(
        ToolCallType::Mcp,
        "search",
        Some("github"),
        200,
        60,
        600,
        true,
    );
    tracker.record_call(ToolCallType::Http, "fetch", None, 150, 100, 1000, false);

    let by_type = tracker.stats_by_type();
    assert_eq!(by_type[&ToolCallType::Mcp].total_calls, 2);
    assert_eq!(by_type[&ToolCallType::Http].failed_calls, 1);

    let by_name = tracker.stats_by_name();
    assert_eq!(by_name["search"].total_calls, 2);
}

// ─── CLI Formatters ──────────────────────────────────────────────────────────

#[test]
fn test_cli_status_format() {
    let summary = hudhudscript_tokenomics::enforcement::UsageSummary {
        daily_usage: 75000,
        daily_limit: 100000,
        daily_pct: 0.75,
        monthly_usage: 500000,
        monthly_limit: 3000000,
        monthly_pct: 0.1667,
        health: hudhudscript_tokenomics::budget::BudgetHealth::Warning,
    };
    let output = cli::format_usage_summary(&summary);
    assert!(output.contains("[WARN]"));
    assert!(output.contains("75000"));
}

// ─── End-to-End: Manager + Cost + Attribution ────────────────────────────────

#[test]
fn test_end_to_end_flow() {
    let mut config = TokenomicsConfig::default();
    config.attribution.enabled = true;
    let mgr = TokenomicsManager::from_config(config);

    // Pre-check
    assert_eq!(mgr.pre_call_check(1000), EnforcementDecision::Allowed);

    // Simulate 3 calls
    for i in 0..3 {
        mgr.post_call_record(
            "claude-sonnet-4",
            "anthropic",
            1000,
            500,
            200,
            0,
            false,
            Some(&format!("user{}", i % 2)),
            None,
            Some("chat"),
        );
    }

    // Verify attribution
    assert_eq!(mgr.total_events(), 3);
    assert!(mgr.total_cost() > 0.0);

    let by_feature = mgr.cost_by_feature();
    assert!(by_feature.contains_key("chat"));

    let by_user = mgr.cost_by_user();
    assert!(by_user.contains_key("user0"));
    assert!(by_user.contains_key("user1"));

    // Verify usage tracked: 3 calls * (1000 input + 500 output + 0 thinking) = 4500
    let summary = mgr.usage_summary();
    assert_eq!(
        summary.daily_usage, 4500,
        "3 calls * 1500 tokens each = 4500"
    );
}
