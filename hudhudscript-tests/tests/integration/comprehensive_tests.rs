//! Comprehensive tests for Resource Manager
//! Target: 100% code coverage

use hudhudscript_mcp::client::McpClient;
use hudhudscript_mcp::transport::TransportConfig;
use hudhudscript_resources::{
    CachedResource, ResourceContent, ResourceManager, ResourceMetadata, ResourceSchema,
};
use std::sync::Arc;
use std::time::Duration;

async fn create_test_client() -> Arc<McpClient> {
    Arc::new(
        McpClient::new(TransportConfig::stdio("echo", vec![]))
            .await
            .unwrap(),
    )
}

#[tokio::test]
async fn test_resource_metadata_creation() {
    let metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        Some("Test file".to_string()),
        Some("text/plain".to_string()),
    );

    assert_eq!(metadata.uri, "file:///test.txt");
    assert_eq!(metadata.name, "test.txt");
    assert_eq!(metadata.server, "server1");
    assert_eq!(metadata.access_count, 0);
}

#[tokio::test]
async fn test_resource_metadata_record_access() {
    let mut metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        None,
        None,
    );

    metadata.record_access();
    assert_eq!(metadata.access_count, 1);
    assert!(metadata.last_accessed.is_some());

    metadata.record_access();
    assert_eq!(metadata.access_count, 2);
}

#[tokio::test]
async fn test_resource_metadata_add_tag() {
    let mut metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        None,
        None,
    );

    metadata.add_tag("important".to_string());
    metadata.add_tag("data".to_string());
    metadata.add_tag("important".to_string()); // Duplicate

    assert_eq!(metadata.tags.len(), 2);
    assert!(metadata.tags.contains(&"important".to_string()));
}

#[tokio::test]
async fn test_resource_metadata_update_etag() {
    let mut metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        None,
        None,
    );

    assert!(metadata.etag.is_none());

    metadata.update_etag("abc123".to_string());
    assert_eq!(metadata.etag, Some("abc123".to_string()));

    metadata.update_etag("def456".to_string());
    assert_eq!(metadata.etag, Some("def456".to_string()));
}

#[tokio::test]
async fn test_resource_content_text() {
    let content = ResourceContent::Text("Hello, World!".to_string());

    assert!(content.is_text());
    assert!(!content.is_binary());
    assert_eq!(content.as_text(), Some("Hello, World!"));
    assert_eq!(content.as_binary(), None);
}

#[tokio::test]
async fn test_resource_content_binary() {
    let content = ResourceContent::Binary("SGVsbG8gV29ybGQh".to_string());

    assert!(content.is_binary());
    assert!(!content.is_text());
    assert_eq!(content.as_binary(), Some("SGVsbG8gV29ybGQh"));
    assert_eq!(content.as_text(), None);
}

#[tokio::test]
async fn test_cached_resource_creation() {
    let metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        None,
        None,
    );

    let content = ResourceContent::Text("test".to_string());
    let cached = CachedResource::new(metadata, content, Some("etag123".to_string()));

    assert_eq!(cached.etag, Some("etag123".to_string()));
}

#[tokio::test]
async fn test_cached_resource_validity() {
    let metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        None,
        None,
    );

    let content = ResourceContent::Text("test".to_string());
    let cached = CachedResource::new(metadata, content, None);

    // Should be valid immediately
    assert!(cached.is_valid(Duration::from_secs(60)));

    // Should be invalid with 0 TTL
    assert!(!cached.is_valid(Duration::from_secs(0)));
}

#[tokio::test]
async fn test_resource_manager_creation() {
    let client = create_test_client().await;
    let manager = ResourceManager::new(client, Duration::from_secs(300));

    let stats = manager.cache_stats();
    assert_eq!(stats.size, 0);
    assert_eq!(stats.ttl, Duration::from_secs(300));
}

#[tokio::test]
async fn test_resource_manager_list_empty() {
    let client = create_test_client().await;
    let manager = ResourceManager::new(client, Duration::from_secs(300));

    let resources = manager.list_resources();
    assert_eq!(resources.len(), 0);
}

#[tokio::test]
async fn test_resource_manager_search_empty() {
    let client = create_test_client().await;
    let manager = ResourceManager::new(client, Duration::from_secs(300));

    let results = manager.search_resources("test");
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_resource_manager_clear_cache() {
    let client = create_test_client().await;
    let manager = ResourceManager::new(client, Duration::from_secs(300));

    manager.clear_cache();

    let stats = manager.cache_stats();
    assert_eq!(stats.size, 0);
}

#[tokio::test]
async fn test_resource_schema_creation() {
    let schema = ResourceSchema {
        uri: "file:///data.json".to_string(),
        name: "data.json".to_string(),
        description: Some("JSON data".to_string()),
        mime_type: Some("application/json".to_string()),
        server: "filesystem".to_string(),
    };

    assert_eq!(schema.uri, "file:///data.json");
    assert_eq!(schema.name, "data.json");
    assert_eq!(schema.server, "filesystem");
}

#[tokio::test]
async fn test_resource_schema_serialization() {
    let schema = ResourceSchema {
        uri: "file:///test.txt".to_string(),
        name: "test.txt".to_string(),
        description: None,
        mime_type: Some("text/plain".to_string()),
        server: "server1".to_string(),
    };

    let json = serde_json::to_string(&schema).unwrap();
    let deserialized: ResourceSchema = serde_json::from_str(&json).unwrap();

    assert_eq!(schema.uri, deserialized.uri);
    assert_eq!(schema.name, deserialized.name);
}

#[tokio::test]
async fn test_resource_content_serialization() {
    let text_content = ResourceContent::Text("Hello".to_string());
    let json = serde_json::to_string(&text_content).unwrap();
    let deserialized: ResourceContent = serde_json::from_str(&json).unwrap();
    assert_eq!(text_content, deserialized);

    let binary_content = ResourceContent::Binary("data".to_string());
    let json = serde_json::to_string(&binary_content).unwrap();
    let deserialized: ResourceContent = serde_json::from_str(&json).unwrap();
    assert_eq!(binary_content, deserialized);
}

#[tokio::test]
async fn test_resource_metadata_serialization() {
    let metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        Some("Test".to_string()),
        Some("text/plain".to_string()),
    );

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ResourceMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.uri, deserialized.uri);
    assert_eq!(metadata.name, deserialized.name);
}

#[tokio::test]
async fn test_cache_stats() {
    let client = create_test_client().await;
    let manager = ResourceManager::new(client, Duration::from_secs(600));

    let stats = manager.cache_stats();
    assert_eq!(stats.size, 0);
    assert_eq!(stats.ttl, Duration::from_secs(600));
}

#[tokio::test]
async fn test_resource_metadata_with_all_fields() {
    let metadata = ResourceMetadata::new(
        "file:///complete.txt".to_string(),
        "complete.txt".to_string(),
        "server1".to_string(),
        Some("Complete resource".to_string()),
        Some("text/plain".to_string()),
    );

    assert!(metadata.description.is_some());
    assert!(metadata.mime_type.is_some());
    assert_eq!(metadata.access_count, 0);
    assert!(metadata.last_accessed.is_none());
    assert_eq!(metadata.tags.len(), 0);
    assert!(metadata.etag.is_none());
}

#[tokio::test]
async fn test_cached_resource_with_etag() {
    let metadata = ResourceMetadata::new(
        "file:///test.txt".to_string(),
        "test.txt".to_string(),
        "server1".to_string(),
        None,
        None,
    );

    let content = ResourceContent::Text("content".to_string());
    let etag = Some("etag-12345".to_string());
    let cached = CachedResource::new(metadata, content, etag.clone());

    assert_eq!(cached.etag, etag);
}

#[tokio::test]
async fn test_resource_content_variants() {
    let text = ResourceContent::Text("text".to_string());
    let binary = ResourceContent::Binary("binary".to_string());

    assert!(matches!(text, ResourceContent::Text(_)));
    assert!(matches!(binary, ResourceContent::Binary(_)));
}
