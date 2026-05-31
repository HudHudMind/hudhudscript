//! Tests for hudhudscript-tools-schema: registry, cache, metadata, serialization

use hudhudscript_tools_schema::{
    JsonSchema, JsonSchemaProperty, ToolCache, ToolMetadata, ToolRegistry, ToolSchema,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------
fn simple_tool_schema(name: &str) -> ToolSchema {
    let mut props = HashMap::new();
    props.insert(
        "input".to_string(),
        JsonSchemaProperty {
            property_type: "string".to_string(),
            description: Some("The input text".to_string()),
            default: None,
            enum_values: None,
        },
    );
    ToolSchema {
        name: name.to_string(),
        description: Some(format!("Tool {}", name)),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(props),
            required: Some(vec!["input".to_string()]),
            items: None,
            description: None,
        },
        server: "test-server".to_string(),
    }
}

fn simple_metadata(name: &str) -> ToolMetadata {
    ToolMetadata::new(
        name.to_string(),
        "test-server".to_string(),
        Some(format!("Tool {}", name)),
    )
}

// ===== 1. ToolRegistry register + get round-trip =====
#[test]
fn registry_register_and_get_tool() {
    let registry = ToolRegistry::new();
    let schema = simple_tool_schema("echo");
    let meta = simple_metadata("echo");

    registry.register_tool(schema, meta).unwrap();

    let retrieved = registry.get_tool("echo").expect("tool should exist");
    assert_eq!(retrieved.name, "echo");
    assert_eq!(retrieved.server, "test-server");
}

// ===== 2. ToolRegistry list_tools =====
#[test]
fn registry_list_tools_returns_registered_names() {
    let registry = ToolRegistry::new();
    registry
        .register_tool(simple_tool_schema("alpha"), simple_metadata("alpha"))
        .unwrap();
    registry
        .register_tool(simple_tool_schema("beta"), simple_metadata("beta"))
        .unwrap();

    let mut names = registry.list_tools();
    names.sort();
    assert_eq!(names, vec!["alpha", "beta"]);
}

// ===== 3. ToolRegistry validate_arguments success =====
#[test]
fn registry_validate_arguments_success() {
    let registry = ToolRegistry::new();
    registry
        .register_tool(simple_tool_schema("greet"), simple_metadata("greet"))
        .unwrap();

    let args = json!({"input": "world"});
    assert!(registry.validate_arguments("greet", &args).is_ok());
}

// ===== 4. ToolRegistry validate_arguments failure — missing required =====
#[test]
fn registry_validate_arguments_missing_required() {
    let registry = ToolRegistry::new();
    registry
        .register_tool(simple_tool_schema("greet"), simple_metadata("greet"))
        .unwrap();

    let args = json!({});
    let err = registry.validate_arguments("greet", &args);
    assert!(err.is_err(), "Should fail when required field is missing");
}

// ===== 5. ToolRegistry validate_arguments — tool not found =====
#[test]
fn registry_validate_arguments_tool_not_found() {
    let registry = ToolRegistry::new();
    let err = registry.validate_arguments("nonexistent", &json!({}));
    assert!(err.is_err());
}

// ===== 6. ToolMetadata record_usage updates count and last_used =====
#[test]
fn metadata_record_usage_updates_fields() {
    let mut meta = simple_metadata("tool1");
    assert_eq!(meta.usage_count, 0);
    assert!(meta.last_used.is_none());

    meta.record_usage();
    assert_eq!(meta.usage_count, 1);
    assert!(meta.last_used.is_some());

    meta.record_usage();
    assert_eq!(meta.usage_count, 2);
}

// ===== 7. ToolMetadata add_tag deduplicates =====
#[test]
fn metadata_add_tag_deduplicates() {
    let mut meta = simple_metadata("tool1");
    meta.add_tag("io".to_string());
    meta.add_tag("io".to_string());
    meta.add_tag("net".to_string());
    assert_eq!(meta.tags, vec!["io", "net"]);
}

// ===== 8. ToolCache put/get and clear =====
#[test]
fn cache_put_get_and_clear() {
    let mut cache = ToolCache::new(Duration::from_secs(60));
    assert_eq!(cache.size(), 0);

    cache.put(simple_tool_schema("cached_tool"));
    assert_eq!(cache.size(), 1);

    let retrieved = cache.get("cached_tool").expect("should be cached");
    assert_eq!(retrieved.name, "cached_tool");

    cache.clear();
    assert_eq!(cache.size(), 0);
    assert!(cache.get("cached_tool").is_none());
}

// ===== 9. ToolSchema serialization round-trip =====
#[test]
fn tool_schema_serde_roundtrip() {
    let schema = simple_tool_schema("roundtrip");
    let json_str = serde_json::to_string(&schema).expect("serialize");
    let deserialized: ToolSchema = serde_json::from_str(&json_str).expect("deserialize");

    assert_eq!(deserialized.name, "roundtrip");
    assert_eq!(deserialized.server, "test-server");
    assert_eq!(deserialized.input_schema.schema_type, "object");
    let props = deserialized.input_schema.properties.as_ref().unwrap();
    assert!(props.contains_key("input"));
    assert_eq!(props["input"].property_type, "string");
}

// ===== 10. JsonSchemaProperty with enum_values and default serializes correctly =====
#[test]
fn json_schema_property_optional_fields_serde() {
    let prop = JsonSchemaProperty {
        property_type: "string".to_string(),
        description: Some("A color".to_string()),
        default: Some(json!("red")),
        enum_values: Some(vec![json!("red"), json!("green"), json!("blue")]),
    };
    let json_str = serde_json::to_string(&prop).expect("serialize");
    let deserialized: JsonSchemaProperty = serde_json::from_str(&json_str).expect("deserialize");

    assert_eq!(deserialized.property_type, "string");
    assert_eq!(deserialized.description.as_deref(), Some("A color"));
    assert_eq!(deserialized.default, Some(json!("red")));
    assert_eq!(
        deserialized.enum_values,
        Some(vec![json!("red"), json!("green"), json!("blue")])
    );
}

// ===== 11. Registry cache_stats reflects state =====
#[test]
fn registry_cache_stats() {
    let registry = ToolRegistry::new();
    let stats = registry.cache_stats();
    assert_eq!(stats.size, 0);
    assert_eq!(stats.ttl, Duration::from_secs(300));

    registry
        .register_tool(simple_tool_schema("x"), simple_metadata("x"))
        .unwrap();
    let stats = registry.cache_stats();
    assert_eq!(stats.size, 1);
}

// ===== 12. Registry clear_cache empties cache but keeps tools =====
#[test]
fn registry_clear_cache_keeps_tools() {
    let registry = ToolRegistry::new();
    registry
        .register_tool(simple_tool_schema("persist"), simple_metadata("persist"))
        .unwrap();

    assert_eq!(registry.cache_stats().size, 1);
    registry.clear_cache();
    assert_eq!(registry.cache_stats().size, 0);

    // Tool should still be accessible via the main map (cache miss, fallback)
    let tool = registry
        .get_tool("persist")
        .expect("tool should still exist in registry");
    assert_eq!(tool.name, "persist");
}
