use hudhudscript_tools_schema::schema::{JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema};
use std::collections::HashMap;

/// Register `DatabaseTool` into the `ToolRegistry` as standard tools.
///
/// Registers the following tool names:
/// - `db_execute_query` — run a SQL query
/// - `db_list_tables`   — list tables in the connected database
pub fn register_database_tools(
    registry: &hudhudscript_tools_schema::ToolRegistry,
) -> Result<usize, hudhudscript_tools_schema::RegistryError> {
    let tools: &[(&str, &str, &[(&str, &str, bool)])] = &[
        (
            "db_execute_query",
            "Execute a SQL query against a database and return the result rows",
            &[
                ("sql", "string", true),
                ("connection_string", "string", true),
                ("backend", "string", true),
            ],
        ),
        (
            "db_list_tables",
            "List all tables in the connected database",
            &[
                ("connection_string", "string", true),
                ("backend", "string", true),
            ],
        ),
    ];

    let mut count = 0;
    for (name, description, params) in tools {
        let properties: HashMap<String, JsonSchemaProperty> = params
            .iter()
            .map(|(field, ty, _)| {
                (
                    field.to_string(),
                    JsonSchemaProperty {
                        property_type: ty.to_string(),
                        description: None,
                        default: None,
                        enum_values: None,
                    },
                )
            })
            .collect();

        let required: Vec<String> = params
            .iter()
            .filter(|(_, _, req)| *req)
            .map(|(f, _, _)| f.to_string())
            .collect();

        let schema = ToolSchema {
            name: name.to_string(),
            description: Some(description.to_string()),
            input_schema: JsonSchema {
                schema_type: "object".to_string(),
                properties: Some(properties),
                required: Some(required),
                items: None,
                description: Some(description.to_string()),
            },
            server: "built-in".to_string(),
        };

        let mut metadata = ToolMetadata::new(
            name.to_string(),
            "built-in".to_string(),
            Some(description.to_string()),
        );
        metadata.add_tag("database".to_string());
        metadata.add_tag("standard".to_string());

        registry.register_tool(schema, metadata)?;
        count += 1;
    }

    Ok(count)
}
