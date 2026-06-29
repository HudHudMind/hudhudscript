//! Provider System Integration Tests

use hudhudscript_runtime::*;
use std::collections::HashMap;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_registry_basic() {
    let registry = ProviderRegistry::new();

    // Initially empty
    assert_eq!(registry.list().await.len(), 0);
    assert!(!registry.exists("test").await);

    // Create a test provider config
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: Some(TokenBudget::default()),
        extra: HashMap::new(),
    };

    // Create provider
    let provider = std::sync::Arc::new(OpenAIProvider::new(config).unwrap());

    // Register provider
    registry
        .register("test_provider".to_string(), provider.clone())
        .await;

    // Verify registration
    assert_eq!(registry.list().await.len(), 1);
    assert!(registry.exists("test_provider").await);

    // Get provider
    let retrieved = registry.get("test_provider").await;
    assert!(retrieved.is_some());

    // Verify provider info
    let info = retrieved.unwrap().info();
    assert_eq!(info.name, "OpenAI");
    assert_eq!(info.model, "gpt-4");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_providers() {
    let registry = ProviderRegistry::new();

    // Create multiple providers
    let configs = vec![
        ("openai_gpt4", "gpt-4"),
        ("openai_gpt35", "gpt-3.5-turbo"),
        ("openai_gpt4_turbo", "gpt-4-turbo"),
    ];

    for (name, model) in configs {
        let config = ProviderConfig {
            provider_type: ProviderType::OpenAI,
            model: model.to_string(),
            api_key: Some("test-key".to_string()),
            endpoint: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
            budget: None,
            extra: HashMap::new(),
        };

        let provider = std::sync::Arc::new(OpenAIProvider::new(config).unwrap());
        registry.register(name.to_string(), provider).await;
    }

    // Verify all registered
    assert_eq!(registry.list().await.len(), 3);

    // Verify each provider
    for (name, model) in &[
        ("openai_gpt4", "gpt-4"),
        ("openai_gpt35", "gpt-3.5-turbo"),
        ("openai_gpt4_turbo", "gpt-4-turbo"),
    ] {
        let provider = registry.get(name).await;
        assert!(provider.is_some());
        let info = provider.unwrap().info();
        assert_eq!(info.model, *model);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_unregister() {
    let registry = ProviderRegistry::new();

    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let provider = std::sync::Arc::new(OpenAIProvider::new(config).unwrap());
    registry.register("test".to_string(), provider).await;

    assert!(registry.exists("test").await);

    // Unregister
    let removed = registry.unregister("test").await;
    assert!(removed.is_some());
    assert!(!registry.exists("test").await);
    assert_eq!(registry.list().await.len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_token_budget_enforcement() {
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: Some(TokenBudget {
            max_tokens_per_call: 1000,
            max_tokens_per_day: 10000,
            alert_threshold: 0.8,
        }),
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::new(config).unwrap();

    // Should pass for reasonable token count
    assert!(provider.check_budget(500).is_ok());

    // Should fail for per-call limit
    let result = provider.check_budget(1500);
    assert!(result.is_err());
    match result {
        Err(ProviderError::BudgetExceeded { limit, requested }) => {
            assert_eq!(limit, 1000);
            assert_eq!(requested, 1500);
        }
        _ => panic!("Expected BudgetExceeded error"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_token_tracker() {
    let mut tracker = TokenTracker::new();

    // Initial state
    assert_eq!(tracker.daily_usage(), 0);
    assert_eq!(tracker.monthly_usage(), 0);

    // Record usage
    tracker.record(100, 0);
    assert_eq!(tracker.daily_usage(), 100);
    assert_eq!(tracker.monthly_usage(), 100);

    tracker.record(200, 0);
    assert_eq!(tracker.daily_usage(), 300);
    assert_eq!(tracker.monthly_usage(), 300);

    // Get stats
    let stats = tracker.get_stats();
    assert_eq!(stats.daily_usage, 300);
    assert_eq!(stats.monthly_usage, 300);
    assert!(stats.estimated_cost > 0.0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_mnemonic_optimization() {
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::new(config).unwrap();

    // Create request with mnemonics
    let mut mnemonics = HashMap::new();
    mnemonics.insert(
        "DA1".to_string(),
        "Validate data format and check for missing values".to_string(),
    );
    mnemonics.insert(
        "DA2".to_string(),
        "Calculate mean, median, standard deviation".to_string(),
    );

    let request = LLMRequest {
        prompt: "Please Validate data format and check for missing values then Calculate mean, median, standard deviation".to_string(),
        system_prompt: None,
        temperature: None,
        max_tokens: None,
        mnemonics: Some(mnemonics),
        optimize: true,
        tools: None,
    };

    // Build messages (this will optimize the prompt)
    let messages = provider.build_messages(&request).unwrap();

    // Verify mnemonic dictionary is added
    let messages_str = format!("{:?}", messages);
    assert!(messages_str.contains("Mnemonic"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_info() {
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::new(config).unwrap();
    let info = provider.info();

    assert_eq!(info.name, "OpenAI");
    assert_eq!(info.model, "gpt-4");
    assert_eq!(info.provider_type, ProviderType::OpenAI);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_usage_stats() {
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let provider = OpenAIProvider::new(config).unwrap();

    // Initial stats
    let stats = provider.get_usage_stats().await;
    assert_eq!(stats.daily_usage, 0);
    assert_eq!(stats.monthly_usage, 0);
    assert_eq!(stats.estimated_cost, 0.0);
}

#[test]
fn test_estimate_tokens() {
    // Test token estimation
    assert_eq!(estimate_tokens("hello"), 1);
    assert_eq!(estimate_tokens("hello world"), 2);
    assert_eq!(estimate_tokens("a".repeat(100).as_str()), 25);
    assert_eq!(estimate_tokens(""), 1); // Minimum 1 token

    // Test with longer text
    let long_text = "This is a longer text that should be estimated to have more tokens based on the character count.";
    let estimated = estimate_tokens(long_text);
    assert!(estimated > 10);
    assert!(estimated < 50);
}

#[test]
fn test_provider_config_validation() {
    // Valid config
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let result = OpenAIProvider::new(config);
    assert!(result.is_ok());

    // Invalid: no API key
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: None,
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let result = OpenAIProvider::new(config);
    assert!(result.is_err());
    match result {
        Err(ProviderError::InvalidConfig(msg)) => {
            assert!(msg.contains("API key"));
        }
        _ => panic!("Expected InvalidConfig error"),
    }

    // Invalid: wrong provider type
    let config = ProviderConfig {
        provider_type: ProviderType::Anthropic,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let result = OpenAIProvider::new(config);
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_provider_access() {
    use std::sync::Arc as StdArc;

    let registry = StdArc::new(ProviderRegistry::new());

    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let provider = std::sync::Arc::new(OpenAIProvider::new(config).unwrap());
    registry.register("test".to_string(), provider).await;

    // Spawn multiple tasks accessing the registry concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let registry_clone = StdArc::clone(&registry);
        let handle = tokio::spawn(async move {
            let provider = registry_clone.get("test").await;
            assert!(provider.is_some());
            let info = provider.unwrap().info();
            assert_eq!(info.model, "gpt-4");
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
}

#[test]
fn test_token_budget_default() {
    let budget = TokenBudget::default();
    assert_eq!(budget.max_tokens_per_call, 4000);
    assert_eq!(budget.max_tokens_per_day, 100000);
    assert_eq!(budget.alert_threshold, 0.8);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_registry_clone() {
    let registry = ProviderRegistry::new();

    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let provider = std::sync::Arc::new(OpenAIProvider::new(config).unwrap());
    registry.register("test".to_string(), provider).await;

    // Clone registry (Arc clone, not deep clone)
    let registry2 = registry.clone();

    // Both should see the same provider
    assert!(registry.exists("test").await);
    assert!(registry2.exists("test").await);

    // Register in one
    let config2 = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-3.5-turbo".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: None,
        extra: HashMap::new(),
    };

    let provider2 = std::sync::Arc::new(OpenAIProvider::new(config2).unwrap());
    registry2.register("test2".to_string(), provider2).await;

    // Both should see both providers
    let list1 = registry.list().await;
    let list2 = registry2.list().await;
    assert_eq!(list1.len(), 2);
    assert_eq!(list2.len(), 2);
}
