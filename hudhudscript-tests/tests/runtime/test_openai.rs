use hudhudscript_runtime::provider::{
    LLMRequest, ProviderConfig, ProviderType, TokenBudget, ToolDefinition,
};
use hudhudscript_runtime::providers::OpenAIProvider;
use std::collections::HashMap;

fn create_test_config() -> ProviderConfig {
    ProviderConfig {
        provider_type: ProviderType::OpenAI,
        model: "gpt-4".to_string(),
        api_key: Some("test-key".to_string()),
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(2000),
        budget: Some(TokenBudget::default()),
        timeout_secs: None,
        extra: HashMap::new(),
    }
}

#[test]
fn test_provider_creation() {
    let config = create_test_config();
    let provider = OpenAIProvider::new(config);
    assert!(provider.is_ok());
}

#[test]
fn test_provider_creation_without_api_key() {
    let mut config = create_test_config();
    config.api_key = None;
    let provider = OpenAIProvider::new(config);
    assert!(provider.is_err());
}

#[test]
fn test_provider_info() {
    let config = create_test_config();
    let provider = OpenAIProvider::new(config).unwrap();
    let info = provider.info();
    assert_eq!(info.name, "OpenAI");
    assert_eq!(info.model, "gpt-4");
    assert_eq!(info.provider_type, ProviderType::OpenAI);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_budget_check() {
    let config = create_test_config();
    let provider = OpenAIProvider::new(config).unwrap();

    // Should pass for reasonable token count
    assert!(provider.check_budget(100).is_ok());

    // Should fail for excessive token count
    assert!(provider.check_budget(10000).is_err());
}

#[test]
fn test_mnemonic_optimization() {
    let config = create_test_config();
    let provider = OpenAIProvider::new(config).unwrap();

    let mut mnemonics = HashMap::new();
    mnemonics.insert("DA1".to_string(), "Validate data format".to_string());
    mnemonics.insert("DA2".to_string(), "Calculate statistics".to_string());

    let request = LLMRequest {
        prompt: "Execute: DA1 then DA2".to_string(),
        system_prompt: None,
        temperature: None,
        max_tokens: None,
        mnemonics: Some(mnemonics),
        optimize: true,
        tools: None,
        timeout_secs: None,
    };

    let optimized = provider.optimize_with_mnemonics(&request).unwrap();
    assert!(optimized.contains("Mnemonics"));
}

#[tokio::test]
async fn test_usage_stats() {
    let config = create_test_config();
    let provider = OpenAIProvider::new(config).unwrap();

    let stats = provider.get_usage_stats().await;
    assert_eq!(stats.daily_usage, 0);
    assert_eq!(stats.monthly_usage, 0);
}
