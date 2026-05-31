use hudhudscript_tools_ai::*;
use std::collections::HashMap;

// ── MemoryStore tests ──────────────────────────────────────────────────

#[test]
fn test_memory_store_new() {
    let store = MemoryStore::new();
    assert!(store.list("any").unwrap().is_empty());
}

#[test]
fn test_memory_store_default() {
    let store = MemoryStore::default();
    assert!(store.list("any").unwrap().is_empty());
}

#[test]
fn test_store_and_recall() {
    let store = MemoryStore::new();
    let id = store.store("agent-1", "user_name", "Alice").unwrap();
    assert!(!id.is_empty());
    let entry = store.recall("agent-1", "user_name").unwrap().unwrap();
    assert_eq!(entry.content, "Alice");
    assert_eq!(entry.key, "user_name");
    assert_eq!(entry.agent_id, "agent-1");
}

#[test]
fn test_overwrite_same_key() {
    let store = MemoryStore::new();
    store.store("a1", "topic", "first").unwrap();
    store.store("a1", "topic", "second").unwrap();
    let all = store.list("a1").unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].content, "second");
}

#[test]
fn test_recall_nonexistent_key() {
    let store = MemoryStore::new();
    assert!(store.recall("a1", "nope").unwrap().is_none());
}

#[test]
fn test_recall_by_id() {
    let store = MemoryStore::new();
    let id = store.store("a1", "topic", "data").unwrap();
    let entry = store.recall_by_id(&id).unwrap().unwrap();
    assert_eq!(entry.content, "data");
}

#[test]
fn test_recall_by_id_nonexistent() {
    let store = MemoryStore::new();
    assert!(store.recall_by_id("bad-uuid").unwrap().is_none());
}

#[test]
fn test_search() {
    let store = MemoryStore::new();
    store
        .store("a1", "weather", "It is sunny in Dubai")
        .unwrap();
    store.store("a1", "traffic", "Light traffic").unwrap();
    let results = store.search("a1", "Dubai sunny", 5).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].key, "weather");
}

#[test]
fn test_search_no_results() {
    let store = MemoryStore::new();
    store.store("a1", "topic", "alpha beta").unwrap();
    let results = store.search("a1", "zzzzz", 5).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_search_limit() {
    let store = MemoryStore::new();
    for i in 0..10 {
        store
            .store(
                "a1",
                &format!("key_{}", i),
                &format!("data about rust programming {}", i),
            )
            .unwrap();
    }
    let results = store.search("a1", "rust programming", 3).unwrap();
    assert!(results.len() <= 3);
}

#[test]
fn test_forget() {
    let store = MemoryStore::new();
    let id = store.store("a1", "tmp", "temp").unwrap();
    assert!(store.forget(&id).unwrap());
    assert!(store.recall("a1", "tmp").unwrap().is_none());
}

#[test]
fn test_forget_nonexistent() {
    let store = MemoryStore::new();
    assert!(!store.forget("bad-id").unwrap());
}

#[test]
fn test_clear_agent() {
    let store = MemoryStore::new();
    store.store("a1", "x", "data").unwrap();
    store.store("a1", "y", "data").unwrap();
    store.store("a2", "z", "other").unwrap();
    let deleted = store.clear("a1").unwrap();
    assert_eq!(deleted, 2);
    assert_eq!(store.list("a1").unwrap().len(), 0);
    assert_eq!(store.list("a2").unwrap().len(), 1);
}

#[test]
fn test_store_with_metadata() {
    let store = MemoryStore::new();
    let mut meta = HashMap::new();
    meta.insert("tag".to_string(), serde_json::json!("important"));
    let id = store
        .store_with_metadata("a1", "topic", "content", meta)
        .unwrap();
    assert!(!id.is_empty());
    let entry = store.recall("a1", "topic").unwrap().unwrap();
    assert_eq!(entry.metadata["tag"], serde_json::json!("important"));
}

#[test]
fn test_list_returns_all_keys_for_agent() {
    let store = MemoryStore::new();
    store.store("a1", "k1", "v1").unwrap();
    store.store("a1", "k2", "v2").unwrap();
    store.store("a1", "k3", "v3").unwrap();
    let entries = store.list("a1").unwrap();
    assert_eq!(entries.len(), 3);
}

#[test]
fn test_list_different_agents_isolated() {
    let store = MemoryStore::new();
    store.store("a1", "key", "v1").unwrap();
    store.store("a2", "key", "v2").unwrap();
    assert_eq!(store.list("a1").unwrap().len(), 1);
    assert_eq!(store.list("a2").unwrap().len(), 1);
    assert_eq!(store.recall("a1", "key").unwrap().unwrap().content, "v1");
    assert_eq!(store.recall("a2", "key").unwrap().unwrap().content, "v2");
}

#[test]
fn test_clear_nonexistent_agent() {
    let store = MemoryStore::new();
    let deleted = store.clear("no-agent").unwrap();
    assert_eq!(deleted, 0);
}

// ── ContextWindow tests ────────────────────────────────────────────────

#[test]
fn test_context_window_new() {
    let window = ContextWindow::new(100);
    assert_eq!(window.remaining_tokens(), 100);
    assert_eq!(window.used_tokens(), 0);
    assert!(!window.is_full());
}

#[test]
fn test_context_window_add_output() {
    let mut window = ContextWindow::new(1000);
    let entry = window.add_output("tool_x", "Hello World").unwrap();
    assert_eq!(entry.tool_name, "tool_x");
    assert_eq!(entry.content, "Hello World");
    assert!(!entry.was_truncated);
}

#[test]
fn test_context_window_full_drops() {
    let mut window = ContextWindow::new(1);
    window.add_output("t1", &"a".repeat(100));
    assert!(window.is_full());
    assert!(window.add_output("t2", "more").is_none());
}

#[test]
fn test_context_window_used_tokens_increases() {
    let mut window = ContextWindow::new(10000);
    let before = window.used_tokens();
    window.add_output("t1", "Some data here");
    assert!(window.used_tokens() > before);
}

#[test]
fn test_estimate_tokens_empty() {
    assert_eq!(estimate_tokens(""), 0);
}

#[test]
fn test_estimate_tokens_values() {
    assert_eq!(estimate_tokens("a"), 1);
    assert_eq!(estimate_tokens("abcdefgh"), 2);
    assert_eq!(estimate_tokens(&"a".repeat(40)), 10);
}

#[test]
fn test_estimate_tokens_long_text() {
    let tokens = estimate_tokens(&"hello world ".repeat(100));
    assert!(tokens > 0);
}

// ── CostTracker tests ──────────────────────────────────────────────────

#[test]
fn test_cost_tracker_with_defaults() {
    let tracker = CostTracker::with_defaults();
    assert_eq!(tracker.request_count(), 0);
    assert!((tracker.total_cost() - 0.0).abs() < 1e-9);
}

#[test]
fn test_calculate_cost_gpt4o() {
    let tracker = CostTracker::with_defaults();
    let cost = tracker.calculate_cost("gpt-4o", 1000, 500).unwrap();
    assert!((cost - 0.0125).abs() < 1e-9);
}

#[test]
fn test_calculate_cost_ollama_free() {
    let tracker = CostTracker::with_defaults();
    let cost = tracker.calculate_cost("ollama-local", 10000, 5000).unwrap();
    assert!((cost - 0.0).abs() < 1e-9);
}

#[test]
fn test_calculate_cost_unknown_model() {
    let tracker = CostTracker::with_defaults();
    assert!(tracker.calculate_cost("no-such-model", 1, 1).is_err());
}

#[test]
fn test_record_usage_accumulates() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    tracker.record_usage("gpt-4o", 1000, 500).unwrap();
    tracker.record_usage("gpt-4o", 1000, 500).unwrap();
    assert_eq!(tracker.request_count(), 2);
    assert!((tracker.total_cost() - 0.025).abs() < 1e-9);
    assert_eq!(tracker.total_tokens(), 3000);
}

#[test]
fn test_budget_exceeded() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 0.001,
        alert_thresholds: vec![],
    });
    let result = tracker.record_usage("gpt-4o", 10000, 10000);
    assert!(matches!(
        result.unwrap_err(),
        CostError::BudgetExceeded { .. }
    ));
}

#[test]
fn test_remaining_budget() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 1.0,
        alert_thresholds: vec![],
    });
    assert!((tracker.remaining_budget() - 1.0).abs() < 1e-9);
}

#[test]
fn test_reset() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    tracker.record_usage("gpt-4o", 1000, 500).unwrap();
    tracker.reset();
    assert_eq!(tracker.request_count(), 0);
    assert!((tracker.total_cost() - 0.0).abs() < 1e-9);
}

#[test]
fn test_cost_by_provider() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    tracker.record_usage("gpt-4o", 1000, 0).unwrap();
    tracker.record_usage("claude-3-haiku", 1000, 0).unwrap();
    let by_provider = tracker.cost_by_provider();
    assert!(by_provider.contains_key(&Provider::OpenAI));
    assert!(by_provider.contains_key(&Provider::Anthropic));
}

#[test]
fn test_cost_by_model() {
    let tracker = CostTracker::new(BudgetConfig {
        hard_limit_usd: 100.0,
        alert_thresholds: vec![],
    });
    tracker.record_usage("gpt-4o", 1000, 0).unwrap();
    tracker.record_usage("claude-3-haiku", 1000, 0).unwrap();
    let by_model = tracker.cost_by_model();
    assert_eq!(by_model.len(), 2);
    assert!(by_model.contains_key("gpt-4o"));
    assert!(by_model.contains_key("claude-3-haiku"));
}

// ── FallbackChain tests ────────────────────────────────────────────────

#[test]
fn test_fallback_chain_creation() {
    let chain = FallbackChain::new(vec![
        FallbackEntry::new(Provider::OpenAI, "gpt-4o"),
        FallbackEntry::new(Provider::Anthropic, "claude-3-haiku"),
    ]);
    assert_eq!(chain.len(), 2);
    assert!(!chain.is_empty());
}

#[test]
fn test_fallback_chain_empty() {
    let chain = FallbackChain::empty();
    assert_eq!(chain.len(), 0);
    assert!(chain.is_empty());
}

#[test]
fn test_fallback_chain_push() {
    let mut chain = FallbackChain::empty();
    chain.push(FallbackEntry::new(Provider::OpenAI, "gpt-4o"));
    assert_eq!(chain.len(), 1);
}

#[test]
fn test_fallback_chain_entries() {
    let chain = FallbackChain::new(vec![FallbackEntry::new(Provider::OpenAI, "gpt-4o")]);
    let entries = chain.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].model, "gpt-4o");
    assert!(entries[0].enabled);
}

#[test]
fn test_fallback_chain_disable_enable() {
    let mut chain = FallbackChain::new(vec![
        FallbackEntry::new(Provider::OpenAI, "gpt-4o"),
        FallbackEntry::new(Provider::Anthropic, "claude-3-haiku"),
    ]);
    chain.disable(0);
    assert!(!chain.entries()[0].enabled);
    chain.enable(0);
    assert!(chain.entries()[0].enabled);
}

#[test]
fn test_fallback_chain_reset() {
    let mut chain = FallbackChain::new(vec![FallbackEntry::new(Provider::OpenAI, "gpt-4o")]);
    chain.disable(0);
    chain.reset();
    assert!(chain.entries()[0].enabled);
}

#[test]
fn test_fallback_chain_execute_success() {
    let mut chain = FallbackChain::new(vec![FallbackEntry::new(Provider::OpenAI, "gpt-4o")]);
    let result =
        chain.execute(|_provider, _model| -> Result<String, String> { Ok("response".to_string()) });
    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.value, "response");
    assert_eq!(r.provider, Provider::OpenAI);
    assert_eq!(r.model, "gpt-4o");
}

#[test]
fn test_fallback_chain_execute_all_fail() {
    let mut chain = FallbackChain::new(vec![FallbackEntry::new(Provider::OpenAI, "gpt-4o")]);
    let result =
        chain.execute(|_provider, _model| -> Result<String, String> { Err("fail".to_string()) });
    assert!(result.is_err());
}

#[test]
fn test_fallback_chain_builder() {
    let chain = FallbackChainBuilder::new()
        .add(Provider::OpenAI, "gpt-4o")
        .add(Provider::Anthropic, "claude-3-haiku")
        .max_consecutive_failures(5)
        .build();
    assert_eq!(chain.len(), 2);
    assert_eq!(chain.max_consecutive_failures, 5);
}

// ── RateLimiter tests ──────────────────────────────────────────────────

#[test]
fn test_rate_limiter_new() {
    let limiter = RateLimiter::new();
    // Should not error for an unknown provider with small request
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 0);
}

#[test]
fn test_rate_limiter_record_and_check() {
    let limiter = RateLimiter::new();
    limiter.record(Provider::OpenAI, 100);
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 1);
    assert!(limiter.current_tpm(Provider::OpenAI) >= 100);
}

#[test]
fn test_rate_limiter_acquire() {
    let limiter = RateLimiter::new();
    let result = limiter.acquire(Provider::OpenAI, 100);
    assert!(result.is_ok());
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 1);
}

#[test]
fn test_rate_limiter_reset() {
    let limiter = RateLimiter::new();
    limiter.record(Provider::OpenAI, 100);
    limiter.reset();
    assert_eq!(limiter.current_rpm(Provider::OpenAI), 0);
}

// ── Conversation tests ─────────────────────────────────────────────────

#[test]
fn test_conversation_new() {
    let conv = Conversation::new("gpt-4o", 4096);
    assert_eq!(conv.model(), "gpt-4o");
    assert_eq!(conv.max_tokens(), 4096);
    assert_eq!(conv.message_count(), 0);
    assert!(conv.system_prompt().is_none());
}

#[test]
fn test_conversation_with_system_prompt() {
    let conv = Conversation::with_system_prompt("gpt-4o", 4096, "You are helpful.");
    assert_eq!(conv.system_prompt(), Some("You are helpful."));
}

#[test]
fn test_conversation_add_messages() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_user("Hello");
    conv.add_assistant("Hi there!");
    assert_eq!(conv.message_count(), 2);
    let last = conv.last_message().unwrap();
    assert_eq!(last.role, Role::Assistant);
    assert_eq!(last.content, "Hi there!");
}

#[test]
fn test_conversation_add_system() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_system("System instruction");
    assert_eq!(conv.message_count(), 1);
    assert_eq!(conv.messages()[0].role, Role::System);
}

#[test]
fn test_conversation_add_tool_result() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_tool_result("call_123", "result data");
    let msg = conv.last_message().unwrap();
    assert_eq!(msg.role, Role::Tool);
    assert_eq!(msg.tool_call_id, Some("call_123".to_string()));
}

#[test]
fn test_conversation_total_tokens() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_user("Hello world");
    assert!(conv.total_tokens() > 0);
}

#[test]
fn test_conversation_clear() {
    let mut conv = Conversation::new("gpt-4o", 4096);
    conv.add_user("Hello");
    conv.add_assistant("Hi");
    conv.clear();
    assert_eq!(conv.message_count(), 0);
}

#[test]
fn test_message_new() {
    let msg = Message::new(Role::User, "hello");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content, "hello");
    assert!(msg.tool_call_id.is_none());
    assert!(msg.tool_calls.is_none());
}

#[test]
fn test_message_tool_result() {
    let msg = Message::tool_result("call1", "result");
    assert_eq!(msg.role, Role::Tool);
    assert_eq!(msg.tool_call_id, Some("call1".to_string()));
}

#[test]
fn test_message_assistant_with_tools() {
    let tool = ToolCall {
        id: "tc1".to_string(),
        name: "search".to_string(),
        arguments: r#"{"q":"rust"}"#.to_string(),
    };
    let msg = Message::assistant_with_tools("Let me search", vec![tool]);
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.tool_calls.as_ref().unwrap().len(), 1);
}

#[test]
fn test_message_estimated_tokens() {
    let msg = Message::new(Role::User, "Hello world");
    let tokens = msg.estimated_tokens();
    assert!(tokens >= 4); // at least role overhead
}

#[test]
fn test_role_display() {
    assert_eq!(format!("{}", Role::User), "user");
    assert_eq!(format!("{}", Role::Assistant), "assistant");
    assert_eq!(format!("{}", Role::System), "system");
    assert_eq!(format!("{}", Role::Tool), "tool");
}

// ── StreamAccumulator tests ────────────────────────────────────────────

#[test]
fn test_stream_accumulator_new() {
    let acc = StreamAccumulator::new();
    assert_eq!(acc.content(), "");
    assert!(acc.tool_calls().is_empty());
    assert!(!acc.is_finished());
}

#[test]
fn test_stream_accumulator_push() {
    let mut acc = StreamAccumulator::new();
    acc.push_content("Hello ");
    acc.push_content("world");
    assert_eq!(acc.content(), "Hello world");
}

#[test]
fn test_stream_accumulator_finish() {
    let mut acc = StreamAccumulator::new();
    acc.push_content("done");
    let msg = acc.finish();
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content, "done");
    assert!(acc.is_finished());
}

#[test]
fn test_stream_accumulator_reset() {
    let mut acc = StreamAccumulator::new();
    acc.push_content("data");
    acc.finish();
    acc.reset();
    assert_eq!(acc.content(), "");
    assert!(!acc.is_finished());
}

#[test]
fn test_stream_accumulator_token_count() {
    let mut acc = StreamAccumulator::new();
    acc.push_content("a");
    acc.push_content("b");
    assert_eq!(acc.token_count(), 2);
}
