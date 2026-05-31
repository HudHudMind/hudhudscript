use hudhudscript_tools::registry::ToolRegistry;
use hudhudscript_tools::schema::{JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();
    assert_eq!(registry.list_tools().len(), 0);
}

#[test]
fn test_tool_registration() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "test_tool".to_string(),
        description: Some("Test tool".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: None,
            items: None,
            description: None,
        },
        server: "test-server".to_string(),
    };

    let metadata = ToolMetadata::new(
        "test_tool".to_string(),
        "test-server".to_string(),
        Some("Test tool".to_string()),
    );

    assert!(registry.register_tool(schema, metadata).is_ok());
    assert_eq!(registry.list_tools().len(), 1);
    assert!(registry.get_tool("test_tool").is_some());
}

#[test]
fn test_tool_lookup() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "lookup_test".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "test-server".to_string(),
    };

    let metadata = ToolMetadata::new("lookup_test".to_string(), "test-server".to_string(), None);

    registry.register_tool(schema.clone(), metadata).unwrap();

    let retrieved = registry.get_tool("lookup_test");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "lookup_test");

    let not_found = registry.get_tool("nonexistent");
    assert!(not_found.is_none());
}

#[test]
fn test_argument_validation() {
    let registry = ToolRegistry::new();

    let mut properties = HashMap::new();
    properties.insert(
        "name".to_string(),
        JsonSchemaProperty {
            property_type: "string".to_string(),
            description: None,
            default: None,
            enum_values: None,
        },
    );

    let schema = ToolSchema {
        name: "validate_test".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: Some(vec!["name".to_string()]),
            items: None,
            description: None,
        },
        server: "test-server".to_string(),
    };

    let metadata = ToolMetadata::new("validate_test".to_string(), "test-server".to_string(), None);

    registry.register_tool(schema, metadata).unwrap();

    // Valid arguments
    let valid_args = json!({ "name": "test" });
    assert!(registry
        .validate_arguments("validate_test", &valid_args)
        .is_ok());

    // Invalid arguments (missing required field)
    let invalid_args = json!({ "age": 30 });
    assert!(registry
        .validate_arguments("validate_test", &invalid_args)
        .is_err());
}

#[test]
fn test_cache_operations() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "cache_test".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "test-server".to_string(),
    };

    let metadata = ToolMetadata::new("cache_test".to_string(), "test-server".to_string(), None);

    registry.register_tool(schema, metadata).unwrap();

    let stats = registry.cache_stats();
    assert_eq!(stats.size, 1);

    registry.clear_cache();
    let stats_after = registry.cache_stats();
    assert_eq!(stats_after.size, 0);
}

#[test]
fn test_sub_crate_reexports() {
    // Verify sub-crate re-exports are accessible
    let _ = hudhudscript_tools::sub::schema::ToolRegistry::new();
}

#[test]
fn test_registry_default() {
    let registry = ToolRegistry::default();
    assert_eq!(registry.list_tools().len(), 0);
}

#[test]
fn test_get_metadata_after_registration() {
    let registry = ToolRegistry::new();
    let schema = ToolSchema {
        name: "meta_test".to_string(),
        description: Some("Meta tool".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "meta-server".to_string(),
    };
    let metadata = ToolMetadata::new(
        "meta_test".to_string(),
        "meta-server".to_string(),
        Some("Meta tool".to_string()),
    );
    registry.register_tool(schema, metadata).unwrap();

    let meta = registry.get_metadata("meta_test").unwrap();
    assert_eq!(meta.name, "meta_test");
    assert_eq!(meta.server, "meta-server");
    assert_eq!(meta.description.as_deref(), Some("Meta tool"));
    assert_eq!(meta.usage_count, 0);
}

#[test]
fn test_get_metadata_nonexistent() {
    let registry = ToolRegistry::new();
    assert!(registry.get_metadata("no_such_tool").is_none());
}

#[test]
fn test_validate_arguments_nonexistent_tool() {
    let registry = ToolRegistry::new();
    let result = registry.validate_arguments("no_such_tool", &json!({}));
    assert!(result.is_err());
}

#[test]
fn test_validate_arguments_type_mismatch() {
    let registry = ToolRegistry::new();
    let mut properties = HashMap::new();
    properties.insert(
        "count".to_string(),
        JsonSchemaProperty {
            property_type: "integer".to_string(),
            description: None,
            default: None,
            enum_values: None,
        },
    );

    let schema = ToolSchema {
        name: "type_check".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: Some(vec!["count".to_string()]),
            items: None,
            description: None,
        },
        server: "test-server".to_string(),
    };
    let metadata = ToolMetadata::new("type_check".to_string(), "test-server".to_string(), None);
    registry.register_tool(schema, metadata).unwrap();

    // Pass a string where integer is expected
    let result = registry.validate_arguments("type_check", &json!({"count": "not_a_number"}));
    assert!(result.is_err());
}

#[test]
fn test_cache_stats_ttl() {
    let registry = ToolRegistry::new();
    let stats = registry.cache_stats();
    assert_eq!(stats.ttl, std::time::Duration::from_secs(300));
}

#[test]
fn test_multiple_tool_registration_and_listing() {
    let registry = ToolRegistry::new();
    for i in 0..5 {
        let name = format!("tool_{}", i);
        let schema = ToolSchema {
            name: name.clone(),
            description: None,
            input_schema: JsonSchema {
                schema_type: "object".to_string(),
                properties: None,
                required: None,
                items: None,
                description: None,
            },
            server: "test-server".to_string(),
        };
        let metadata = ToolMetadata::new(name, "test-server".to_string(), None);
        registry.register_tool(schema, metadata).unwrap();
    }
    assert_eq!(registry.list_tools().len(), 5);
    assert_eq!(registry.cache_stats().size, 5);
}

#[test]
fn test_overwrite_tool_registration() {
    let registry = ToolRegistry::new();
    let schema1 = ToolSchema {
        name: "overwrite_test".to_string(),
        description: Some("first".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "server-1".to_string(),
    };
    let meta1 = ToolMetadata::new(
        "overwrite_test".to_string(),
        "server-1".to_string(),
        Some("first".to_string()),
    );
    registry.register_tool(schema1, meta1).unwrap();

    let schema2 = ToolSchema {
        name: "overwrite_test".to_string(),
        description: Some("second".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
            items: None,
            description: None,
        },
        server: "server-2".to_string(),
    };
    let meta2 = ToolMetadata::new(
        "overwrite_test".to_string(),
        "server-2".to_string(),
        Some("second".to_string()),
    );
    registry.register_tool(schema2, meta2).unwrap();

    let tool = registry.get_tool("overwrite_test").unwrap();
    assert_eq!(tool.server, "server-2");
    assert_eq!(tool.description.as_deref(), Some("second"));
}
