//! Tests extracted from hudhudscript-rag/src/provider.rs

use hudhudscript_rag::provider::{ApiProvider, ApiProviderConfig, MockProvider};
use hudhudscript_rag::EmbeddingProvider;

#[test]
fn test_mock_provider_dimensions() {
    let p = MockProvider::new(64);
    assert_eq!(p.dimensions(), 64);
}

#[test]
fn test_mock_provider_embed() {
    let p = MockProvider::new(32);
    let v = p.embed("hello world").unwrap();
    assert_eq!(v.len(), 32);
}

#[test]
fn test_mock_provider_deterministic() {
    let p = MockProvider::new(32);
    let v1 = p.embed("test").unwrap();
    let v2 = p.embed("test").unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn test_mock_provider_different_texts() {
    let p = MockProvider::new(32);
    let v1 = p.embed("hello").unwrap();
    let v2 = p.embed("world").unwrap();
    assert_ne!(v1, v2);
}

#[test]
fn test_mock_provider_empty_input() {
    let p = MockProvider::new(32);
    assert!(p.embed("").is_err());
    assert!(p.embed("   ").is_err());
}

#[test]
fn test_mock_provider_normalized() {
    let p = MockProvider::new(64);
    let v = p.embed("normalize me").unwrap();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4, "norm was {}", norm);
}

#[test]
fn test_api_provider_creation() {
    let config = ApiProviderConfig {
        endpoint: "https://api.example.com/embed".to_string(),
        api_key: "test-key".to_string(),
        model: "test-model".to_string(),
        dimensions: 128,
    };
    let p = ApiProvider::new(config);
    assert_eq!(p.dimensions(), 128);
    assert_eq!(p.config().model, "test-model");
}

#[test]
fn test_api_provider_empty_input() {
    let config = ApiProviderConfig {
        endpoint: "https://api.example.com/embed".to_string(),
        api_key: "key".to_string(),
        model: "model".to_string(),
        dimensions: 64,
    };
    let p = ApiProvider::new(config);
    let result = p.embed("");
    assert!(result.is_err());
    let result = p.embed("   ");
    assert!(result.is_err());
}

#[test]
fn test_api_provider_error_message_contains_model_and_endpoint() {
    let config = ApiProviderConfig {
        endpoint: "https://myhost.com/v1/embed".to_string(),
        api_key: "key".to_string(),
        model: "my-model".to_string(),
        dimensions: 64,
    };
    let p = ApiProvider::new(config);
    let err = p.embed("test").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("my-model"),
        "error msg missing 'my-model': {}",
        msg
    );
    assert!(msg.contains("myhost.com"));
}

#[test]
fn test_api_provider_returns_error() {
    let config = ApiProviderConfig {
        endpoint: "https://api.example.com/embed".to_string(),
        api_key: "key".to_string(),
        model: "model".to_string(),
        dimensions: 64,
    };
    let p = ApiProvider::new(config);
    let result = p.embed("test");
    assert!(result.is_err());
}
