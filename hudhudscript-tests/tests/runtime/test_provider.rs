use hudhudscript_runtime::provider::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

// ===============================================================================
// ProviderType
// ===============================================================================

#[test]
fn provider_type_openai_eq() {
    assert_eq!(ProviderType::OpenAI, ProviderType::OpenAI);
}

#[test]
fn provider_type_anthropic_eq() {
    assert_eq!(ProviderType::Anthropic, ProviderType::Anthropic);
}

#[test]
fn provider_type_ollama_eq() {
    assert_eq!(ProviderType::Ollama, ProviderType::Ollama);
}

#[test]
fn provider_type_deepseek_eq() {
    assert_eq!(ProviderType::DeepSeek, ProviderType::DeepSeek);
}

#[test]
fn provider_type_gemini_eq() {
    assert_eq!(ProviderType::Gemini, ProviderType::Gemini);
}

#[test]
fn provider_type_mistral_eq() {
    assert_eq!(ProviderType::Mistral, ProviderType::Mistral);
}

#[test]
fn provider_type_groq_eq() {
    assert_eq!(ProviderType::Groq, ProviderType::Groq);
}

#[test]
fn provider_type_cohere_eq() {
    assert_eq!(ProviderType::Cohere, ProviderType::Cohere);
}

#[test]
fn provider_type_together_eq() {
    assert_eq!(ProviderType::Together, ProviderType::Together);
}

#[test]
fn provider_type_xai_eq() {
    assert_eq!(ProviderType::XAI, ProviderType::XAI);
}

#[test]
fn provider_type_openrouter_eq() {
    assert_eq!(ProviderType::OpenRouter, ProviderType::OpenRouter);
}

#[test]
fn provider_type_http_eq() {
    assert_eq!(ProviderType::Http, ProviderType::Http);
}

#[test]
fn provider_type_ne_across_variants() {
    assert_ne!(ProviderType::OpenAI, ProviderType::Anthropic);
    assert_ne!(ProviderType::Ollama, ProviderType::Gemini);
    assert_ne!(ProviderType::Http, ProviderType::Groq);
}

#[test]
fn provider_type_clone() {
    let pt = ProviderType::DeepSeek;
    let cloned = pt.clone();
    assert_eq!(pt, cloned);
}

#[test]
fn provider_type_debug() {
    let dbg = format!("{:?}", ProviderType::OpenAI);
    assert!(dbg.contains("OpenAI"));
}

#[test]
fn provider_type_serde_roundtrip() {
    let pt = ProviderType::Anthropic;
    let json = serde_json::to_string(&pt).unwrap();
    let deserialized: ProviderType = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, deserialized);
}

#[test]
fn provider_type_all_variants_serde() {
    let variants = vec![
        ProviderType::OpenAI,
        ProviderType::Anthropic,
        ProviderType::Ollama,
        ProviderType::DeepSeek,
        ProviderType::Gemini,
        ProviderType::Mistral,
        ProviderType::Groq,
        ProviderType::Cohere,
        ProviderType::Together,
        ProviderType::XAI,
        ProviderType::OpenRouter,
        ProviderType::Http,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).unwrap();
        let back: ProviderType = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
    }
}

// ===============================================================================
// TokenBudget
// ===============================================================================

#[test]
fn token_budget_default_max_tokens_per_call() {
    let b = TokenBudget::default();
    assert_eq!(b.max_tokens_per_call, 4000);
}

#[test]
fn token_budget_default_max_tokens_per_day() {
    let b = TokenBudget::default();
    assert_eq!(b.max_tokens_per_day, 100000);
}

#[test]
fn token_budget_default_alert_threshold() {
    let b = TokenBudget::default();
    assert!((b.alert_threshold - 0.8).abs() < f64::EPSILON);
}

#[test]
fn token_budget_custom_values() {
    let b = TokenBudget {
        max_tokens_per_call: 8000,
        max_tokens_per_day: 200000,
        alert_threshold: 0.9,
    };
    assert_eq!(b.max_tokens_per_call, 8000);
    assert_eq!(b.max_tokens_per_day, 200000);
    assert!((b.alert_threshold - 0.9).abs() < f64::EPSILON);
}

#[test]
fn token_budget_serde_roundtrip() {
    let b = TokenBudget::default();
    let json = serde_json::to_string(&b).unwrap();
    let back: TokenBudget = serde_json::from_str(&json).unwrap();
    assert_eq!(back.max_tokens_per_call, b.max_tokens_per_call);
    assert_eq!(back.max_tokens_per_day, b.max_tokens_per_day);
}

// ===============================================================================
// ProviderConfig
// ===============================================================================

#[test]
fn provider_config_minimal() {
    let cfg = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: None,
        endpoint: None,
        temperature: None,
        max_tokens: None,
        budget: None,
        timeout_secs: None,
        extra: HashMap::new(),
    };
    assert_eq!(cfg.model, "gpt-4");
    assert!(cfg.api_key.is_none());
}

#[test]
fn provider_config_with_all_fields() {
    let cfg = ProviderConfig {
        provider_type: ProviderType::Anthropic,
        model: "claude-3-opus".to_string(),
        api_key: Some("sk-test".to_string()),
        endpoint: Some("https://api.anthropic.com".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(4096),
        budget: Some(TokenBudget::default()),
        timeout_secs: None,
        extra: {
            let mut m = HashMap::new();
            m.insert("version".to_string(), json!("2024-01-01"));
            m
        },
    };
    assert_eq!(cfg.api_key.as_deref(), Some("sk-test"));
    assert_eq!(cfg.max_tokens, Some(4096));
    assert!(cfg.budget.is_some());
    assert!(cfg.extra.contains_key("version"));
}

#[test]
fn provider_config_serde_roundtrip() {
    let cfg = ProviderConfig {
        provider_type: ProviderType::Ollama,
        model: "llama3".to_string(),
        api_key: None,
        endpoint: Some("http://localhost:11434".to_string()),
        temperature: Some(0.5),
        max_tokens: Some(2048),
        budget: None,
        timeout_secs: None,
        extra: HashMap::new(),
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let back: ProviderConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.model, "llama3");
    assert_eq!(back.provider_type, ProviderType::Ollama);
}

#[test]
fn provider_config_clone() {
    let cfg = ProviderConfig {
        provider_type: ProviderType::Gemini,
        model: "gemini-pro".to_string(),
        api_key: Some("key".to_string()),
        endpoint: None,
        temperature: None,
        max_tokens: None,
        budget: None,
        timeout_secs: None,
        extra: HashMap::new(),
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.model, cfg.model);
}

// ===============================================================================
// ToolDefinition
// ===============================================================================

#[test]
fn tool_definition_creation() {
    let td = ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get weather info".to_string(),
        parameters: json!({"type": "object", "properties": {"city": {"type": "string"}}}),
    };
    assert_eq!(td.name, "get_weather");
}

#[test]
fn tool_definition_serde_roundtrip() {
    let td = ToolDefinition {
        name: "search".to_string(),
        description: "Search the web".to_string(),
        parameters: json!({"type": "object"}),
    };
    let json = serde_json::to_string(&td).unwrap();
    let back: ToolDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "search");
    assert_eq!(back.description, "Search the web");
}

// ===============================================================================
// ToolCallResult
// ===============================================================================

#[test]
fn tool_call_result_fields() {
    let tcr = ToolCallResult {
        tool_call_id: "call_123".to_string(),
        name: "get_time".to_string(),
        content: "12:00".to_string(),
    };
    assert_eq!(tcr.tool_call_id, "call_123");
    assert_eq!(tcr.content, "12:00");
}

#[test]
fn tool_call_result_serde_roundtrip() {
    let tcr = ToolCallResult {
        tool_call_id: "id1".to_string(),
        name: "fn1".to_string(),
        content: "result".to_string(),
    };
    let json = serde_json::to_string(&tcr).unwrap();
    let back: ToolCallResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool_call_id, "id1");
}

// ===============================================================================
// FunctionCallResultType
// ===============================================================================

#[test]
fn function_call_result_type_default_is_json() {
    let default = FunctionCallResultType::default();
    assert_eq!(default, FunctionCallResultType::Json);
}

#[test]
fn function_call_result_type_variants() {
    assert_eq!(FunctionCallResultType::Json, FunctionCallResultType::Json);
    assert_eq!(FunctionCallResultType::Text, FunctionCallResultType::Text);
    assert_eq!(
        FunctionCallResultType::Binary,
        FunctionCallResultType::Binary
    );
    assert_ne!(FunctionCallResultType::Json, FunctionCallResultType::Text);
}

#[test]
fn function_call_result_type_serde() {
    let types = vec![
        FunctionCallResultType::Json,
        FunctionCallResultType::Text,
        FunctionCallResultType::Binary,
    ];
    for t in types {
        let json = serde_json::to_string(&t).unwrap();
        let back: FunctionCallResultType = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }
}

// ===============================================================================
// FunctionCall
// ===============================================================================

#[test]
fn function_call_creation() {
    let fc = FunctionCall {
        id: "fc1".to_string(),
        name: "read_file".to_string(),
        arguments: json!({"path": "/tmp/test"}),
        result_type: FunctionCallResultType::Text,
    };
    assert_eq!(fc.id, "fc1");
    assert_eq!(fc.name, "read_file");
    assert_eq!(fc.result_type, FunctionCallResultType::Text);
}

#[test]
fn function_call_serde_roundtrip() {
    let fc = FunctionCall {
        id: "fc2".to_string(),
        name: "compute".to_string(),
        arguments: json!({"x": 42}),
        result_type: FunctionCallResultType::Json,
    };
    let json = serde_json::to_string(&fc).unwrap();
    let back: FunctionCall = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "fc2");
    assert_eq!(back.arguments["x"], 42);
}

// ===============================================================================
// FunctionCallResult
// ===============================================================================

#[test]
fn function_call_result_ok() {
    let r = FunctionCallResult::ok("id1", "fn1", json!({"status": "done"}));
    assert!(r.success);
    assert!(r.error.is_none());
    assert_eq!(r.id, "id1");
    assert_eq!(r.name, "fn1");
}

#[test]
fn function_call_result_err() {
    let r = FunctionCallResult::err("id2", "fn2", "something went wrong");
    assert!(!r.success);
    assert_eq!(r.error.as_deref(), Some("something went wrong"));
    assert!(r.output.is_null());
}

#[test]
fn function_call_result_as_tool_call_result_success() {
    let r = FunctionCallResult::ok("id1", "fn1", json!(42));
    let tcr = r.as_tool_call_result();
    assert_eq!(tcr.tool_call_id, "id1");
    assert_eq!(tcr.name, "fn1");
    assert_eq!(tcr.content, "42");
}

#[test]
fn function_call_result_as_tool_call_result_error() {
    let r = FunctionCallResult::err("id2", "fn2", "fail");
    let tcr = r.as_tool_call_result();
    assert!(tcr.content.contains("Error: fail"));
}

#[test]
fn function_call_result_err_no_message_shows_unknown() {
    let r = FunctionCallResult {
        id: "id".to_string(),
        name: "fn".to_string(),
        output: json!(null),
        success: false,
        error: None,
    };
    let tcr = r.as_tool_call_result();
    assert!(tcr.content.contains("unknown"));
}

#[test]
fn function_call_result_serde_roundtrip() {
    let r = FunctionCallResult::ok("id1", "fn1", json!({"key": "val"}));
    let json = serde_json::to_string(&r).unwrap();
    let back: FunctionCallResult = serde_json::from_str(&json).unwrap();
    assert!(back.success);
    assert_eq!(back.output["key"], "val");
}

// ===============================================================================
// LLMRequest
// ===============================================================================

#[test]
fn llm_request_minimal() {
    let req = LLMRequest {
        prompt: "Hello".to_string(),
        system_prompt: None,
        temperature: None,
        max_tokens: None,
        mnemonics: None,
        optimize: false,
        tools: None,
        timeout_secs: None,
    };
    assert_eq!(req.prompt, "Hello");
    assert!(!req.optimize);
}

#[test]
fn llm_request_full() {
    let req = LLMRequest {
        prompt: "Explain quantum physics".to_string(),
        system_prompt: Some("You are a physicist".to_string()),
        temperature: Some(0.3),
        max_tokens: Some(1000),
        mnemonics: Some({
            let mut m = HashMap::new();
            m.insert("QM".to_string(), "Quantum Mechanics".to_string());
            m
        }),
        optimize: true,
        tools: Some(vec![ToolDefinition {
            name: "calculator".to_string(),
            description: "A calculator".to_string(),
            parameters: json!({"type": "object"}),
        }]),
        timeout_secs: None,
    };
    assert!(req.optimize);
    assert!(req.tools.as_ref().unwrap().len() == 1);
    assert!(req.mnemonics.as_ref().unwrap().contains_key("QM"));
}

#[test]
fn llm_request_serde_roundtrip() {
    let req = LLMRequest {
        prompt: "test".to_string(),
        system_prompt: Some("sys".to_string()),
        temperature: Some(0.5),
        max_tokens: Some(500),
        mnemonics: None,
        optimize: false,
        tools: None,
        timeout_secs: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: LLMRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.prompt, "test");
    assert_eq!(back.system_prompt.as_deref(), Some("sys"));
}

// ===============================================================================
// LLMToolCall
// ===============================================================================

#[test]
fn llm_tool_call_fields() {
    let tc = LLMToolCall {
        id: "tc1".to_string(),
        name: "search".to_string(),
        arguments: json!({"query": "rust"}),
    };
    assert_eq!(tc.id, "tc1");
    assert_eq!(tc.arguments["query"], "rust");
}

// ===============================================================================
// LLMResponse
// ===============================================================================

#[test]
fn llm_response_without_tool_calls() {
    let resp = LLMResponse {
        content: "Hello world".to_string(),
        tokens_used: TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        },
        model: "gpt-4".to_string(),
        finish_reason: "stop".to_string(),
        tool_calls: None,
    };
    assert_eq!(resp.content, "Hello world");
    assert!(resp.tool_calls.is_none());
}

#[test]
fn llm_response_with_tool_calls() {
    let resp = LLMResponse {
        content: "".to_string(),
        tokens_used: TokenUsage {
            prompt_tokens: 20,
            completion_tokens: 10,
            total_tokens: 30,
        },
        model: "gpt-4o".to_string(),
        finish_reason: "tool_calls".to_string(),
        tool_calls: Some(vec![LLMToolCall {
            id: "tc1".to_string(),
            name: "get_time".to_string(),
            arguments: json!({}),
        }]),
    };
    assert_eq!(resp.finish_reason, "tool_calls");
    assert_eq!(resp.tool_calls.as_ref().unwrap().len(), 1);
}

// ===============================================================================
// TokenUsage
// ===============================================================================

#[test]
fn token_usage_fields() {
    let u = TokenUsage {
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
    };
    assert_eq!(u.prompt_tokens, 100);
    assert_eq!(u.completion_tokens, 50);
    assert_eq!(u.total_tokens, 150);
}

#[test]
fn token_usage_serde() {
    let u = TokenUsage {
        prompt_tokens: 1,
        completion_tokens: 2,
        total_tokens: 3,
    };
    let json = serde_json::to_string(&u).unwrap();
    let back: TokenUsage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.total_tokens, 3);
}

// ===============================================================================
// ProviderInfo
// ===============================================================================

#[test]
fn provider_info_fields() {
    let info = ProviderInfo {
        name: "my-provider".to_string(),
        model: "gpt-4".to_string(),
        provider_type: ProviderType::OpenAI,
    };
    assert_eq!(info.name, "my-provider");
    assert_eq!(info.provider_type, ProviderType::OpenAI);
}

#[test]
fn provider_info_serde_roundtrip() {
    let info = ProviderInfo {
        name: "ollama".to_string(),
        model: "llama3".to_string(),
        provider_type: ProviderType::Ollama,
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: ProviderInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "ollama");
}

// ===============================================================================
// TokenUsageStats
// ===============================================================================

#[test]
fn token_usage_stats_fields() {
    let stats = TokenUsageStats {
        daily_usage: 5000,
        monthly_usage: 50000,
        estimated_cost: 1.5,
        last_reset: std::time::SystemTime::now(),
    };
    assert_eq!(stats.daily_usage, 5000);
    assert_eq!(stats.monthly_usage, 50000);
}

// ===============================================================================
// ProviderError
// ===============================================================================

#[test]
fn provider_error_not_found_display() {
    let e = ProviderError::NotFound("gpt-5".to_string());
    assert!(e.to_string().contains("gpt-5"));
}

#[test]
fn provider_error_not_configured_display() {
    let e = ProviderError::NotConfigured("openai".to_string());
    assert!(e.to_string().contains("openai"));
}

#[test]
fn provider_error_api_error_display() {
    let e = ProviderError::ApiError("rate limit".to_string());
    assert!(e.to_string().contains("rate limit"));
}

#[test]
fn provider_error_budget_exceeded_display() {
    let e = ProviderError::BudgetExceeded {
        limit: 4000,
        requested: 5000,
    };
    let s = e.to_string();
    assert!(s.contains("4000"));
    assert!(s.contains("5000"));
}

#[test]
fn provider_error_daily_budget_display() {
    let e = ProviderError::DailyBudgetExceeded {
        limit: 100000,
        current: 100001,
    };
    assert!(e.to_string().contains("100000"));
}

#[test]
fn provider_error_monthly_budget_display() {
    let e = ProviderError::MonthlyBudgetExceeded {
        limit: 1000000,
        current: 1000001,
    };
    assert!(e.to_string().contains("1000000"));
}

#[test]
fn provider_error_invalid_config_display() {
    let e = ProviderError::InvalidConfig("missing API key".to_string());
    assert!(e.to_string().contains("missing API key"));
}

#[test]
fn provider_error_network_display() {
    let e = ProviderError::NetworkError("timeout".to_string());
    assert!(e.to_string().contains("timeout"));
}

#[test]
fn provider_error_serialization_display() {
    let e = ProviderError::SerializationError("invalid json".to_string());
    assert!(e.to_string().contains("invalid json"));
}

#[test]
fn provider_error_optimization_display() {
    let e = ProviderError::OptimizationError("mnemonic conflict".to_string());
    assert!(e.to_string().contains("mnemonic conflict"));
}

#[test]
fn provider_error_from_serde_json() {
    let serde_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let pe: ProviderError = serde_err.into();
    assert!(matches!(pe, ProviderError::SerializationError(_)));
}

// ===============================================================================
// ProviderRegistry
// ===============================================================================

#[tokio::test]
async fn provider_registry_new_is_empty() {
    let reg = ProviderRegistry::new();
    assert!(reg.list().await.is_empty());
}

#[tokio::test]
async fn provider_registry_default_is_empty() {
    let reg = ProviderRegistry::default();
    assert!(reg.list().await.is_empty());
}

#[tokio::test]
async fn provider_registry_exists_nonexistent() {
    let reg = ProviderRegistry::new();
    assert!(!reg.exists("nonexistent").await);
}

#[tokio::test]
async fn provider_registry_get_nonexistent() {
    let reg = ProviderRegistry::new();
    assert!(reg.get("missing").await.is_none());
}

#[tokio::test]
async fn provider_registry_clone() {
    let reg = ProviderRegistry::new();
    let cloned = reg.clone();
    assert!(cloned.list().await.is_empty());
}

// ===============================================================================
// estimate_tokens
// ===============================================================================

#[test]
fn estimate_tokens_empty_string() {
    assert_eq!(estimate_tokens(""), 1); // max(0/4, 1) = 1
}

#[test]
fn estimate_tokens_short_string() {
    assert_eq!(estimate_tokens("hi"), 1); // max(2/4, 1) = 1
}

#[test]
fn estimate_tokens_four_chars() {
    assert_eq!(estimate_tokens("abcd"), 1);
}

#[test]
fn estimate_tokens_eight_chars() {
    assert_eq!(estimate_tokens("abcdefgh"), 2);
}

#[test]
fn estimate_tokens_longer_text() {
    let text = "This is a longer piece of text that should produce more tokens.";
    let estimated = estimate_tokens(text);
    assert!(estimated > 10);
}

// ===============================================================================
// TokenTracker
// ===============================================================================

#[test]
fn token_tracker_new_zero_usage() {
    let t = TokenTracker::new();
    assert_eq!(t.daily_usage(), 0);
    assert_eq!(t.monthly_usage(), 0);
}

#[test]
fn token_tracker_default_zero_usage() {
    let t = TokenTracker::default();
    assert_eq!(t.daily_usage(), 0);
    assert_eq!(t.monthly_usage(), 0);
}

#[test]
fn token_tracker_record_increments_usage() {
    let mut t = TokenTracker::new();
    t.record(100, 0);
    assert_eq!(t.daily_usage(), 100);
    assert_eq!(t.monthly_usage(), 100);
}

#[test]
fn token_tracker_record_multiple() {
    let mut t = TokenTracker::new();
    t.record(100, 0);
    t.record(200, 0);
    t.record(300, 0);
    assert_eq!(t.daily_usage(), 600);
    assert_eq!(t.monthly_usage(), 600);
}

#[test]
fn token_tracker_last_reset() {
    let before = std::time::SystemTime::now();
    let t = TokenTracker::new();
    let after = std::time::SystemTime::now();
    assert!(t.last_reset() >= before);
    assert!(t.last_reset() <= after);
}

#[test]
fn token_tracker_get_stats() {
    let mut t = TokenTracker::new();
    t.record(1000, 0);
    let stats = t.get_stats();
    assert_eq!(stats.daily_usage, 1000);
    assert_eq!(stats.monthly_usage, 1000);
    assert!(stats.estimated_cost > 0.0);
}

#[test]
fn token_tracker_cost_estimation() {
    let mut t = TokenTracker::new();
    t.record(1000, 0);
    let stats = t.get_stats();
    // 1000 tokens / 1000 * 0.03 = 0.03
    assert!((stats.estimated_cost - 0.03).abs() < 0.001);
}

#[test]
fn token_tracker_large_usage_cost() {
    let mut t = TokenTracker::new();
    t.record(100_000, 0);
    let stats = t.get_stats();
    // 100000 / 1000 * 0.03 = 3.0
    assert!((stats.estimated_cost - 3.0).abs() < 0.01);
}
