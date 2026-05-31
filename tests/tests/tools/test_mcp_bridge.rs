use hudhudscript_tools::registry::ToolRegistry;
use hudhudscript_tools::schema::{JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema};
use std::collections::HashMap;

/// Verify that ToolRegistry behaves correctly after manual registration
/// (the MCP client is not instantiatable in unit tests without a running server)
#[test]
fn test_registry_after_manual_registration() {
    let registry = ToolRegistry::new();

    let schema = ToolSchema {
        name: "search".to_string(),
        description: Some("Search the web".to_string()),
        input_schema: JsonSchema {
            schema_type: "object".to_string(),
            properties: Some({
                let mut m = HashMap::new();
                m.insert(
                    "query".to_string(),
                    JsonSchemaProperty {
                        property_type: "string".to_string(),
                        description: Some("Search query".to_string()),
                        default: None,
                        enum_values: None,
                    },
                );
                m
            }),
            required: Some(vec!["query".to_string()]),
            items: None,
            description: None,
        },
        server: "brave-search".to_string(),
    };

    let metadata = ToolMetadata::new(
        "search".to_string(),
        "brave-search".to_string(),
        Some("Search the web".to_string()),
    );

    registry.register_tool(schema, metadata).unwrap();

    let tool = registry.get_tool("search").unwrap();
    assert_eq!(tool.server, "brave-search");
}
