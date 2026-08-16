//! Provider-Agent Integration Tests
//!
//! Tests the integration between providers and agents in the runtime.

use hudhudscript_mcp::client::McpClient;
use hudhudscript_resources::ResourceManager;
use hudhudscript_runtime::{
    agent::{Agent, Task},
    provider::{Provider, ProviderConfig, ProviderRegistry, ProviderType, TokenBudget},
    providers::openai::OpenAIProvider,
    AgentRuntime,
};
use hudhudscript_tools::ToolRegistry;
use std::sync::Arc;

async fn create_test_runtime_with_providers() -> (AgentRuntime, Arc<ProviderRegistry>) {
    let mcp_client = Arc::new(
        McpClient::new(hudhudscript_mcp::transport::TransportConfig::stdio(
            "echo",
            vec![],
        ))
        .await
        .unwrap(),
    );

    let tool_registry = Arc::new(ToolRegistry::new());
    let resource_manager = Arc::new(ResourceManager::new(
        mcp_client.clone(),
        std::time::Duration::from_secs(300),
    ));

    let runtime = AgentRuntime::new(tool_registry, resource_manager, mcp_client);
    let provider_registry = runtime.provider_registry();

    (runtime, provider_registry)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_register_provider_to_runtime() {
    let (runtime, provider_registry) = create_test_runtime_with_providers().await;

    // Create OpenAI provider
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(4000),
        budget: Some(TokenBudget {
            max_tokens_per_call: 4000,
            max_tokens_per_day: 100000,
            alert_threshold: 0.8,
        }),
        timeout_secs: None,
        extra: std::collections::HashMap::new(),
    };

    let provider = Arc::new(OpenAIProvider::new(config).unwrap());

    // Register provider
    provider_registry
        .register("openai_gpt4".to_string(), provider)
        .await;

    // Verify provider is registered
    assert!(provider_registry.exists("openai_gpt4").await);

    // Get provider
    let retrieved = provider_registry.get("openai_gpt4").await;
    assert!(retrieved.is_some());

    // Check provider info
    let info = retrieved.unwrap().info();
    assert_eq!(info.name, "OpenAI"); // Provider type name, not registry name
    assert_eq!(info.model, "gpt-4");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_with_provider() {
    let (runtime, provider_registry) = create_test_runtime_with_providers().await;

    // Register provider
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(4000),
        budget: Some(TokenBudget::default()),
        timeout_secs: None,
        extra: std::collections::HashMap::new(),
    };

    let provider = Arc::new(OpenAIProvider::new(config).unwrap());
    provider_registry
        .register("openai_gpt4".to_string(), provider)
        .await;

    // Create agent with provider
    let mut agent = Agent::new("agent-1".to_string(), "DataAnalyst".to_string());
    agent.set_provider("openai_gpt4".to_string());
    agent.description = Some("Analyzes data using GPT-4".to_string());

    // Add a task
    let task = Task::new("analyze_data".to_string(), "Analyze dataset".to_string());
    agent.add_task(task);

    // Register agent
    runtime.register_agent(agent).await.unwrap();

    // Verify agent is registered with provider
    let retrieved_agent = runtime.get_agent("agent-1").await.unwrap();
    assert_eq!(retrieved_agent.get_provider(), Some("openai_gpt4"));

    // Verify provider exists
    assert!(provider_registry.exists("openai_gpt4").await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_agents_with_different_providers() {
    let (runtime, provider_registry) = create_test_runtime_with_providers().await;

    // Register OpenAI provider
    let openai_config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(4000),
        budget: Some(TokenBudget::default()),
        timeout_secs: None,
        extra: std::collections::HashMap::new(),
    };

    let openai_provider = Arc::new(OpenAIProvider::new(openai_config).unwrap());
    provider_registry
        .register("openai_gpt4".to_string(), openai_provider)
        .await;

    // Register another OpenAI provider with different model
    let gpt35_config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-3.5-turbo".to_string(),
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: Some(0.5),
        max_tokens: Some(2000),
        budget: Some(TokenBudget::default()),
        timeout_secs: None,
        extra: std::collections::HashMap::new(),
    };

    let gpt35_provider = Arc::new(OpenAIProvider::new(gpt35_config).unwrap());
    provider_registry
        .register("openai_gpt35".to_string(), gpt35_provider)
        .await;

    // Create first agent with GPT-4
    let mut agent1 = Agent::new("agent-1".to_string(), "DataAnalyst".to_string());
    agent1.set_provider("openai_gpt4".to_string());
    runtime.register_agent(agent1).await.unwrap();

    // Create second agent with GPT-3.5
    let mut agent2 = Agent::new("agent-2".to_string(), "QuickAssistant".to_string());
    agent2.set_provider("openai_gpt35".to_string());
    runtime.register_agent(agent2).await.unwrap();

    // Verify both agents have correct providers
    let retrieved1 = runtime.get_agent("agent-1").await.unwrap();
    assert_eq!(retrieved1.get_provider(), Some("openai_gpt4"));

    let retrieved2 = runtime.get_agent("agent-2").await.unwrap();
    assert_eq!(retrieved2.get_provider(), Some("openai_gpt35"));

    // Verify both providers exist
    assert!(provider_registry.exists("openai_gpt4").await);
    assert!(provider_registry.exists("openai_gpt35").await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_without_provider() {
    let (runtime, _provider_registry) = create_test_runtime_with_providers().await;

    // Create agent without provider
    let agent = Agent::new("agent-1".to_string(), "SimpleAgent".to_string());

    // Register agent
    runtime.register_agent(agent).await.unwrap();

    // Verify agent has no provider
    let retrieved = runtime.get_agent("agent-1").await.unwrap();
    assert_eq!(retrieved.get_provider(), None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_registry_operations() {
    let (_runtime, provider_registry) = create_test_runtime_with_providers().await;

    // Register multiple providers
    for i in 1..=3 {
        let config = ProviderConfig {
            provider_type: ProviderType::OpenAI,
            model: format!("gpt-{}", i),
            api_key: Some("sk-test-key".to_string()),
            endpoint: None,
            temperature: Some(0.7),
            max_tokens: Some(4000),
            budget: Some(TokenBudget::default()),
            timeout_secs: None,
            extra: std::collections::HashMap::new(),
        };

        let provider = Arc::new(OpenAIProvider::new(config).unwrap());
        provider_registry
            .register(format!("provider-{}", i), provider)
            .await;
    }

    // List all providers
    let providers = provider_registry.list().await;
    assert_eq!(providers.len(), 3);

    // Unregister one provider
    let removed = provider_registry.unregister("provider-2").await;
    assert!(removed.is_some());

    // Verify it's removed
    assert!(!provider_registry.exists("provider-2").await);

    // List again
    let providers = provider_registry.list().await;
    assert_eq!(providers.len(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_provider_budget_configuration() {
    let (_runtime, provider_registry) = create_test_runtime_with_providers().await;

    // Create provider with custom budget
    let config = ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("sk-test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(4000),
        budget: Some(TokenBudget {
            max_tokens_per_call: 2000,
            max_tokens_per_day: 50000,
            alert_threshold: 0.9,
        }),
        timeout_secs: None,
        extra: std::collections::HashMap::new(),
    };

    let provider = Arc::new(OpenAIProvider::new(config).unwrap());
    provider_registry
        .register("budget_provider".to_string(), provider.clone())
        .await;

    // Verify provider is registered
    assert!(provider_registry.exists("budget_provider").await);

    // Get provider info
    let info = provider.info();
    assert_eq!(info.model, "gpt-4");
}
