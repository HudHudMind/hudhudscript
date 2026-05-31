//! Tests for ResourceSchema, ResourceMetadata, CachedResource, ResourceContent.

use hudhudscript_resources::{
    CachedResource, ResourceContent, ResourceMetadata, ResourceSchema,
};
use std::time::{Duration, SystemTime};

fn make_metadata(uri: &str, name: &str) -> ResourceMetadata {
    ResourceMetadata {
        uri: uri.into(),
        name: name.into(),
        description: None,
        mime_type: Some("text/plain".into()),
        server: "test".into(),
        discovered_at: SystemTime::now(),
        last_accessed: None,
        access_count: 0,
        tags: vec![],
        etag: None,
    }
}

#[test]
fn resource_schema_construction() {
    let schema = ResourceSchema {
        uri: "file:///data/config.json".into(),
        name: "Config".into(),
        description: Some("Application configuration".into()),
        mime_type: Some("application/json".into()),
        server: "local-fs".into(),
    };
    assert_eq!(schema.uri, "file:///data/config.json");
    assert_eq!(schema.name, "Config");
    assert_eq!(schema.description, Some("Application configuration".into()));
    assert_eq!(schema.mime_type, Some("application/json".into()));
    assert_eq!(schema.server, "local-fs");
}

#[test]
fn resource_schema_minimal() {
    let schema = ResourceSchema {
        uri: "memory://cache".into(),
        name: "Cache".into(),
        description: None,
        mime_type: None,
        server: "memory".into(),
    };
    assert!(schema.description.is_none());
    assert!(schema.mime_type.is_none());
}

#[test]
fn resource_metadata_tracks_access() {
    let now = SystemTime::now();
    let meta = ResourceMetadata {
        uri: "file:///data/report.txt".into(),
        name: "Report".into(),
        description: None,
        mime_type: Some("text/plain".into()),
        server: "local-fs".into(),
        discovered_at: now,
        last_accessed: None,
        access_count: 0,
        tags: vec![],
        etag: None,
    };
    assert_eq!(meta.access_count, 0);
    assert!(meta.last_accessed.is_none());
}

#[test]
fn resource_metadata_with_access() {
    let now = SystemTime::now();
    let accessed = now + Duration::from_secs(60);
    let meta = ResourceMetadata {
        uri: "http://api/data".into(),
        name: "API Data".into(),
        description: Some("Remote data".into()),
        mime_type: Some("application/json".into()),
        server: "http-api".into(),
        discovered_at: now,
        last_accessed: Some(accessed),
        access_count: 42,
        tags: vec!["api".into()],
        etag: Some("abc123".into()),
    };
    assert_eq!(meta.access_count, 42);
    assert!(meta.last_accessed.is_some());
    assert_eq!(meta.tags.len(), 1);
    assert_eq!(meta.etag, Some("abc123".into()));
}

#[test]
fn resource_content_text() {
    let content = ResourceContent::Text("Hello, World!".into());
    match &content {
        ResourceContent::Text(t) => assert_eq!(t, "Hello, World!"),
        _ => panic!("expected Text variant"),
    }
    assert_eq!(content.as_text(), Some("Hello, World!"));
}

#[test]
fn resource_content_binary() {
    let content = ResourceContent::Binary("aGVsbG8=".into());
    match &content {
        ResourceContent::Binary(b) => assert_eq!(b, "aGVsbG8="),
        _ => panic!("expected Binary variant"),
    }
    assert!(content.as_text().is_none());
    assert_eq!(content.as_binary(), Some("aGVsbG8="));
}

#[test]
fn cached_resource_basic() {
    let now = SystemTime::now();
    let cached = CachedResource {
        metadata: make_metadata("file:///test.txt", "Test"),
        content: ResourceContent::Text("cached data".into()),
        cached_at: now,
        etag: None,
    };
    assert_eq!(cached.metadata.uri, "file:///test.txt");
    assert!(cached.is_valid(Duration::from_secs(300)));
}

#[test]
fn cached_resource_expired() {
    let old_time = SystemTime::now() - Duration::from_secs(600);
    let cached = CachedResource {
        metadata: make_metadata("file:///old.txt", "Old"),
        content: ResourceContent::Text("stale".into()),
        cached_at: old_time,
        etag: None,
    };
    assert!(!cached.is_valid(Duration::from_secs(300)));
}

#[test]
fn cached_resource_with_etag() {
    let now = SystemTime::now();
    let cached = CachedResource {
        metadata: make_metadata("file:///etag.txt", "ETag"),
        content: ResourceContent::Text("versioned".into()),
        cached_at: now,
        etag: Some("v1.0".into()),
    };
    assert_eq!(cached.etag, Some("v1.0".into()));
}

#[test]
fn resource_schema_serialize_roundtrip() {
    let schema = ResourceSchema {
        uri: "test://uri".into(),
        name: "TestResource".into(),
        description: Some("A test resource".into()),
        mime_type: Some("application/json".into()),
        server: "test-server".into(),
    };
    let json = serde_json::to_string(&schema).unwrap();
    let deser: ResourceSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(schema.uri, deser.uri);
    assert_eq!(schema.name, deser.name);
}

#[test]
fn resource_metadata_serialize_roundtrip() {
    let meta = make_metadata("test://meta", "Meta");
    let json = serde_json::to_string(&meta).unwrap();
    let deser: ResourceMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta.uri, deser.uri);
    assert_eq!(meta.access_count, deser.access_count);
}
