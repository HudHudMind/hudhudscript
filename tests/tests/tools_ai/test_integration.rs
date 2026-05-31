//! Integration and edge-case tests for hudhudscript-tools-ai.
//!
//! Covers: serde roundtrips, thread safety, custom backends, edge cases
//! in token estimation, context window boundary conditions, and cross-module
//! interactions (CostTracker -> Analytics -> JSON roundtrip).

use hudhudscript_tools_ai::analytics::AnalyticsReport;
use hudhudscript_tools_ai::cost::{default_pricing, AiTokenUsage};
use hudhudscript_tools_ai::{
    estimate_tokens, Analytics, BudgetConfig, ContextWindow, Conversation, CostTracker,
    FallbackChainBuilder, InMemoryBackend, MemoryBackend, MemoryEntry, MemoryStore, ModelPricing,
    OutputLimiterConfig, Provider, ProviderRateLimit, RateLimiter, Role, StreamAccumulator,
    ToolCall, ToolOutputLimiter,
};
use std::collections::HashMap;
use std::sync::Arc;

// ── 1. Serde roundtrip: AiTokenUsage ────────────────────────────────────

#[test]
fn test_token_usage_serde_roundtrip() {
    let usage = AiTokenUsage {
        model: "gpt-4o".into(),
        provider: Provider::OpenAI,
        input_tokens: 1500,
        output_tokens: 800,
        cost_usd: 0.0195,
        timestamp: 1700000000,
    };
    let json = serde_json::to_string(&usage).unwrap();
    let parsed: AiTokenUsage = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.model, "gpt-4o");
    assert_eq!(parsed.provider, Provider::OpenAI);
    assert_eq!(parsed.input_tokens, 1500);
    assert_eq!(parsed.output_tokens, 800);
    assert!((parsed.cost_usd - 0.0195).abs() < 1e-9);
    assert_eq!(parsed.timestamp, 1700000000);
}

// ── 2. Serde roundtrip: AnalyticsReport (full JSON roundtrip) ─────────

#[test]
fn test_analytics_report_json_roundtrip() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    tracker.record_usage("gpt-4o", 1000, 500).unwrap();
    tracker.record_usage("claude-3-haiku", 2000, 1000).unwrap();

    let analytics = Analytics::new(tracker);
    let json_str = analytics.report_json().unwrap();

    // Deserialize back into AnalyticsReport
    let report: AnalyticsReport = serde_json::from_str(&json_str).unwrap();
    assert_eq!(report.summary.total_requests, 2);
    assert_eq!(report.summary.total_input_tokens, 3000);
    assert_eq!(report.summary.total_output_tokens, 1500);
    assert_eq!(report.summary.total_tokens, 4500);
    assert!(report.summary.total_cost_usd > 0.0);
    assert!(report.generated_at > 0);
    assert_eq!(report.by_provider.len(), 2);
    assert_eq!(report.by_model.len(), 2);
}

// ── 3. Provider serde roundtrip for all variants ──────────────────────

#[test]
fn test_provider_serde_all_variants() {
    let providers = [
        Provider::OpenAI,
        Provider::Anthropic,
        Provider::Ollama,
        Provider::DeepSeek,
    ];
    for provider in &providers {
        let json = serde_json::to_string(provider).unwrap();
        let parsed: Provider = serde_json::from_str(&json).unwrap();
        assert_eq!(*provider, parsed);
    }
}

// ── 4. MemoryEntry score field skipped in serialization ───────────────

#[test]
fn test_memory_entry_score_skipped_in_serde() {
    let store = MemoryStore::new();
    store.store("a1", "weather", "sunny in Dubai").unwrap();
    store
        .store("a1", "traffic", "heavy traffic in Dubai")
        .unwrap();

    // Search produces entries with non-zero scores
    let results = store.search("a1", "Dubai", 5).unwrap();
    assert!(!results.is_empty());
    assert!(results[0].score > 0.0);

    // Serialize and deserialize — score should reset to default (0.0)
    let json = serde_json::to_string(&results[0]).unwrap();
    let parsed: MemoryEntry = serde_json::from_str(&json).unwrap();
    assert!((parsed.score - 0.0).abs() < f32::EPSILON);
    // But other fields survive
    assert_eq!(parsed.agent_id, "a1");
    assert!(!parsed.content.is_empty());
}

// ── 5. Thread-safe CostTracker from multiple threads ──────────────────

#[test]
fn test_cost_tracker_thread_safety() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 1000.0,
        alert_thresholds: vec![],
    });

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let t = tracker.clone();
            std::thread::spawn(move || {
                t.record_usage("gpt-4o", 100, 50).unwrap();
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(tracker.request_count(), 10);
    assert_eq!(tracker.total_tokens(), 10 * 150);
}

// ── 6. Thread-safe RateLimiter from multiple threads ──────────────────

#[test]
fn test_rate_limiter_thread_safety() {
    let mut limits = HashMap::new();
    limits.insert(Provider::OpenAI, ProviderRateLimit { rpm: 0, tpm: 0 }); // unlimited
    let limiter = RateLimiter::with_limits(limits);

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let l = limiter.clone();
            std::thread::spawn(move || {
                l.acquire(Provider::OpenAI, 100).unwrap();
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(limiter.current_rpm(Provider::OpenAI), 10);
    assert_eq!(limiter.current_tpm(Provider::OpenAI), 1000);
}

// ── 7. MemoryStore with_backend custom backend ────────────────────────

#[test]
fn test_memory_store_with_custom_backend() {
    let backend = Arc::new(InMemoryBackend::new());

    // Store via the backend directly
    let entry = MemoryEntry {
        id: "custom-id-1".into(),
        agent_id: "agent-x".into(),
        key: "direct".into(),
        content: "stored via backend".into(),
        metadata: HashMap::new(),
        created_at: 1000,
        accessed_at: 1000,
        score: 0.0,
    };
    backend.store(entry).unwrap();

    // Access via MemoryStore wrapping the same backend
    let store = MemoryStore::with_backend(backend.clone());
    let recalled = store.recall("agent-x", "direct").unwrap().unwrap();
    assert_eq!(recalled.content, "stored via backend");

    // Store via MemoryStore, retrieve via backend
    store.store("agent-x", "second", "via store").unwrap();
    let list = backend.list_agent("agent-x").unwrap();
    assert_eq!(list.len(), 2);
}

// ── 8. Context window exact boundary — no truncation at limit ─────────

#[test]
fn test_context_window_exact_boundary() {
    // 8 chars = 2 tokens under heuristic (8/4 = 2)
    let text = "abcdefgh"; // exactly 2 tokens
    assert_eq!(estimate_tokens(text), 2);

    let mut window = ContextWindow::new(2);
    let entry = window.add_output("tool", text).unwrap();
    assert!(!entry.was_truncated);
    assert_eq!(entry.content, text);
    assert_eq!(window.remaining_tokens(), 0);
    assert!(window.is_full());
}

// ── 9. FallbackChain execute tracks all attempts correctly ────────────

#[test]
fn test_fallback_chain_attempt_tracking() {
    let mut chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-sonnet")
        .add(Provider::DeepSeek, "deepseek-chat")
        .build();

    let mut call_count = 0;
    let result = chain
        .execute(|provider, _model| {
            call_count += 1;
            if provider == Provider::DeepSeek {
                Ok(42)
            } else {
                Err(format!("{} failed", provider))
            }
        })
        .unwrap();

    assert_eq!(result.value, 42);
    assert_eq!(result.provider, Provider::DeepSeek);
    assert_eq!(result.model, "deepseek-chat");
    assert_eq!(result.attempts.len(), 3);
    assert!(!result.attempts[0].success);
    assert!(!result.attempts[1].success);
    assert!(result.attempts[2].success);
    assert_eq!(result.attempts[0].error.as_deref(), Some("OpenAI failed"));
    assert_eq!(call_count, 3);
}

// ── 10. CostTracker calculate_cost arithmetic for all default models ──

#[test]
fn test_cost_calculation_all_default_models() {
    let tracker = CostTracker::with_defaults();
    let pricing_table = default_pricing();

    for (model_name, pricing) in &pricing_table {
        let cost = tracker.calculate_cost(model_name, 1000, 1000).unwrap();
        let expected = pricing.input_cost_per_1k + pricing.output_cost_per_1k;
        assert!(
            (cost - expected).abs() < 1e-9,
            "Cost mismatch for model {}: got {}, expected {}",
            model_name,
            cost,
            expected
        );
    }
}

// ── 11. Conversation save/load preserves tool calls in messages ───────

#[test]
fn test_conversation_save_load_with_tool_calls() {
    let dir = std::env::temp_dir();
    let path = dir.join("hudhud_conv_tool_calls_test.json");

    let mut conv = Conversation::new("claude-3-sonnet", 8192);
    conv.add_user("Search for weather");
    conv.add_assistant_with_tools(
        "Calling tool",
        vec![ToolCall {
            id: "tc_1".into(),
            name: "weather_api".into(),
            arguments: r#"{"city":"Istanbul"}"#.into(),
        }],
    );
    conv.add_tool_result("tc_1", r#"{"temp":22}"#);
    conv.add_assistant("It is 22 degrees in Istanbul.");

    conv.save(&path).unwrap();
    let loaded = Conversation::load(&path).unwrap();

    assert_eq!(loaded.message_count(), 4);
    assert_eq!(loaded.model(), "claude-3-sonnet");

    // Verify the tool call message survived the roundtrip
    let api = loaded.messages_for_api();
    assert!(api[1]["tool_calls"].is_array());
    assert_eq!(api[1]["tool_calls"][0]["function"]["name"], "weather_api");
    assert_eq!(api[2]["tool_call_id"], "tc_1");

    let _ = std::fs::remove_file(&path);
}

// ── 12. StreamAccumulator finish with tool calls produces correct role ─

#[test]
fn test_stream_accumulator_finish_with_multiple_tools() {
    let mut acc = StreamAccumulator::new();
    acc.push_content("I need two tools.");
    acc.push_tool_call(ToolCall {
        id: "t1".into(),
        name: "search".into(),
        arguments: "{}".into(),
    });
    acc.push_tool_call(ToolCall {
        id: "t2".into(),
        name: "calculate".into(),
        arguments: r#"{"x":1}"#.into(),
    });

    assert_eq!(acc.tool_calls().len(), 2);

    let msg = acc.finish();
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content, "I need two tools.");
    let calls = msg.tool_calls.unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].name, "search");
    assert_eq!(calls[1].name, "calculate");
    assert!(acc.is_finished());
}

// ── 13. ToolOutputLimiter preserves short output exactly ──────────────

#[test]
fn test_tool_output_limiter_preserves_short_output_exactly() {
    let config = OutputLimiterConfig {
        max_tokens: 100,
        truncation_suffix: "[CUT]".into(),
        warn_on_truncation: false,
    };
    let limiter = ToolOutputLimiter::new(config);

    // "Hello!" = 6 chars = max(6/4, 1) = 1 token, well within 100
    let output = "Hello!";
    let result = limiter.limit("tool", output);
    assert_eq!(result, output); // must be byte-identical, not just equal-ish

    let (result2, truncated) = limiter.limit_with_info("tool", output);
    assert_eq!(result2, output);
    assert!(!truncated);
}

// ── 14. Analytics summary averages are mathematically correct ─────────

#[test]
fn test_analytics_summary_averages() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 1000.0,
        alert_thresholds: vec![],
    });
    // 5 identical requests
    for _ in 0..5 {
        tracker.record_usage("gpt-4o", 200, 100).unwrap();
    }

    let analytics = Analytics::new(tracker);
    let summary = analytics.summary();

    assert_eq!(summary.total_requests, 5);
    assert_eq!(summary.total_input_tokens, 1000);
    assert_eq!(summary.total_output_tokens, 500);
    assert_eq!(summary.total_tokens, 1500);

    // avg_tokens_per_request = 1500 / 5 = 300
    assert!((summary.avg_tokens_per_request - 300.0).abs() < 1e-9);

    // avg_cost_per_request = total_cost / 5
    let expected_avg_cost = summary.total_cost_usd / 5.0;
    assert!((summary.avg_cost_per_request - expected_avg_cost).abs() < 1e-9);
}

// ── 15. BudgetConfig and ModelPricing serde roundtrip ─────────────────

#[test]
fn test_budget_config_and_model_pricing_serde() {
    let budget = BudgetConfig {
        hard_limit_usd: 42.5,
        alert_thresholds: vec![0.25, 0.75, 0.9],
    };
    let json = serde_json::to_string(&budget).unwrap();
    let parsed: BudgetConfig = serde_json::from_str(&json).unwrap();
    assert!((parsed.hard_limit_usd - 42.5).abs() < 1e-9);
    assert_eq!(parsed.alert_thresholds, vec![0.25, 0.75, 0.9]);

    let pricing = ModelPricing {
        provider: Provider::DeepSeek,
        model: "deepseek-coder".into(),
        input_cost_per_1k: 0.00014,
        output_cost_per_1k: 0.00028,
    };
    let json = serde_json::to_string(&pricing).unwrap();
    let parsed: ModelPricing = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.provider, Provider::DeepSeek);
    assert_eq!(parsed.model, "deepseek-coder");
    assert!((parsed.input_cost_per_1k - 0.00014).abs() < 1e-9);
}
