use hudhudscript_runtime::provider::*;
use std::sync::Arc;

#[test]
fn test_token_tracker_daily_reset_threshold() {
    let tracker = TokenTracker::new();
    // A freshly created tracker should NOT need a daily reset
    // (less than 86400 seconds have elapsed)
    assert!(!tracker.should_reset_daily());
    assert!(!tracker.should_reset_monthly());

    // Create a tracker with a stale daily reset time (>24h ago)
    let mut stale_tracker = TokenTracker::new();
    stale_tracker.last_daily_reset =
        std::time::SystemTime::now() - std::time::Duration::from_secs(86401);
    assert!(stale_tracker.should_reset_daily());

    // Create a tracker with a stale monthly reset time (>30 days ago)
    let mut stale_monthly = TokenTracker::new();
    stale_monthly.last_monthly_reset =
        std::time::SystemTime::now() - std::time::Duration::from_secs(2592001);
    assert!(stale_monthly.should_reset_monthly());
}

#[test]
fn test_estimate_tokens_edge_cases() {
    // Empty string — min is 1
    assert_eq!(estimate_tokens(""), 1);

    // Single character — len=1, 1/4=0, max(0,1)=1
    assert_eq!(estimate_tokens("a"), 1);

    // Exactly 4 chars — 4/4=1
    assert_eq!(estimate_tokens("abcd"), 1);

    // 5 chars — 5/4=1 (integer division)
    assert_eq!(estimate_tokens("abcde"), 1);

    // 8 chars — 8/4=2
    assert_eq!(estimate_tokens("abcdefgh"), 2);

    // Unicode: each CJK char is 3 bytes in UTF-8
    // "你好" = 6 bytes, 6/4 = 1
    assert_eq!(estimate_tokens("你好"), 1);

    // "你好世界" = 12 bytes, 12/4 = 3
    assert_eq!(estimate_tokens("你好世界"), 3);

    // Emoji: 🎉 is 4 bytes, so "🎉🎉" = 8 bytes, 8/4 = 2
    assert_eq!(estimate_tokens("🎉🎉"), 2);

    // Very long string: 10000 chars -> 10000/4 = 2500
    let long_string = "x".repeat(10000);
    assert_eq!(estimate_tokens(&long_string), 2500);
}

#[test]
fn test_llm_request_with_tools() {
    let tool = ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the weather for a city".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "city": { "type": "string" }
            },
            "required": ["city"]
        }),
    };

    let request = LLMRequest {
        prompt: "What's the weather in Berlin?".to_string(),
        system_prompt: Some("You are a helpful assistant.".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(512),
        mnemonics: None,
        optimize: false,
        tools: Some(vec![tool]),
    };

    assert_eq!(request.prompt, "What's the weather in Berlin?");
    assert_eq!(
        request.system_prompt.as_deref(),
        Some("You are a helpful assistant.")
    );
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(512));
    assert!(!request.optimize);

    let tools = request.tools.as_ref().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "get_weather");
    assert_eq!(tools[0].description, "Get the weather for a city");
    assert_eq!(tools[0].parameters["type"], "object");
    assert_eq!(tools[0].parameters["required"][0], "city");
}

#[test]
fn test_function_call_result_ok_and_err() {
    let ok_result =
        FunctionCallResult::ok("call-1", "get_weather", serde_json::json!({"temp": 22}));
    assert_eq!(ok_result.id, "call-1");
    assert_eq!(ok_result.name, "get_weather");
    assert_eq!(ok_result.output, serde_json::json!({"temp": 22}));
    assert!(ok_result.success);
    assert!(ok_result.error.is_none());

    let err_result = FunctionCallResult::err("call-2", "get_weather", "city not found");
    assert_eq!(err_result.id, "call-2");
    assert_eq!(err_result.name, "get_weather");
    assert_eq!(err_result.output, serde_json::Value::Null);
    assert!(!err_result.success);
    assert_eq!(err_result.error.as_deref(), Some("city not found"));
}

#[test]
fn test_function_call_result_as_tool_call_result() {
    // Successful result conversion
    let ok_result = FunctionCallResult::ok("call-1", "lookup", serde_json::json!({"found": true}));
    let tool_result = ok_result.as_tool_call_result();
    assert_eq!(tool_result.tool_call_id, "call-1");
    assert_eq!(tool_result.name, "lookup");
    assert_eq!(tool_result.content, "{\"found\":true}");

    // Error result conversion
    let err_result = FunctionCallResult::err("call-2", "lookup", "not found");
    let tool_result = err_result.as_tool_call_result();
    assert_eq!(tool_result.tool_call_id, "call-2");
    assert_eq!(tool_result.name, "lookup");
    assert_eq!(tool_result.content, "Error: not found");

    // Error result with no error message (shouldn't happen, but test the path)
    let mut no_msg = FunctionCallResult::err("call-3", "lookup", "oops");
    no_msg.error = None; // manually clear it
    let tool_result = no_msg.as_tool_call_result();
    assert_eq!(tool_result.content, "Error: unknown");
}

#[tokio::test]
async fn test_registry_unregister() {
    let registry = ProviderRegistry::new();

    // Create a mock provider
    struct MockProvider;

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        async fn call(&self, _req: LLMRequest) -> Result<LLMResponse, ProviderError> {
            unimplemented!()
        }
        async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
            Ok(vec!["mock-model".to_string()])
        }
        fn info(&self) -> ProviderInfo {
            ProviderInfo {
                name: "mock".to_string(),
                model: "mock-model".to_string(),
                provider_type: ProviderType::Ollama,
            }
        }
        fn check_budget(&self, _tokens: usize) -> Result<(), ProviderError> {
            Ok(())
        }
        async fn get_usage_stats(&self) -> TokenUsageStats {
            TokenUsageStats {
                daily_usage: 0,
                monthly_usage: 0,
                estimated_cost: 0.0,
                last_reset: std::time::SystemTime::now(),
            }
        }
    }

    let provider: Arc<dyn Provider> = Arc::new(MockProvider);
    registry.register("mock".to_string(), provider).await;

    assert!(registry.exists("mock").await);
    assert_eq!(registry.list().await.len(), 1);

    let removed = registry.unregister("mock").await;
    assert!(removed.is_some());
    assert!(!registry.exists("mock").await);
    assert_eq!(registry.list().await.len(), 0);

    // Unregistering a nonexistent provider returns None
    let removed_again = registry.unregister("mock").await;
    assert!(removed_again.is_none());
}

#[tokio::test]
async fn test_registry_overwrite() {
    struct ProviderA;
    struct ProviderB;

    #[async_trait::async_trait]
    impl Provider for ProviderA {
        async fn call(&self, _req: LLMRequest) -> Result<LLMResponse, ProviderError> {
            unimplemented!()
        }
        async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
            Ok(vec!["model-a".to_string()])
        }
        fn info(&self) -> ProviderInfo {
            ProviderInfo {
                name: "provider-a".to_string(),
                model: "model-a".to_string(),
                provider_type: ProviderType::OpenAI,
            }
        }
        fn check_budget(&self, _tokens: usize) -> Result<(), ProviderError> {
            Ok(())
        }
        async fn get_usage_stats(&self) -> TokenUsageStats {
            TokenUsageStats {
                daily_usage: 0,
                monthly_usage: 0,
                estimated_cost: 0.0,
                last_reset: std::time::SystemTime::now(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Provider for ProviderB {
        async fn call(&self, _req: LLMRequest) -> Result<LLMResponse, ProviderError> {
            unimplemented!()
        }
        async fn list_models(&self) -> Result<Vec<String>, ProviderError> {
            Ok(vec!["model-b".to_string()])
        }
        fn info(&self) -> ProviderInfo {
            ProviderInfo {
                name: "provider-b".to_string(),
                model: "model-b".to_string(),
                provider_type: ProviderType::Anthropic,
            }
        }
        fn check_budget(&self, _tokens: usize) -> Result<(), ProviderError> {
            Ok(())
        }
        async fn get_usage_stats(&self) -> TokenUsageStats {
            TokenUsageStats {
                daily_usage: 0,
                monthly_usage: 0,
                estimated_cost: 0.0,
                last_reset: std::time::SystemTime::now(),
            }
        }
    }

    let registry = ProviderRegistry::new();
    registry
        .register("same-name".to_string(), Arc::new(ProviderA))
        .await;

    let first = registry.get("same-name").await.unwrap();
    assert_eq!(first.info().name, "provider-a");
    assert_eq!(first.info().model, "model-a");

    // Overwrite with ProviderB
    registry
        .register("same-name".to_string(), Arc::new(ProviderB))
        .await;

    // Still only one entry
    assert_eq!(registry.list().await.len(), 1);

    // The second provider wins
    let second = registry.get("same-name").await.unwrap();
    assert_eq!(second.info().name, "provider-b");
    assert_eq!(second.info().model, "model-b");
}
