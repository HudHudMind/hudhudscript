//! Comprehensive tests for hudhudscript-tools

use hudhudscript_tools::registry::ToolRegistry;
use hudhudscript_tools::schema::{
    JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema, ValidationError,
};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_tool_schema_creation() {
    let mut properties = HashMap::new();
    properties.insert(
        "path".to_string(),
        JsonSchemaProperty {
            property_type: "string".to_string(),
            description: Some("File path".to_string()),
            default: None,
            enum_values: None,
        },
    );

    let schema = ToolSchema {
        name: "read_file".to_string(),
        description: Some("Read file content".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(properties),
            required: Some(vec!["path".to_string()]),
            items: None,
            description: None,
        },
        server: "filesystem".to_string(),
    };

    assert_eq!(schema.name, "read_file");
    assert_eq!(schema.server, "filesystem");
    assert!(schema.description.is_some());
}

#[test]
fn test_tool_metadata_creation() {
    let metadata = ToolMetadata::new(
        "test_tool".to_string(),
        "server1".to_string(),
        Some("Test tool description".to_string()),
    );

    assert_eq!(metadata.name, "test_tool");
    assert_eq!(metadata.server, "server1");
    assert_eq!(metadata.usage_count, 0);
    assert!(metadata.last_used.is_none());
}

#[test]
fn test_tool_metadata_record_usage() {
    let mut metadata = ToolMetadata::new("test_tool".to_string(), "server1".to_string(), None);

    metadata.record_usage();
    assert_eq!(metadata.usage_count, 1);
    assert!(metadata.last_used.is_some());

    metadata.record_usage();
    assert_eq!(metadata.usage_count, 2);
}

#[test]
fn test_json_schema_validation_valid() {
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
    properties.insert(
        "age".to_string(),
        JsonSchemaProperty {
            property_type: "number".to_string(),
            description: None,
            default: None,
            enum_values: None,
        },
    );

    let schema = JsonSchema {
        schema_type: "object".to_string(),
        properties: Some(properties),
        required: Some(vec!["name".to_string()]),
        items: None,
        description: None,
    };

    let valid_data = json!({
        "name": "John",
        "age": 30
    });

    assert!(schema.validate(&valid_data).is_ok());
}

#[test]
fn test_json_schema_validation_missing_required() {
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

    let schema = JsonSchema {
        schema_type: "object".to_string(),
        properties: Some(properties),
        required: Some(vec!["name".to_string()]),
        items: None,
        description: None,
    };

    let invalid_data = json!({
        "age": 30
    });

    let result = schema.validate(&invalid_data);
    assert!(result.is_err());
    match result {
        Err(ValidationError::MissingRequired(field)) => {
            assert_eq!(field, "name");
        }
        _ => panic!("Expected MissingRequired error"),
    }
}

#[test]
fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();
    assert_eq!(registry.list_tools().len(), 0);
}

#[test]
fn test_tool_registry_register() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "test_tool".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: Some(vec![]),
            items: None,
            description: None,
        },
        server: "test_server".to_string(),
    };

    let metadata = ToolMetadata::new("test_tool".to_string(), "test_server".to_string(), None);

    registry.register_tool(schema, metadata).unwrap();
    assert_eq!(registry.list_tools().len(), 1);
}

#[test]
fn test_tool_registry_get_tool() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "test_tool".to_string(),
        description: Some("Test description".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: Some(vec![]),
            items: None,
            description: None,
        },
        server: "test_server".to_string(),
    };

    let metadata = ToolMetadata::new(
        "test_tool".to_string(),
        "test_server".to_string(),
        Some("Test description".to_string()),
    );

    registry.register_tool(schema, metadata).unwrap();

    let retrieved = registry.get_tool("test_tool");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_tool");
}

#[test]
fn test_tool_registry_get_metadata() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "test_tool".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: Some(vec![]),
            items: None,
            description: None,
        },
        server: "test_server".to_string(),
    };

    let metadata = ToolMetadata::new("test_tool".to_string(), "test_server".to_string(), None);

    registry.register_tool(schema, metadata).unwrap();

    let retrieved = registry.get_metadata("test_tool");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "test_tool");
}

#[test]
fn test_validation_error_display() {
    let error = ValidationError::MissingRequired("name".to_string());
    let display = format!("{}", error);
    assert!(display.contains("Missing required field"));

    let error = ValidationError::TypeMismatch {
        expected: "number".to_string(),
        found: "string".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("Type mismatch"));
}

#[test]
fn test_registry_cache_clear() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "test".to_string(),
        description: None,
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some(HashMap::new()),
            required: Some(vec![]),
            items: None,
            description: None,
        },
        server: "server1".to_string(),
    };

    let metadata = ToolMetadata::new("test".to_string(), "server1".to_string(), None);

    registry.register_tool(schema, metadata).unwrap();
    assert_eq!(registry.list_tools().len(), 1);

    registry.clear_cache();
    // Cache cleared but registry still has the tool
    assert_eq!(registry.list_tools().len(), 1);
}

#[test]
fn test_multiple_tools() {
    let registry = ToolRegistry::new();

    for i in 1..=5 {
        let schema = ToolSchema {
            name: format!("tool{}", i),
            description: None,
            input_schema: JsonSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::new()),
                required: Some(vec![]),
                items: None,
                description: None,
            },
            server: "test_server".to_string(),
        };

        let metadata = ToolMetadata::new(format!("tool{}", i), "test_server".to_string(), None);

        registry.register_tool(schema, metadata).unwrap();
    }

    assert_eq!(registry.list_tools().len(), 5);
}

#[test]
fn test_cache_stats() {
    let registry = ToolRegistry::new();
    let stats = registry.cache_stats();
    assert_eq!(stats.size, 0);
}
