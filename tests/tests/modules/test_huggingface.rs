//! Tests extracted from hudhudscript-modules/src/huggingface.rs

use hudhudscript_modules::{value_to_hf_model, HfClient, HfError, HfFile, HfModel};

#[test]
fn test_download_url_construction() {
    let client = HfClient::public();
    let url = client.download_url("TheBloke/Llama-2-7B-GGUF", "model.gguf");
    assert_eq!(
        url,
        "https://huggingface.co/TheBloke/Llama-2-7B-GGUF/resolve/main/model.gguf"
    );
}

#[test]
fn test_value_to_hf_model() {
    let v = serde_json::json!({
        "modelId": "org/model",
        "author": "org",
        "tags": ["gguf", "llama"],
        "downloads": 42,
        "lastModified": "2025-01-01T00:00:00Z",
        "pipeline_tag": "text-generation"
    });
    let m = value_to_hf_model(&v);
    assert_eq!(m.model_id, "org/model");
    assert_eq!(m.author, "org");
    assert_eq!(m.downloads, 42);
    assert_eq!(m.tags, vec!["gguf", "llama"]);
    assert_eq!(m.pipeline_tag, Some("text-generation".into()));
}

#[test]
fn test_hf_client_with_auth() {
    let client = HfClient::new("https://huggingface.co", Some("hf_token123".into()));
    assert_eq!(client.auth_token, Some("hf_token123".into()));
}

// ---- value_to_hf_model edge cases ----

#[test]
fn test_value_to_hf_model_missing_fields() {
    // All fields missing should use defaults
    let v = serde_json::json!({});
    let m = value_to_hf_model(&v);
    assert_eq!(m.model_id, "");
    assert_eq!(m.author, "");
    assert!(m.tags.is_empty());
    assert_eq!(m.downloads, 0);
    assert_eq!(m.last_modified, "");
    assert!(m.pipeline_tag.is_none());
}

#[test]
fn test_value_to_hf_model_uses_id_fallback() {
    // When "modelId" is absent, should fall back to "id"
    let v = serde_json::json!({
        "id": "fallback/model"
    });
    let m = value_to_hf_model(&v);
    assert_eq!(m.model_id, "fallback/model");
}

// ---- download_url with custom base ----

#[test]
fn test_download_url_custom_base() {
    let client = HfClient::new("https://custom.hub.co", None);
    let url = client.download_url("org/model", "weights.bin");
    assert_eq!(
        url,
        "https://custom.hub.co/org/model/resolve/main/weights.bin"
    );
}

// ---- HfError display ----

#[test]
fn test_hf_error_display() {
    let err = HfError::Http("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));

    let err = HfError::Deserialize("invalid json".to_string());
    assert!(err.to_string().contains("invalid json"));
}

// ---- HfModel serialization roundtrip ----

#[test]
fn test_hf_model_serialization() {
    let model = HfModel {
        model_id: "test/model".to_string(),
        author: "test".to_string(),
        tags: vec!["gguf".to_string()],
        downloads: 100,
        last_modified: "2025-01-01".to_string(),
        pipeline_tag: Some("text-generation".to_string()),
    };
    let json = serde_json::to_string(&model).unwrap();
    let deserialized: HfModel = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.model_id, "test/model");
    assert_eq!(deserialized.downloads, 100);
    assert_eq!(deserialized.pipeline_tag, Some("text-generation".into()));
}

// ---- HfFile serialization ----

#[test]
fn test_hf_file_serialization() {
    let file = HfFile {
        filename: "model.safetensors".to_string(),
        size: Some(1_000_000),
    };
    let json = serde_json::to_string(&file).unwrap();
    let deserialized: HfFile = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.filename, "model.safetensors");
    assert_eq!(deserialized.size, Some(1_000_000));
}

#[test]
fn test_hf_file_no_size() {
    let file = HfFile {
        filename: "readme.md".to_string(),
        size: None,
    };
    let json = serde_json::to_string(&file).unwrap();
    let deserialized: HfFile = serde_json::from_str(&json).unwrap();
    assert!(deserialized.size.is_none());
}

// ---- HfClient public base_url ----

#[test]
fn test_hf_client_public_base_url() {
    let client = HfClient::public();
    assert_eq!(client.base_url, "https://huggingface.co");
    assert!(client.auth_token.is_none());
}
