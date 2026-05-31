pub fn register_git_tools(
    registry: &hudhudscript_tools_schema::ToolRegistry,
) -> Result<usize, hudhudscript_tools_schema::RegistryError> {
    use hudhudscript_tools_schema::schema::{
        JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema,
    };
    use std::collections::HashMap;

    let tools: &[(&str, &str, &[(&str, &str, bool)])] = &[
        (
            "git_status",
            "Show the working tree status of a git repository",
            &[("workdir", "string", false)],
        ),
        (
            "git_commit",
            "Stage all changes and create a git commit with the given message",
            &[
                ("message", "string", true),
                ("workdir", "string", false),
                ("stage_all", "boolean", false),
            ],
        ),
        (
            "git_push",
            "Push commits to a remote git repository",
            &[
                ("workdir", "string", false),
                ("remote", "string", false),
                ("branch", "string", false),
            ],
        ),
        (
            "git_branch",
            "List, create, or delete git branches",
            &[
                ("workdir", "string", false),
                ("action", "string", false),
                ("name", "string", false),
            ],
        ),
        (
            "git_checkout",
            "Checkout a branch or commit in a git repository",
            &[
                ("target", "string", true),
                ("workdir", "string", false),
                ("new_branch", "boolean", false),
            ],
        ),
        (
            "git_log",
            "Show the recent commit log for a git repository",
            &[("workdir", "string", false), ("count", "integer", false)],
        ),
    ];

    for (name, _, _) in tools {
        if registry.get_tool(name).is_some() {
            return Err(hudhudscript_tools_schema::RegistryError::DuplicateTool(
                name.to_string(),
            ));
        }
    }

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
                required: if required.is_empty() {
                    None
                } else {
                    Some(required)
                },
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
        metadata.add_tag("git".to_string());
        metadata.add_tag("standard".to_string());

        registry.register_tool(schema, metadata)?;
        count += 1;
    }

    Ok(count)
}
