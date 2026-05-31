//! External tests for hudhudscript-package::registry —
//! RegistryClient, PackageMetadata, VersionInfo, PublishRequest, PackageStats, PublishResponse.

use hudhudscript_package::registry::{PackageStats, PublishRequest, PublishResponse, VersionInfo};
use hudhudscript_package::{PackageMetadata, RegistryClient};
use std::collections::HashMap;

#[test]
fn test_registry_client_creation() {
    let client = RegistryClient::new("https://registry.hudhudscript.org");
    assert!(client.is_ok());
}

#[test]
fn test_registry_client_base_url() {
    let client = RegistryClient::new("https://custom.registry.com").unwrap();
    assert_eq!(client.base_url(), "https://custom.registry.com");
}

#[test]
fn test_registry_client_has_http_client() {
    let client = RegistryClient::new("https://registry.example.com").unwrap();
    // Verify we can get a reference to the HTTP client
    let _http_client: &reqwest::Client = client.client();
}

#[test]
fn test_package_metadata_serialization() {
    let meta = PackageMetadata {
        name: "test-pkg".to_string(),
        description: "A test package".to_string(),
        latest_version: "1.0.0".to_string(),
        versions: vec!["0.1.0".to_string(), "1.0.0".to_string()],
        authors: vec!["author".to_string()],
        license: "MIT".to_string(),
        repository: Some("https://github.com/example/test".to_string()),
        homepage: None,
        keywords: vec!["test".to_string()],
        categories: vec!["tools".to_string()],
        downloads: 100,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: PackageMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "test-pkg");
    assert_eq!(deserialized.versions.len(), 2);
    assert_eq!(deserialized.downloads, 100);
    assert_eq!(deserialized.homepage, None);
    assert_eq!(
        deserialized.repository,
        Some("https://github.com/example/test".to_string())
    );
}

#[test]
fn test_version_info_serialization() {
    let info = VersionInfo {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        description: "desc".to_string(),
        dependencies: {
            let mut m = std::collections::HashMap::new();
            m.insert("dep-a".to_string(), "^1.0".to_string());
            m
        },
        dev_dependencies: std::collections::HashMap::new(),
        checksum: "abc123".to_string(),
        size: 4096,
        published_at: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: VersionInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "pkg");
    assert_eq!(deserialized.size, 4096);
    assert_eq!(deserialized.dependencies.len(), 1);
    assert_eq!(deserialized.dev_dependencies.len(), 0);
}

#[test]
fn test_publish_request_serialization() {
    let req = PublishRequest {
        name: "my-pkg".to_string(),
        version: "0.1.0".to_string(),
        description: "test".to_string(),
        authors: vec!["alice".to_string()],
        license: "MIT".to_string(),
        repository: None,
        homepage: None,
        keywords: vec![],
        categories: vec![],
        dependencies: std::collections::HashMap::new(),
        tarball: vec![1, 2, 3],
        checksum: "deadbeef".to_string(),
    };

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: PublishRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "my-pkg");
    assert_eq!(deserialized.tarball, vec![1, 2, 3]);
}

#[test]
fn test_package_stats_serialization() {
    let stats = PackageStats {
        downloads_total: 1000,
        downloads_last_week: 50,
        downloads_last_month: 200,
        dependents: 5,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: PackageStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.downloads_total, 1000);
    assert_eq!(deserialized.downloads_last_week, 50);
    assert_eq!(deserialized.downloads_last_month, 200);
    assert_eq!(deserialized.dependents, 5);
}

#[test]
fn test_publish_response_serialization() {
    let resp = PublishResponse {
        success: true,
        message: "Published".to_string(),
        package_url: "https://registry.example.com/pkg/1.0.0".to_string(),
    };
    let json = serde_json::to_string(&resp).unwrap();
    let deserialized: PublishResponse = serde_json::from_str(&json).unwrap();
    assert!(deserialized.success);
    assert_eq!(deserialized.message, "Published");
    assert!(deserialized.package_url.contains("pkg"));
}

#[test]
fn test_publish_response_failure() {
    let resp = PublishResponse {
        success: false,
        message: "Version already exists".to_string(),
        package_url: String::new(),
    };
    assert!(!resp.success);
    assert!(resp.message.contains("already exists"));
}

#[test]
fn test_version_info_with_dev_dependencies() {
    let info = VersionInfo {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        description: "desc".to_string(),
        dependencies: HashMap::new(),
        dev_dependencies: {
            let mut m = HashMap::new();
            m.insert("test-framework".to_string(), "^1.0".to_string());
            m.insert("mock-server".to_string(), "^0.5".to_string());
            m
        },
        checksum: "abc".to_string(),
        size: 2048,
        published_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&info).unwrap();
    let deserialized: VersionInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.dev_dependencies.len(), 2);
}

#[test]
fn test_package_metadata_all_fields() {
    let meta = PackageMetadata {
        name: "full-pkg".to_string(),
        description: "A full package".to_string(),
        latest_version: "2.0.0".to_string(),
        versions: vec!["1.0.0".to_string(), "2.0.0".to_string()],
        authors: vec!["alice".to_string(), "bob".to_string()],
        license: "Apache-2.0".to_string(),
        repository: None,
        homepage: Some("https://example.com".to_string()),
        keywords: vec!["ai".to_string(), "ml".to_string()],
        categories: vec!["machine-learning".to_string()],
        downloads: 0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: PackageMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.authors.len(), 2);
    assert_eq!(deserialized.keywords.len(), 2);
    assert_eq!(
        deserialized.homepage,
        Some("https://example.com".to_string())
    );
    assert!(deserialized.repository.is_none());
}

#[test]
fn test_registry_client_clone() {
    let client = RegistryClient::new("https://example.com").unwrap();
    let cloned = client.clone();
    assert_eq!(cloned.base_url(), "https://example.com");
}

#[test]
fn test_package_stats_zero_values() {
    let stats = PackageStats {
        downloads_total: 0,
        downloads_last_week: 0,
        downloads_last_month: 0,
        dependents: 0,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: PackageStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.downloads_total, 0);
}
