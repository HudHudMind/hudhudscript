use hudhudscript_tools::registry::{CacheStats, RegistryError, ToolCache};
use hudhudscript_tools::schema::{JsonSchema, ToolSchema, ValidationError};
use std::time::Duration;

#[test]
fn test_tool_cache_put_and_get() {
    let mut cache = ToolCache::new(Duration::from_secs(60));
    let schema = ToolSchema {
        name: "test".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "srv".to_string(),
    };
    cache.put(schema.clone());
    assert_eq!(cache.size(), 1);
    let retrieved = cache.get("test");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test");
}

#[test]
fn test_tool_cache_get_nonexistent() {
    let cache = ToolCache::new(Duration::from_secs(60));
    assert!(cache.get("nonexistent").is_none());
}

#[test]
fn test_tool_cache_clear() {
    let mut cache = ToolCache::new(Duration::from_secs(60));
    let schema = ToolSchema {
        name: "test".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "srv".to_string(),
    };
    cache.put(schema);
    assert_eq!(cache.size(), 1);
    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_tool_cache_cleanup_retains_fresh_entries() {
    let mut cache = ToolCache::new(Duration::from_secs(3600));
    let schema = ToolSchema {
        name: "fresh".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "srv".to_string(),
    };
    cache.put(schema);
    cache.cleanup();
    // Entry was just added, so cleanup should keep it
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_registry_error_display() {
    let err = RegistryError::ToolNotFound("my_tool".to_string());
    assert!(err.to_string().contains("Tool not found: my_tool"));

    let err = RegistryError::ServerNotFound("my_server".to_string());
    assert!(err.to_string().contains("Server not found: my_server"));

    let err = RegistryError::DiscoveryFailed("timeout".to_string());
    assert!(err.to_string().contains("Tool discovery failed: timeout"));

    let err = RegistryError::CallFailed("connection refused".to_string());
    assert!(err
        .to_string()
        .contains("Tool call failed: connection refused"));
}

#[test]
fn test_registry_error_validation_failed() {
    let validation_err = ValidationError::MissingRequired("field".to_string());
    let err = RegistryError::ValidationFailed(validation_err);
    assert!(err.to_string().contains("Missing required field"));
}

#[test]
fn test_cache_stats_fields() {
    let stats = CacheStats {
        size: 42,
        ttl: Duration::from_secs(120),
    };
    assert_eq!(stats.size, 42);
    assert_eq!(stats.ttl, Duration::from_secs(120));
}

// ---- ToolCache with very short TTL ----

#[test]
fn test_tool_cache_expired_entry() {
    let mut cache = ToolCache::new(Duration::from_nanos(1)); // virtually instant expiry
    let schema = ToolSchema {
        name: "ephemeral".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "srv".to_string(),
    };
    cache.put(schema);
    // Sleep briefly to ensure TTL expires
    std::thread::sleep(Duration::from_millis(1));
    // Entry should be expired
    assert!(cache.get("ephemeral").is_none());
}

// ---- ToolCache cleanup removes expired ----

#[test]
fn test_tool_cache_cleanup_removes_expired() {
    let mut cache = ToolCache::new(Duration::from_nanos(1));
    let schema = ToolSchema {
        name: "expired".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "srv".to_string(),
    };
    cache.put(schema);
    std::thread::sleep(Duration::from_millis(1));
    cache.cleanup();
    assert_eq!(cache.size(), 0);
}

// ---- ToolCache put overwrites existing ----

#[test]
fn test_tool_cache_put_overwrites() {
    let mut cache = ToolCache::new(Duration::from_secs(60));
    let schema1 = ToolSchema {
        name: "tool".to_string(),
        description: Some("first".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "srv1".to_string(),
    };
    cache.put(schema1);

    let schema2 = ToolSchema {
        name: "tool".to_string(),
        description: Some("second".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "srv2".to_string(),
    };
    cache.put(schema2);
    assert_eq!(cache.size(), 1);
    let retrieved = cache.get("tool").unwrap();
    assert_eq!(retrieved.server, "srv2");
}

// ---- CacheStats debug ----

#[test]
fn test_cache_stats_debug() {
    let stats = CacheStats {
        size: 5,
        ttl: Duration::from_secs(60),
    };
    let debug = format!("{:?}", stats);
    assert!(debug.contains("CacheStats"));
    assert!(debug.contains("5"));
}

// ---- CacheStats clone ----

#[test]
fn test_cache_stats_clone() {
    let stats = CacheStats {
        size: 10,
        ttl: Duration::from_secs(300),
    };
    let cloned = stats.clone();
    assert_eq!(cloned.size, 10);
    assert_eq!(cloned.ttl, Duration::from_secs(300));
}

// ---- RegistryError clone ----

#[test]
fn test_registry_error_clone() {
    let err = RegistryError::ToolNotFound("test".to_string());
    let cloned = err.clone();
    assert!(cloned.to_string().contains("Tool not found: test"));
}

// ---- Multiple cleanup on empty cache ----

#[test]
fn test_tool_cache_cleanup_empty() {
    let mut cache = ToolCache::new(Duration::from_secs(60));
    cache.cleanup();
    assert_eq!(cache.size(), 0);
}
