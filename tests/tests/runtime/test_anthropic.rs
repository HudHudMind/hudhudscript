use hudhudscript_runtime::provider::{ProviderConfig, ProviderType};
use hudhudscript_runtime::providers::AnthropicProvider;
use std::collections::HashMap;

fn make_config(api_key: Option<String>) -> ProviderConfig {
    ProviderConfig {
        provider_type: ProviderType::Anthropic,
        model: "claude-3-opus-20240229".to_string(),
        api_key,
        endpoint: None,
        temperature: Some(0.7),
        max_tokens: Some(1000),
        budget: None,
        extra: HashMap::new(),
    }
}

#[test]
fn test_anthropic_provider_creation() {
    let provider = AnthropicProvider::new(make_config(Some("test-key".to_string())));
    assert!(provider.is_ok());
}

#[test]
fn test_anthropic_provider_missing_api_key() {
    // Should fail without API key in env
    std::env::remove_var("ANTHROPIC_API_KEY");
    let provider = AnthropicProvider::new(make_config(None));
    assert!(provider.is_err());
}

#[test]
fn test_anthropic_info() {
    let provider = AnthropicProvider::new(make_config(Some("test-key".to_string()))).unwrap();
    let info = provider.info();

    assert_eq!(info.name, "anthropic");
    assert_eq!(info.model, "claude-3-opus-20240229");
    assert_eq!(info.provider_type, ProviderType::Anthropic);
}
