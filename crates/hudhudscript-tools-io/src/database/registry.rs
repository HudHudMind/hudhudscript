#[cfg(feature = "db")]
use std::collections::HashMap;
#[cfg(feature = "db")]
use std::sync::Arc;

#[cfg(feature = "db")]
use hudhudscript_tools_schema::schema::{JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema};
use hudhudscript_tools_schema::{RegistryError, ToolRegistry};
#[cfg(feature = "db")]
use serde_json::Value;

use super::DatabaseConfig;
#[cfg(feature = "db")]
use super::{DatabaseTool, Migration};

/// Register executable database tools backed by one configured, lazily opened pool.
/// The connection URL is captured here and is never exposed in an agent tool schema.
#[cfg(feature = "db")]
pub fn register_database_tools(
    registry: &ToolRegistry,
    config: DatabaseConfig,
) -> Result<usize, RegistryError> {
    let tool = Arc::new(DatabaseTool::new(config));
    register(
        registry,
        "db_query",
        "Run a parameterized query and return rows",
        &[("sql", "string", true), ("params", "array", false)],
        {
            let tool = Arc::clone(&tool);
            move |arguments: Value| {
                let tool = Arc::clone(&tool);
                async move {
                    let sql = string_arg(&arguments, "sql")?;
                    let params = array_arg(&arguments, "params")?;
                    json(tool.execute_query(sql, params).await)
                }
            }
        },
    )?;
    register(
        registry,
        "db_execute",
        "Run a parameterized write statement",
        &[("sql", "string", true), ("params", "array", false)],
        {
            let tool = Arc::clone(&tool);
            move |arguments: Value| {
                let tool = Arc::clone(&tool);
                async move {
                    let sql = string_arg(&arguments, "sql")?;
                    let params = array_arg(&arguments, "params")?;
                    json(tool.execute(sql, params).await)
                }
            }
        },
    )?;
    register(registry, "db_list_tables", "List database tables", &[], {
        let tool = Arc::clone(&tool);
        move |_arguments: Value| {
            let tool = Arc::clone(&tool);
            async move { json(tool.list_tables().await) }
        }
    })?;
    register(
        registry,
        "db_describe_table",
        "Describe columns for one table",
        &[("table", "string", true)],
        {
            let tool = Arc::clone(&tool);
            move |arguments: Value| {
                let tool = Arc::clone(&tool);
                async move { json(tool.describe_table(string_arg(&arguments, "table")?).await) }
            }
        },
    )?;
    register(
        registry,
        "db_migrate",
        "Apply ordered, checksum-verified migrations",
        &[("migrations", "array", true)],
        {
            let tool = Arc::clone(&tool);
            move |arguments: Value| {
                let tool = Arc::clone(&tool);
                async move {
                    let migrations: Vec<Migration> = serde_json::from_value(
                        arguments.get("migrations").cloned().unwrap_or(Value::Null),
                    )
                    .map_err(|error| {
                        RegistryError::CallFailed(format!("invalid migrations: {error}"))
                    })?;
                    json(tool.migrate(migrations).await)
                }
            }
        },
    )?;
    Ok(5)
}

#[cfg(not(feature = "db"))]
pub fn register_database_tools(
    _registry: &ToolRegistry,
    _config: DatabaseConfig,
) -> Result<usize, RegistryError> {
    Err(RegistryError::CallFailed(
        "database support is not enabled".into(),
    ))
}

#[cfg(feature = "db")]
fn register<F, Fut>(
    registry: &ToolRegistry,
    name: &str,
    description: &str,
    fields: &[(&str, &str, bool)],
    handler: F,
) -> Result<(), RegistryError>
where
    F: Fn(Value) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Value, RegistryError>> + Send + 'static,
{
    let properties: HashMap<String, JsonSchemaProperty> = fields
        .iter()
        .map(|(field, kind, _)| {
            (
                (*field).into(),
                JsonSchemaProperty {
                    property_type: (*kind).into(),
                    description: None,
                    default: None,
                    enum_values: None,
                },
            )
        })
        .collect();
    let required = fields
        .iter()
        .filter(|(_, _, required)| *required)
        .map(|(field, _, _)| (*field).to_string())
        .collect();
    let schema = ToolSchema {
        name: name.into(),
        description: Some(description.into()),
        server: "built-in".into(),
        input_schema: JsonSchema {
            schema_type: "object".into(),
            properties: Some(properties),
            required: Some(required),
            items: None,
            description: Some(description.into()),
        },
    };
    let mut metadata = ToolMetadata::new(name.into(), "built-in".into(), Some(description.into()));
    metadata.add_tag("database".into());
    registry.register_native_tool(schema, metadata, Arc::new(handler))
}

#[cfg(feature = "db")]
fn string_arg<'a>(arguments: &'a Value, name: &str) -> Result<&'a str, RegistryError> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| RegistryError::CallFailed(format!("missing string argument '{name}'")))
}

#[cfg(feature = "db")]
fn array_arg<'a>(arguments: &'a Value, name: &str) -> Result<&'a [Value], RegistryError> {
    match arguments.get(name) {
        None | Some(Value::Null) => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        Some(_) => Err(RegistryError::CallFailed(format!(
            "argument '{name}' must be an array"
        ))),
    }
}

#[cfg(feature = "db")]
fn json<T: serde::Serialize>(
    result: Result<T, super::DatabaseError>,
) -> Result<Value, RegistryError> {
    result
        .map_err(|error| RegistryError::CallFailed(error.to_string()))
        .and_then(|value| {
            serde_json::to_value(value)
                .map_err(|error| RegistryError::CallFailed(error.to_string()))
        })
}
