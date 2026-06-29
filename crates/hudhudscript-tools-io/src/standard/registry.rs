use hudhudscript_tools_schema::registry::{RegistryError, ToolRegistry};
use hudhudscript_tools_schema::schema::{JsonSchema, JsonSchemaProperty, ToolSchema};
use serde_json::Value;
use std::collections::HashMap;

use super::StandardTool;

/// Register all standard built-in tools into a `ToolRegistry`.
///
/// This includes the core standard tools (`file_read`, `http_get`, `json_parse`)
/// as well as the database tools (Issue #21) and git tools (Issue #22).
///
/// Returns the total number of tools registered.
pub fn register_standard_tools(registry: &ToolRegistry) -> Result<usize, RegistryError> {
    let core_tools = [
        StandardTool::FileRead,
        StandardTool::HttpGet,
        StandardTool::HttpPost,
        StandardTool::HttpPut,
        StandardTool::HttpDelete,
        StandardTool::JsonParse,
    ];

    let mut count = 0;
    for tool in &core_tools {
        let schema = tool.to_schema()?;
        let metadata = tool.to_metadata();
        registry.register_tool(schema, metadata)?;
        count += 1;
    }

    count += crate::database::register_database_tools(registry)?;

    // Register git tools (side effect, not counted)
    let _git_count = hudhudscript_tools_vcs::git::register_git_tools(registry)?;

    Ok(count)
}

/// Build a JSON Schema `object` value from a list of `(name, type, required, desc)` tuples.
///
/// Useful when implementing `CustomTool::parameter_schema`.
pub fn build_object_schema(fields: &[(&str, &str, bool, Option<&str>)]) -> Value {
    let mut properties: HashMap<String, Value> = HashMap::new();
    let mut required: Vec<String> = Vec::new();

    for (name, ty, req, desc) in fields {
        let mut prop = serde_json::json!({ "type": ty });
        if let Some(d) = desc {
            prop["description"] = Value::String(d.to_string());
        }
        properties.insert(name.to_string(), prop);
        if *req {
            required.push(name.to_string());
        }
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Build a minimal `JsonSchema` for an object with the given required string fields
pub fn object_schema_with_required_strings(fields: &[&str]) -> JsonSchema {
    let properties: HashMap<String, JsonSchemaProperty> = fields
        .iter()
        .map(|f| {
            (
                f.to_string(),
                JsonSchemaProperty {
                    property_type: "string".to_string(),
                    description: None,
                    default: None,
                    enum_values: None,
                },
            )
        })
        .collect();

    JsonSchema {
        schema_type: "object".to_string(),
        properties: Some(properties),
        required: Some(fields.iter().map(|s| s.to_string()).collect()),
        items: None,
        description: None,
    }
}
