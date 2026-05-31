//! Tests extracted from hudhudscript-modules/src/ollama.rs

use hudhudscript_modules::{
    value_to_manifest, value_to_ollama_model, OllamaClient, OllamaError, OllamaLayer,
    OllamaManifest, OllamaModel,
};

#[test]
fn test_value_to_ollama_model_with_tag() {
    let v = serde_json::json!({
        "name": "llama2:7b-q4_0",
        "size": 4_000_000_000u64,
        "digest": "sha256:abc123",
        "modified_at": "2025-06-01T12:00:00Z"
    });
    let m = value_to_ollama_model(&v);
    assert_eq!(m.name, "llama2");
    assert_eq!(m.tag, "7b-q4_0");
    assert_eq!(m.size, 4_000_000_000);
}

#[test]
fn test_value_to_ollama_model_no_tag() {
    let v = serde_json::json!({
        "name": "mistral",
        "size": 0,
        "digest": "",
        "modified_at": ""
    });
    let m = value_to_ollama_model(&v);
    assert_eq!(m.name, "mistral");
    assert_eq!(m.tag, "latest");
}

#[test]
fn test_value_to_manifest() {
    let v = serde_json::json!({
        "mediaType": "application/vnd.ollama.image.manifest.v1+json",
        "layers": [
            {
                "mediaType": "application/vnd.ollama.image.model",
                "digest": "sha256:abc",
                "size": 1000
            }
        ],
        "config": {
            "mediaType": "application/vnd.ollama.image.config",
            "digest": "sha256:def",
            "size": 200
        }
    });
    let manifest = value_to_manifest(&v).unwrap();
    assert_eq!(manifest.layers.len(), 1);
    assert!(manifest.config.is_some());
}

#[test]
fn test_ollama_client_default_url() {
    let client = OllamaClient::local();
    assert_eq!(client.base_url, "http://localhost:11434");
}

// ---- OllamaClient custom URL ----

#[test]
fn test_ollama_client_custom_url() {
    let client = OllamaClient::new("http://remote:11434");
    assert_eq!(client.base_url, "http://remote:11434");
}

// ---- value_to_ollama_model edge cases ----

#[test]
fn test_value_to_ollama_model_missing_fields() {
    let v = serde_json::json!({});
    let m = value_to_ollama_model(&v);
    assert_eq!(m.name, "");
    assert_eq!(m.tag, "latest");
    assert_eq!(m.size, 0);
    assert_eq!(m.digest, "");
    assert_eq!(m.modified_at, "");
}

// ---- value_to_manifest edge cases ----

#[test]
fn test_value_to_manifest_defaults() {
    // Missing layers, config, mediaType
    let v = serde_json::json!({});
    let manifest = value_to_manifest(&v).unwrap();
    assert!(manifest.layers.is_empty());
    assert!(manifest.config.is_none());
    assert_eq!(
        manifest.media_type,
        "application/vnd.ollama.image.manifest.v1+json"
    );
}

// ---- OllamaError display ----

#[test]
fn test_ollama_error_display() {
    let err = OllamaError::Http("timeout".to_string());
    assert!(err.to_string().contains("timeout"));

    let err = OllamaError::Deserialize("unexpected token".to_string());
    assert!(err.to_string().contains("unexpected token"));
}

// ---- OllamaModel serialization roundtrip ----

#[test]
fn test_ollama_model_serialization() {
    let model = OllamaModel {
        name: "llama2".to_string(),
        tag: "7b".to_string(),
        size: 4_000_000_000,
        digest: "sha256:abc".to_string(),
        modified_at: "2025-01-01".to_string(),
    };
    let json = serde_json::to_string(&model).unwrap();
    let deserialized: OllamaModel = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "llama2");
    assert_eq!(deserialized.tag, "7b");
    assert_eq!(deserialized.size, 4_000_000_000);
}

// ---- OllamaManifest serialization roundtrip ----

#[test]
fn test_ollama_manifest_serialization() {
    let manifest = OllamaManifest {
        layers: vec![OllamaLayer {
            media_type: "application/vnd.ollama.image.model".to_string(),
            digest: "sha256:abc".to_string(),
            size: 1000,
        }],
        config: None,
        media_type: "application/vnd.ollama.image.manifest.v1+json".to_string(),
    };
    let json = serde_json::to_string(&manifest).unwrap();
    let deserialized: OllamaManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.layers.len(), 1);
    assert_eq!(deserialized.layers[0].size, 1000);
}

// ---- OllamaLayer serialization ----

#[test]
fn test_ollama_layer_serialization() {
    let layer = OllamaLayer {
        media_type: "application/vnd.ollama.image.model".to_string(),
        digest: "sha256:def".to_string(),
        size: 500,
    };
    let json = serde_json::to_string(&layer).unwrap();
    assert!(json.contains("mediaType"));
    let deserialized: OllamaLayer = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.media_type,
        "application/vnd.ollama.image.model"
    );
    assert_eq!(deserialized.digest, "sha256:def");
    assert_eq!(deserialized.size, 500);
}
