use hudhudscript_runtime::provider::{ProviderConfig, ProviderType, TokenBudget};
use hudhudscript_runtime::providers::OllamaProvider;
use std::collections::HashMap;

fn make_config(endpoint: Option<String>) -> ProviderConfig {
    ProviderConfig {
        provider_type: ProviderType::Ollama,
        model: "llama2".to_string(),
        api_key: None,
        endpoint,
        temperature: Some(0.7),
        max_tokens: Some(1000),
        budget: None,
        timeout_secs: None,
        extra: HashMap::new(),
    }
}

#[test]
fn test_ollama_provider_creation() {
    let provider = OllamaProvider::new(make_config(None));
    assert!(provider.is_ok());
}

#[test]
fn test_ollama_custom_endpoint() {
    let provider = OllamaProvider::new(make_config(Some(
        "http://custom:11434/api/generate".to_string(),
    )))
    .unwrap();
    assert_eq!(provider.get_endpoint(), "http://custom:11434/api/generate");
}

#[test]
fn test_ollama_info() {
    let provider = OllamaProvider::new(make_config(None)).unwrap();
    let info = provider.info();

    assert_eq!(info.name, "ollama");
    assert_eq!(info.model, "llama2");
    assert_eq!(info.provider_type, ProviderType::Ollama);
}
