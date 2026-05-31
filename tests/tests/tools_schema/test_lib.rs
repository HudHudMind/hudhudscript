//! Public API tests for hudhudscript-tools-schema

use hudhudscript_tools_schema::{
    CacheStats, JsonSchema, JsonSchemaProperty, RegistryError, ToolCache, ToolMetadata,
    ToolRegistry, ToolSchema, ValidationError,
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

fn make_schema(name: &str, server: &str) -> ToolSchema {
    ToolSchema {
        name: name.into(),
        description: Some(format!("{} tool", name)),
        input_schema: JsonSchema {
            schema_type: "object".into(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: server.into(),
    }
}
fn make_metadata(name: &str, server: &str) -> ToolMetadata {
    ToolMetadata::new(name.into(), server.into(), Some(format!("{} desc", name)))
}

// ── ToolMetadata ────────────────────────────────────────────────────

#[test]
fn test_metadata_new() {
    let m = ToolMetadata::new("t".into(), "s".into(), None);
    assert_eq!(m.name, "t");
    assert_eq!(m.usage_count, 0);
    assert!(m.last_used.is_none());
    assert!(m.tags.is_empty());
}
#[test]
fn test_metadata_with_desc() {
    assert_eq!(
        ToolMetadata::new("t".into(), "s".into(), Some("d".into()))
            .description
            .as_deref(),
        Some("d")
    );
}
#[test]
fn test_metadata_record_usage() {
    let mut m = ToolMetadata::new("t".into(), "s".into(), None);
    m.record_usage();
    assert_eq!(m.usage_count, 1);
    assert!(m.last_used.is_some());
    m.record_usage();
    assert_eq!(m.usage_count, 2);
}
#[test]
fn test_metadata_add_tag() {
    let mut m = ToolMetadata::new("t".into(), "s".into(), None);
    m.add_tag("io".into());
    assert_eq!(m.tags.len(), 1);
    m.add_tag("io".into());
    assert_eq!(m.tags.len(), 1);
    m.add_tag("net".into());
    assert_eq!(m.tags.len(), 2);
}
#[test]
fn test_metadata_serde() {
    let mut m = ToolMetadata::new("t".into(), "s".into(), Some("d".into()));
    m.record_usage();
    m.add_tag("cat".into());
    let d: ToolMetadata = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    assert_eq!(d.name, "t");
    assert_eq!(d.usage_count, 1);
}

// ── JsonSchema validation ───────────────────────────────────────────

#[test]
fn test_validate_string_ok() {
    let s = JsonSchema {
        schema_type: "string".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!("hi")).is_ok());
}
#[test]
fn test_validate_string_err() {
    let s = JsonSchema {
        schema_type: "string".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!(42)).is_err());
}
#[test]
fn test_validate_number_ok() {
    let s = JsonSchema {
        schema_type: "number".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!(3.14)).is_ok());
}
#[test]
fn test_validate_number_err() {
    let s = JsonSchema {
        schema_type: "number".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!("x")).is_err());
}
#[test]
fn test_validate_integer_ok() {
    let s = JsonSchema {
        schema_type: "integer".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!(42)).is_ok());
}
#[test]
fn test_validate_integer_err() {
    let s = JsonSchema {
        schema_type: "integer".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!(3.14)).is_err());
}
#[test]
fn test_validate_boolean_ok() {
    let s = JsonSchema {
        schema_type: "boolean".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!(true)).is_ok());
}
#[test]
fn test_validate_boolean_err() {
    let s = JsonSchema {
        schema_type: "boolean".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!(1)).is_err());
}
#[test]
fn test_validate_null_ok() {
    let s = JsonSchema {
        schema_type: "null".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!(null)).is_ok());
}
#[test]
fn test_validate_null_err() {
    let s = JsonSchema {
        schema_type: "null".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!("x")).is_err());
}
#[test]
fn test_validate_array_ok() {
    let s = JsonSchema {
        schema_type: "array".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!([1, 2])).is_ok());
}
#[test]
fn test_validate_array_err() {
    let s = JsonSchema {
        schema_type: "array".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!({})).is_err());
}
#[test]
fn test_validate_array_items() {
    let items = JsonSchema {
        schema_type: "string".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    let s = JsonSchema {
        schema_type: "array".into(),
        properties: None,
        required: None,
        items: Some(Box::new(items)),
        description: None,
    };
    assert!(s.validate(&json!(["a"])).is_ok());
    assert!(s.validate(&json!([1])).is_err());
}
#[test]
fn test_validate_object_ok() {
    let s = JsonSchema {
        schema_type: "object".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!({})).is_ok());
}
#[test]
fn test_validate_object_err() {
    let s = JsonSchema {
        schema_type: "object".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!("x")).is_err());
}
#[test]
fn test_validate_object_missing_required() {
    let s = JsonSchema {
        schema_type: "object".into(),
        properties: None,
        required: Some(vec!["name".into()]),
        items: None,
        description: None,
    };
    assert!(
        matches!(s.validate(&json!({})).unwrap_err(), ValidationError::MissingRequired(ref f) if f == "name")
    );
}
#[test]
fn test_validate_object_property_type() {
    let mut p = HashMap::new();
    p.insert(
        "age".into(),
        JsonSchemaProperty {
            property_type: "integer".into(),
            description: None,
            default: None,
            enum_values: None,
        },
    );
    let s = JsonSchema {
        schema_type: "object".into(),
        properties: Some(p),
        required: None,
        items: None,
        description: None,
    };
    assert!(s.validate(&json!({"age": 25})).is_ok());
    assert!(s.validate(&json!({"age": "old"})).is_err());
}
#[test]
fn test_validate_unknown_type() {
    assert!(matches!(
        JsonSchema {
            schema_type: "unicorn".into(),
            properties: None,
            required: None,
            items: None,
            description: None
        }
        .validate(&json!("x"))
        .unwrap_err(),
        ValidationError::UnknownType(_)
    ));
}
#[test]
fn test_validate_object_extra_props_ok() {
    let mut p = HashMap::new();
    p.insert(
        "name".into(),
        JsonSchemaProperty {
            property_type: "string".into(),
            description: None,
            default: None,
            enum_values: None,
        },
    );
    let s = JsonSchema {
        schema_type: "object".into(),
        properties: Some(p),
        required: Some(vec!["name".into()]),
        items: None,
        description: None,
    };
    assert!(s.validate(&json!({"name": "x", "extra": 1})).is_ok());
}

// ── ValidationError ─────────────────────────────────────────────────

#[test]
fn test_validation_error_display() {
    assert!(ValidationError::TypeMismatch {
        expected: "string".into(),
        found: "number".into()
    }
    .to_string()
    .contains("Type mismatch: expected string, found number"));
    assert!(ValidationError::MissingRequired("f".into())
        .to_string()
        .contains("f"));
    assert!(ValidationError::UnknownType("x".into())
        .to_string()
        .contains("x"));
}
#[test]
fn test_validation_error_clone() {
    let e = ValidationError::TypeMismatch {
        expected: "a".into(),
        found: "b".into(),
    };
    assert_eq!(e.to_string(), e.clone().to_string());
}

// ── ToolSchema serde ────────────────────────────────────────────────

#[test]
fn test_tool_schema_serde() {
    let d: ToolSchema =
        serde_json::from_str(&serde_json::to_string(&make_schema("read", "fs")).unwrap()).unwrap();
    assert_eq!(d.name, "read");
}
#[test]
fn test_json_schema_skip_none() {
    let j = serde_json::to_string(&JsonSchema {
        schema_type: "string".into(),
        properties: None,
        required: None,
        items: None,
        description: None,
    })
    .unwrap();
    assert!(!j.contains("properties"));
}
#[test]
fn test_property_defaults_enums() {
    let p = JsonSchemaProperty {
        property_type: "string".into(),
        description: Some("C".into()),
        default: Some(json!("red")),
        enum_values: Some(vec![json!("red"), json!("blue")]),
    };
    let d: JsonSchemaProperty = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(d.default, Some(json!("red")));
    assert_eq!(d.enum_values.unwrap().len(), 2);
}

// ── ToolCache ───────────────────────────────────────────────────────

#[test]
fn test_cache_new() {
    assert_eq!(ToolCache::new(Duration::from_secs(60)).size(), 0);
}
#[test]
fn test_cache_put_get() {
    let mut c = ToolCache::new(Duration::from_secs(60));
    c.put(make_schema("t", "s"));
    assert_eq!(c.get("t").unwrap().name, "t");
}
#[test]
fn test_cache_get_missing() {
    assert!(ToolCache::new(Duration::from_secs(60)).get("x").is_none());
}
#[test]
fn test_cache_clear() {
    let mut c = ToolCache::new(Duration::from_secs(60));
    c.put(make_schema("a", "s"));
    c.clear();
    assert_eq!(c.size(), 0);
}
#[test]
fn test_cache_expired() {
    let mut c = ToolCache::new(Duration::from_nanos(1));
    c.put(make_schema("t", "s"));
    std::thread::sleep(Duration::from_millis(1));
    assert!(c.get("t").is_none());
}
#[test]
fn test_cache_cleanup_expired() {
    let mut c = ToolCache::new(Duration::from_nanos(1));
    c.put(make_schema("t", "s"));
    std::thread::sleep(Duration::from_millis(1));
    c.cleanup();
    assert_eq!(c.size(), 0);
}
#[test]
fn test_cache_cleanup_fresh() {
    let mut c = ToolCache::new(Duration::from_secs(300));
    c.put(make_schema("t", "s"));
    c.cleanup();
    assert_eq!(c.size(), 1);
}
#[test]
fn test_cache_overwrite() {
    let mut c = ToolCache::new(Duration::from_secs(60));
    c.put(make_schema("t", "s1"));
    c.put(make_schema("t", "s2"));
    assert_eq!(c.get("t").unwrap().server, "s2");
}

// ── ToolRegistry ────────────────────────────────────────────────────

#[test]
fn test_registry_new() {
    assert!(ToolRegistry::new().list_tools().is_empty());
}
#[test]
fn test_registry_default() {
    assert!(ToolRegistry::default().list_tools().is_empty());
}
#[test]
fn test_registry_register_get() {
    let r = ToolRegistry::new();
    r.register_tool(make_schema("r", "f"), make_metadata("r", "f"))
        .unwrap();
    assert_eq!(r.get_tool("r").unwrap().name, "r");
}
#[test]
fn test_registry_get_missing() {
    assert!(ToolRegistry::new().get_tool("x").is_none());
}
#[test]
fn test_registry_get_metadata() {
    let r = ToolRegistry::new();
    r.register_tool(make_schema("t", "s"), make_metadata("t", "s"))
        .unwrap();
    assert_eq!(r.get_metadata("t").unwrap().name, "t");
}
#[test]
fn test_registry_list() {
    let r = ToolRegistry::new();
    r.register_tool(make_schema("a", "s"), make_metadata("a", "s"))
        .unwrap();
    r.register_tool(make_schema("b", "s"), make_metadata("b", "s"))
        .unwrap();
    let mut t = r.list_tools();
    t.sort();
    assert_eq!(t, vec!["a", "b"]);
}
#[test]
fn test_registry_validate_ok() {
    let mut p = HashMap::new();
    p.insert(
        "path".into(),
        JsonSchemaProperty {
            property_type: "string".into(),
            description: None,
            default: None,
            enum_values: None,
        },
    );
    let s = ToolSchema {
        name: "r".into(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".into(),
            properties: Some(p),
            required: Some(vec!["path".into()]),
            items: None,
            description: None,
        },
        server: "f".into(),
    };
    let r = ToolRegistry::new();
    r.register_tool(s, make_metadata("r", "f")).unwrap();
    assert!(r.validate_arguments("r", &json!({"path": "/tmp"})).is_ok());
}
#[test]
fn test_registry_validate_missing() {
    let s = ToolSchema {
        name: "t".into(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".into(),
            properties: None,
            required: Some(vec!["x".into()]),
            items: None,
            description: None,
        },
        server: "s".into(),
    };
    let r = ToolRegistry::new();
    r.register_tool(s, make_metadata("t", "s")).unwrap();
    assert!(r.validate_arguments("t", &json!({})).is_err());
}
#[test]
fn test_registry_validate_not_found() {
    assert!(ToolRegistry::new()
        .validate_arguments("x", &json!({}))
        .is_err());
}
#[test]
fn test_registry_clear_cache() {
    let r = ToolRegistry::new();
    r.register_tool(make_schema("t", "s"), make_metadata("t", "s"))
        .unwrap();
    r.clear_cache();
    assert_eq!(r.cache_stats().size, 0);
    assert!(r.get_tool("t").is_some());
}
#[test]
fn test_registry_cache_stats() {
    let s = ToolRegistry::new().cache_stats();
    assert_eq!(s.size, 0);
    assert_eq!(s.ttl, Duration::from_secs(300));
}

// ── RegistryError ───────────────────────────────────────────────────

#[test]
fn test_registry_error_display() {
    assert!(RegistryError::ToolNotFound("t".into())
        .to_string()
        .contains("t"));
    assert!(RegistryError::ServerNotFound("s".into())
        .to_string()
        .contains("s"));
    assert!(RegistryError::DiscoveryFailed("d".into())
        .to_string()
        .contains("d"));
    assert!(RegistryError::CallFailed("c".into())
        .to_string()
        .contains("c"));
    assert!(
        RegistryError::ValidationFailed(ValidationError::MissingRequired("f".into()))
            .to_string()
            .contains("Validation")
    );
}
#[test]
fn test_registry_error_clone() {
    let e = RegistryError::ToolNotFound("t".into());
    assert_eq!(e.to_string(), e.clone().to_string());
}
#[test]
fn test_cache_stats_debug() {
    assert!(format!(
        "{:?}",
        CacheStats {
            size: 5,
            ttl: Duration::from_secs(300)
        }
    )
    .contains("5"));
}
