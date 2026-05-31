//! Tests extracted from hudhudscript-modules/src/model_manager.rs

use hudhudscript_modules::{
    ModelEntry, ModelFormat, ModelManager, ModelManagerError, ModelProvider,
};
use std::path::PathBuf;
use tempfile::TempDir;

fn sample_entry(name: &str) -> ModelEntry {
    ModelEntry {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        provider: ModelProvider::Local,
        path: PathBuf::from("/models").join(name),
        size: 1_000_000,
        format: ModelFormat::Gguf,
    }
}

#[test]
fn test_register_and_get() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("llama")).unwrap();

    let entry = mgr.get("llama").unwrap();
    assert_eq!(entry.name, "llama");
    assert_eq!(entry.version, "1.0.0");
}

#[test]
fn test_duplicate_register() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("llama")).unwrap();
    assert!(matches!(
        mgr.register(sample_entry("llama")),
        Err(ModelManagerError::AlreadyExists(_))
    ));
}

#[test]
fn test_list() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("a")).unwrap();
    mgr.register(sample_entry("b")).unwrap();
    assert_eq!(mgr.list().len(), 2);
}

#[test]
fn test_remove() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("llama")).unwrap();
    let removed = mgr.remove("llama").unwrap();
    assert_eq!(removed.name, "llama");
    assert!(mgr.get("llama").is_err());
}

#[test]
fn test_remove_not_found() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    assert!(matches!(
        mgr.remove("nope"),
        Err(ModelManagerError::NotFound(_))
    ));
}

#[test]
fn test_disk_usage() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("a")).unwrap();
    mgr.register(sample_entry("b")).unwrap();
    assert_eq!(mgr.disk_usage(), 2_000_000);
}

#[test]
fn test_check_disk_space_ok() {
    let tmp = TempDir::new().unwrap();
    let mgr = ModelManager::new(tmp.path());
    // Asking for 1 byte should always succeed on any real filesystem.
    assert!(mgr.check_disk_space(1).unwrap());
}

#[test]
fn test_check_disk_space_insufficient() {
    let tmp = TempDir::new().unwrap();
    let mgr = ModelManager::new(tmp.path());
    // Asking for an absurd amount should fail.
    assert!(matches!(
        mgr.check_disk_space(u64::MAX),
        Err(ModelManagerError::InsufficientDiskSpace { .. })
    ));
}

#[test]
fn test_model_format_variants() {
    assert_ne!(ModelFormat::Gguf, ModelFormat::SafeTensors);
    assert_ne!(ModelFormat::Bin, ModelFormat::Other("custom".into()));
}

#[test]
fn test_model_provider_variants() {
    assert_ne!(ModelProvider::HuggingFace, ModelProvider::Ollama);
    assert_ne!(ModelProvider::Ollama, ModelProvider::Local);
}

// ---- ModelManagerError display ----

#[test]
fn test_model_manager_error_display() {
    let err = ModelManagerError::NotFound("mymodel".to_string());
    assert!(err.to_string().contains("Model not found: mymodel"));

    let err = ModelManagerError::AlreadyExists("llama".to_string());
    assert!(err.to_string().contains("Model already registered: llama"));

    let err = ModelManagerError::InsufficientDiskSpace {
        needed: 1000,
        available: 500,
    };
    let msg = err.to_string();
    assert!(msg.contains("1000"));
    assert!(msg.contains("500"));

    let err = ModelManagerError::Io("permission denied".to_string());
    assert!(err.to_string().contains("permission denied"));
}

// ---- ModelEntry serialization roundtrip ----

#[test]
fn test_model_entry_serialization() {
    let entry = sample_entry("test_model");
    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: ModelEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "test_model");
    assert_eq!(deserialized.version, "1.0.0");
    assert_eq!(deserialized.provider, ModelProvider::Local);
    assert_eq!(deserialized.format, ModelFormat::Gguf);
    assert_eq!(deserialized.size, 1_000_000);
}

// ---- ModelFormat serialization roundtrip ----

#[test]
fn test_model_format_serialization() {
    for format in &[
        ModelFormat::Gguf,
        ModelFormat::SafeTensors,
        ModelFormat::Bin,
        ModelFormat::Other("onnx".to_string()),
    ] {
        let json = serde_json::to_string(format).unwrap();
        let deserialized: ModelFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, format);
    }
}

// ---- ModelProvider serialization roundtrip ----

#[test]
fn test_model_provider_serialization() {
    for provider in &[
        ModelProvider::HuggingFace,
        ModelProvider::Ollama,
        ModelProvider::Local,
    ] {
        let json = serde_json::to_string(provider).unwrap();
        let deserialized: ModelProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, provider);
    }
}

// ---- get not found error ----

#[test]
fn test_get_not_found() {
    let tmp = TempDir::new().unwrap();
    let mgr = ModelManager::new(tmp.path());
    let result = mgr.get("nonexistent");
    assert!(matches!(result, Err(ModelManagerError::NotFound(_))));
}

// ---- disk_usage on empty manager ----

#[test]
fn test_disk_usage_empty() {
    let tmp = TempDir::new().unwrap();
    let mgr = ModelManager::new(tmp.path());
    assert_eq!(mgr.disk_usage(), 0);
}

// ---- list empty ----

#[test]
fn test_list_empty() {
    let tmp = TempDir::new().unwrap();
    let mgr = ModelManager::new(tmp.path());
    assert!(mgr.list().is_empty());
}

// ---- remove then re-register ----

#[test]
fn test_remove_then_reregister() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("model_a")).unwrap();
    mgr.remove("model_a").unwrap();
    // Should be able to register again
    mgr.register(sample_entry("model_a")).unwrap();
    assert_eq!(mgr.list().len(), 1);
}

#[test]
fn test_disk_usage_after_remove() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("a")).unwrap();
    mgr.register(sample_entry("b")).unwrap();
    assert_eq!(mgr.disk_usage(), 2_000_000);

    mgr.remove("a").unwrap();
    assert_eq!(mgr.disk_usage(), 1_000_000);
}

#[test]
fn test_register_different_providers() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());

    let hf_entry = ModelEntry {
        name: "hf-model".to_string(),
        version: "1.0.0".to_string(),
        provider: ModelProvider::HuggingFace,
        path: PathBuf::from("/models/hf"),
        size: 5_000_000,
        format: ModelFormat::SafeTensors,
    };
    mgr.register(hf_entry).unwrap();

    let ollama_entry = ModelEntry {
        name: "ollama-model".to_string(),
        version: "latest".to_string(),
        provider: ModelProvider::Ollama,
        path: PathBuf::from("/models/ollama"),
        size: 3_000_000,
        format: ModelFormat::Bin,
    };
    mgr.register(ollama_entry).unwrap();

    assert_eq!(mgr.list().len(), 2);
    assert_eq!(mgr.disk_usage(), 8_000_000);

    let hf = mgr.get("hf-model").unwrap();
    assert_eq!(hf.provider, ModelProvider::HuggingFace);
    assert_eq!(hf.format, ModelFormat::SafeTensors);

    let ol = mgr.get("ollama-model").unwrap();
    assert_eq!(ol.provider, ModelProvider::Ollama);
    assert_eq!(ol.format, ModelFormat::Bin);
}

#[test]
fn test_model_format_other() {
    let entry = ModelEntry {
        name: "custom".to_string(),
        version: "0.1.0".to_string(),
        provider: ModelProvider::Local,
        path: PathBuf::from("/custom"),
        size: 100,
        format: ModelFormat::Other("onnx".to_string()),
    };
    assert_eq!(entry.format, ModelFormat::Other("onnx".to_string()));
}

#[test]
fn test_model_manager_clone() {
    let tmp = TempDir::new().unwrap();
    let mut mgr = ModelManager::new(tmp.path());
    mgr.register(sample_entry("test")).unwrap();

    let cloned = mgr.clone();
    assert_eq!(cloned.list().len(), 1);
    assert_eq!(cloned.disk_usage(), 1_000_000);
}

#[test]
fn test_check_disk_space_small_requirement() {
    let tmp = TempDir::new().unwrap();
    let mgr = ModelManager::new(tmp.path());
    // 1 byte should always be available
    assert!(mgr.check_disk_space(1).is_ok());
}

#[test]
fn test_model_entry_path() {
    let entry = sample_entry("my-model");
    assert_eq!(entry.path, PathBuf::from("/models/my-model"));
}

#[test]
fn test_insufficient_disk_space_error_message() {
    let err = ModelManagerError::InsufficientDiskSpace {
        needed: 10_000_000,
        available: 5_000_000,
    };
    let msg = err.to_string();
    assert!(msg.contains("10000000"));
    assert!(msg.contains("5000000"));
}
