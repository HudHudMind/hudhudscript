use hudhudscript_tools::registry::{CacheStats, RegistryError, ToolCache, ToolRegistry};
use hudhudscript_tools::schema::{
    JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema, ValidationError,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

// ===============================================================================
// Helper: build a ToolSchema
// ===============================================================================

fn make_schema(name: &str, server: &str) -> ToolSchema {
    ToolSchema {
        name: name.to_string(),
        description: Some(format!("{} description", name)),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: server.to_string(),
    }
}

fn make_metadata(name: &str, server: &str) -> ToolMetadata {
    ToolMetadata::new(
        name.to_string(),
        server.to_string(),
        Some(format!("{} desc", name)),
    )
}

// ===============================================================================
// ToolRegistry -- construction
// ===============================================================================

#[test]
fn registry_new_empty() {
    let reg = ToolRegistry::new();
    assert!(reg.list_tools().is_empty());
}

#[test]
fn registry_default_empty() {
    let reg = ToolRegistry::default();
    assert!(reg.list_tools().is_empty());
}

// ===============================================================================
// ToolRegistry -- register_tool & get_tool
// ===============================================================================

#[test]
fn register_and_get_tool() {
    let reg = ToolRegistry::new();
    let schema = make_schema("search", "srv1");
    let meta = make_metadata("search", "srv1");
    reg.register_tool(schema, meta).unwrap();

    let found = reg.get_tool("search");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "search");
}

#[test]
fn register_multiple_tools() {
    let reg = ToolRegistry::new();
    for i in 0..5 {
        let name = format!("tool_{}", i);
        reg.register_tool(make_schema(&name, "srv"), make_metadata(&name, "srv"))
            .unwrap();
    }
    assert_eq!(reg.list_tools().len(), 5);
}

#[test]
fn get_tool_nonexistent_returns_none() {
    let reg = ToolRegistry::new();
    assert!(reg.get_tool("missing").is_none());
}

#[test]
fn register_tool_overwrites_existing() {
    let reg = ToolRegistry::new();
    reg.register_tool(make_schema("t1", "s1"), make_metadata("t1", "s1"))
        .unwrap();
    reg.register_tool(make_schema("t1", "s2"), make_metadata("t1", "s2"))
        .unwrap();
    let tool = reg.get_tool("t1").unwrap();
    assert_eq!(tool.server, "s2");
}

#[test]
fn register_tool_with_no_description() {
    let reg = ToolRegistry::new();
    let schema = ToolSchema {
        name: "nodesc".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "s1".to_string(),
    };
    let meta = ToolMetadata::new("nodesc".to_string(), "s1".to_string(), None);
    reg.register_tool(schema, meta).unwrap();
    let t = reg.get_tool("nodesc").unwrap();
    assert!(t.description.is_none());
}

// ===============================================================================
// ToolRegistry -- get_metadata
// ===============================================================================

#[test]
fn get_metadata_exists() {
    let reg = ToolRegistry::new();
    reg.register_tool(make_schema("t1", "s1"), make_metadata("t1", "s1"))
        .unwrap();
    let meta = reg.get_metadata("t1");
    assert!(meta.is_some());
    assert_eq!(meta.unwrap().name, "t1");
}

#[test]
fn get_metadata_nonexistent() {
    let reg = ToolRegistry::new();
    assert!(reg.get_metadata("missing").is_none());
}

#[test]
fn get_metadata_server_field() {
    let reg = ToolRegistry::new();
    reg.register_tool(
        make_schema("t1", "myserver"),
        make_metadata("t1", "myserver"),
    )
    .unwrap();
    let meta = reg.get_metadata("t1").unwrap();
    assert_eq!(meta.server, "myserver");
}

// ===============================================================================
// ToolRegistry -- list_tools
// ===============================================================================

#[test]
fn list_tools_returns_names() {
    let reg = ToolRegistry::new();
    reg.register_tool(make_schema("alpha", "s"), make_metadata("alpha", "s"))
        .unwrap();
    reg.register_tool(make_schema("beta", "s"), make_metadata("beta", "s"))
        .unwrap();
    let tools = reg.list_tools();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"alpha".to_string()));
    assert!(tools.contains(&"beta".to_string()));
}

#[test]
fn list_tools_empty_registry() {
    let reg = ToolRegistry::new();
    assert!(reg.list_tools().is_empty());
}

// ===============================================================================
// ToolRegistry -- validate_arguments
// ===============================================================================

#[test]
fn validate_arguments_unknown_tool() {
    let reg = ToolRegistry::new();
    let result = reg.validate_arguments("nonexistent", &json!({}));
    assert!(result.is_err());
}

#[test]
fn validate_arguments_valid_object() {
    let reg = ToolRegistry::new();
    let schema = make_schema("t1", "s1");
    reg.register_tool(schema, make_metadata("t1", "s1"))
        .unwrap();
    assert!(reg.validate_arguments("t1", &json!({})).is_ok());
}

#[test]
fn validate_arguments_required_missing() {
    let reg = ToolRegistry::new();
    let schema = ToolSchema {
        name: "t1".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some({
                let mut p = HashMap::new();
                p.insert(
                    "name".to_string(),
                    JsonSchemaProperty {
                        property_type: "string".to_string(),
                        description: None,
                        default: None,
                        enum_values: None,
                    },
                );
                p
            }),
            required: Some(vec!["name".to_string()]),
            items: None,
            description: None,
        },
        server: "s1".to_string(),
    };
    reg.register_tool(schema, make_metadata("t1", "s1"))
        .unwrap();
    assert!(reg.validate_arguments("t1", &json!({})).is_err());
    assert!(reg
        .validate_arguments("t1", &json!({"name": "test"}))
        .is_ok());
}

#[test]
fn validate_arguments_type_mismatch() {
    let reg = ToolRegistry::new();
    let schema = ToolSchema {
        name: "t1".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some({
                let mut p = HashMap::new();
                p.insert(
                    "count".to_string(),
                    JsonSchemaProperty {
                        property_type: "integer".to_string(),
                        description: None,
                        default: None,
                        enum_values: None,
                    },
                );
                p
            }),
            required: None,
            items: None,
            description: None,
        },
        server: "s1".to_string(),
    };
    reg.register_tool(schema, make_metadata("t1", "s1"))
        .unwrap();
    assert!(reg
        .validate_arguments("t1", &json!({"count": "not a number"}))
        .is_err());
}

#[test]
fn validate_arguments_extra_properties_ok() {
    let reg = ToolRegistry::new();
    reg.register_tool(make_schema("t1", "s1"), make_metadata("t1", "s1"))
        .unwrap();
    assert!(reg
        .validate_arguments("t1", &json!({"extra": "field"}))
        .is_ok());
}

// ===============================================================================
// ToolRegistry -- cache operations
// ===============================================================================

#[test]
fn clear_cache_empties_cache() {
    let reg = ToolRegistry::new();
    reg.register_tool(make_schema("t1", "s1"), make_metadata("t1", "s1"))
        .unwrap();
    let stats_before = reg.cache_stats();
    assert_eq!(stats_before.size, 1);

    reg.clear_cache();
    let stats_after = reg.cache_stats();
    assert_eq!(stats_after.size, 0);
}

#[test]
fn cache_stats_initial() {
    let reg = ToolRegistry::new();
    let stats = reg.cache_stats();
    assert_eq!(stats.size, 0);
    assert_eq!(stats.ttl, Duration::from_secs(300));
}

#[test]
fn cache_stats_after_register() {
    let reg = ToolRegistry::new();
    for i in 0..3 {
        let n = format!("t{}", i);
        reg.register_tool(make_schema(&n, "s"), make_metadata(&n, "s"))
            .unwrap();
    }
    assert_eq!(reg.cache_stats().size, 3);
}

#[test]
fn get_tool_after_cache_clear_falls_back_to_registry() {
    let reg = ToolRegistry::new();
    reg.register_tool(make_schema("t1", "s1"), make_metadata("t1", "s1"))
        .unwrap();
    reg.clear_cache();
    // Should still find it from the registry (not cache)
    let t = reg.get_tool("t1");
    assert!(t.is_some());
    assert_eq!(t.unwrap().name, "t1");
}

// ===============================================================================
// ToolCache -- direct tests
// ===============================================================================

#[test]
fn tool_cache_new() {
    let cache = ToolCache::new(Duration::from_secs(60));
    assert_eq!(cache.size(), 0);
}

#[test]
fn tool_cache_put_and_get() {
    let mut cache = ToolCache::new(Duration::from_secs(300));
    cache.put(make_schema("t1", "s1"));
    assert_eq!(cache.size(), 1);
    let fetched = cache.get("t1");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "t1");
}

#[test]
fn tool_cache_get_missing() {
    let cache = ToolCache::new(Duration::from_secs(300));
    assert!(cache.get("missing").is_none());
}

#[test]
fn tool_cache_clear() {
    let mut cache = ToolCache::new(Duration::from_secs(300));
    cache.put(make_schema("t1", "s1"));
    cache.put(make_schema("t2", "s1"));
    assert_eq!(cache.size(), 2);
    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn tool_cache_overwrite_same_key() {
    let mut cache = ToolCache::new(Duration::from_secs(300));
    cache.put(make_schema("t1", "s1"));
    cache.put(make_schema("t1", "s2"));
    assert_eq!(cache.size(), 1);
    let fetched = cache.get("t1").unwrap();
    assert_eq!(fetched.server, "s2");
}

#[test]
fn tool_cache_cleanup_removes_nothing_when_fresh() {
    let mut cache = ToolCache::new(Duration::from_secs(300));
    cache.put(make_schema("t1", "s1"));
    cache.cleanup();
    assert_eq!(cache.size(), 1);
}

#[test]
fn tool_cache_multiple_items() {
    let mut cache = ToolCache::new(Duration::from_secs(300));
    for i in 0..10 {
        cache.put(make_schema(&format!("t{}", i), "s"));
    }
    assert_eq!(cache.size(), 10);
    for i in 0..10 {
        assert!(cache.get(&format!("t{}", i)).is_some());
    }
}

// ===============================================================================
// CacheStats
// ===============================================================================

#[test]
fn cache_stats_debug() {
    let stats = CacheStats {
        size: 5,
        ttl: Duration::from_secs(120),
    };
    let dbg = format!("{:?}", stats);
    assert!(dbg.contains("5"));
}

#[test]
fn cache_stats_clone() {
    let stats = CacheStats {
        size: 3,
        ttl: Duration::from_secs(60),
    };
    let cloned = stats.clone();
    assert_eq!(cloned.size, 3);
    assert_eq!(cloned.ttl, Duration::from_secs(60));
}

// ===============================================================================
// RegistryError
// ===============================================================================

#[test]
fn registry_error_tool_not_found_display() {
    let e = RegistryError::ToolNotFound("missing_tool".to_string());
    assert!(e.to_string().contains("missing_tool"));
}

#[test]
fn registry_error_server_not_found_display() {
    let e = RegistryError::ServerNotFound("unknown_server".to_string());
    assert!(e.to_string().contains("unknown_server"));
}

#[test]
fn registry_error_discovery_failed_display() {
    let e = RegistryError::DiscoveryFailed("connection refused".to_string());
    assert!(e.to_string().contains("connection refused"));
}

#[test]
fn registry_error_call_failed_display() {
    let e = RegistryError::CallFailed("timeout".to_string());
    assert!(e.to_string().contains("timeout"));
}

#[test]
fn registry_error_clone() {
    let e = RegistryError::ToolNotFound("x".to_string());
    let cloned = e.clone();
    assert_eq!(e.to_string(), cloned.to_string());
}

#[test]
fn registry_error_debug() {
    let e = RegistryError::ToolNotFound("debug_test".to_string());
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("ToolNotFound"));
}

// ===============================================================================
// ToolMetadata
// ===============================================================================

#[test]
fn tool_metadata_new_fields() {
    let meta = ToolMetadata::new(
        "tool1".to_string(),
        "server1".to_string(),
        Some("desc".to_string()),
    );
    assert_eq!(meta.name, "tool1");
    assert_eq!(meta.server, "server1");
    assert_eq!(meta.usage_count, 0);
    assert!(meta.last_used.is_none());
    assert!(meta.tags.is_empty());
}

#[test]
fn tool_metadata_new_no_description() {
    let meta = ToolMetadata::new("t".to_string(), "s".to_string(), None);
    assert!(meta.description.is_none());
}

#[test]
fn tool_metadata_record_usage() {
    let mut meta = ToolMetadata::new("t".to_string(), "s".to_string(), None);
    meta.record_usage();
    assert_eq!(meta.usage_count, 1);
    assert!(meta.last_used.is_some());
    meta.record_usage();
    assert_eq!(meta.usage_count, 2);
}

#[test]
fn tool_metadata_add_tag() {
    let mut meta = ToolMetadata::new("t".to_string(), "s".to_string(), None);
    meta.add_tag("io".to_string());
    assert_eq!(meta.tags, vec!["io"]);
    meta.add_tag("io".to_string()); // duplicate
    assert_eq!(meta.tags.len(), 1);
    meta.add_tag("net".to_string());
    assert_eq!(meta.tags.len(), 2);
}

#[test]
fn tool_metadata_multiple_tags() {
    let mut meta = ToolMetadata::new("t".to_string(), "s".to_string(), None);
    meta.add_tag("a".to_string());
    meta.add_tag("b".to_string());
    meta.add_tag("c".to_string());
    assert_eq!(meta.tags.len(), 3);
}

// ===============================================================================
// JsonSchema validation
// ===============================================================================

#[test]
fn json_schema_validate_string_ok() {
    let schema = JsonSchema {
        schema_type: "string".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!("hello")).is_ok());
}

#[test]
fn json_schema_validate_string_fail() {
    let schema = JsonSchema {
        schema_type: "string".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!(42)).is_err());
}

#[test]
fn json_schema_validate_number_ok() {
    let schema = JsonSchema {
        schema_type: "number".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!(3.14)).is_ok());
}

#[test]
fn json_schema_validate_integer_ok() {
    let schema = JsonSchema {
        schema_type: "integer".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!(42)).is_ok());
}

#[test]
fn json_schema_validate_integer_fail_float() {
    let schema = JsonSchema {
        schema_type: "integer".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!(3.14)).is_err());
}

#[test]
fn json_schema_validate_boolean_ok() {
    let schema = JsonSchema {
        schema_type: "boolean".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!(true)).is_ok());
    assert!(schema.validate(&json!(false)).is_ok());
}

#[test]
fn json_schema_validate_null_ok() {
    let schema = JsonSchema {
        schema_type: "null".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!(null)).is_ok());
}

#[test]
fn json_schema_validate_array_ok() {
    let schema = JsonSchema {
        schema_type: "array".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!([1, 2, 3])).is_ok());
}

#[test]
fn json_schema_validate_array_with_items() {
    let items_schema = JsonSchema {
        schema_type: "string".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    let schema = JsonSchema {
        schema_type: "array".to_string(),
        properties: None,
        required: None,
        items: Some(Box::new(items_schema)),
        description: None,
    };
    assert!(schema.validate(&json!(["a", "b"])).is_ok());
    assert!(schema.validate(&json!([1, 2])).is_err());
}

#[test]
fn json_schema_validate_unknown_type() {
    let schema = JsonSchema {
        schema_type: "custom".to_string(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(schema.validate(&json!("anything")).is_err());
}

// ===============================================================================
// ValidationError
// ===============================================================================

#[test]
fn validation_error_type_mismatch_display() {
    let e = ValidationError::TypeMismatch {
        expected: "string".to_string(),
        found: "number".to_string(),
    };
    assert!(e.to_string().contains("string"));
    assert!(e.to_string().contains("number"));
}

#[test]
fn validation_error_missing_required_display() {
    let e = ValidationError::MissingRequired("name".to_string());
    assert!(e.to_string().contains("name"));
}

#[test]
fn validation_error_unknown_type_display() {
    let e = ValidationError::UnknownType("foobar".to_string());
    assert!(e.to_string().contains("foobar"));
}

#[test]
fn validation_error_clone() {
    let e = ValidationError::TypeMismatch {
        expected: "a".to_string(),
        found: "b".to_string(),
    };
    let cloned = e.clone();
    assert_eq!(e.to_string(), cloned.to_string());
}
