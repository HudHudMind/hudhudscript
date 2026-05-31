use hudhudscript_tools_schema::registry::{RegistryError, ToolRegistry};
use hudhudscript_tools_schema::schema::{JsonSchema, ToolMetadata, ToolSchema};
use serde_json::Value;

use super::ToolError;

/// Implement this trait to register a custom Rust tool with the `ToolRegistry`.
///
/// # Example
/// ```
/// use hudhudscript_tools_io::standard::{CustomTool, ToolError};
/// use serde_json::{json, Value};
///
/// struct GreetTool;
///
/// impl CustomTool for GreetTool {
///     fn name(&self) -> &str { "greet" }
///     fn description(&self) -> &str { "Greet someone by name" }
///     fn server(&self) -> &str { "built-in" }
///     fn parameter_schema(&self) -> Value {
///         json!({
///             "type": "object",
///             "properties": { "name": { "type": "string" } },
///             "required": ["name"]
///         })
///     }
///     fn call(&self, args: &Value) -> Result<Value, ToolError> {
///         let name = args["name"].as_str().unwrap_or("world");
///         Ok(json!({ "message": format!("Hello, {}!", name) }))
///     }
/// }
/// ```
pub trait CustomTool: Send + Sync {
    /// Unique tool name (snake_case recommended)
    fn name(&self) -> &str;

    /// Human-readable description used by LLMs to decide when to call this tool
    fn description(&self) -> &str;

    /// Server / namespace this tool belongs to (use `"built-in"` for native tools)
    fn server(&self) -> &str;

    /// JSON Schema for the tool's parameters
    fn parameter_schema(&self) -> Value;

    /// Execute the tool with validated arguments
    fn call(&self, args: &Value) -> Result<Value, ToolError>;

    /// Register this tool into a `ToolRegistry`
    fn register(&self, registry: &ToolRegistry) -> Result<(), RegistryError> {
        let input_schema: JsonSchema = serde_json::from_value(self.parameter_schema())
            .map_err(|e| RegistryError::DiscoveryFailed(e.to_string()))?;

        let schema = ToolSchema {
            name: self.name().to_string(),
            description: Some(self.description().to_string()),
            input_schema,
            server: self.server().to_string(),
        };

        let metadata = ToolMetadata::new(
            self.name().to_string(),
            self.server().to_string(),
            Some(self.description().to_string()),
        );

        registry.register_tool(schema, metadata)
    }
}
