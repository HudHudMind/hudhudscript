//! Dynamic Tool Discovery via OpenAPI / Swagger (Issue #121)
//!
//! Parses a subset of OpenAPI 3.x (or 2.x / Swagger) JSON to extract tool
//! definitions that can be registered in the [`ToolRegistry`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, warn};

use hudhudscript_tools_schema::registry::{RegistryError, ToolRegistry};
use hudhudscript_tools_schema::schema::{JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema};

// ---------------------------------------------------------------------------
// OpenAPI data model (minimal subset needed for tool discovery)
// ---------------------------------------------------------------------------

/// Top-level OpenAPI document.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenApiDocument {
    /// OpenAPI version string (e.g. "3.1.0" or "2.0").
    #[serde(rename = "openapi", default)]
    pub openapi: Option<String>,
    /// Swagger version (present in 2.x documents).
    #[serde(rename = "swagger", default)]
    pub swagger: Option<String>,
    /// API metadata.
    pub info: Option<OpenApiInfo>,
    /// Path objects keyed by path string.
    #[serde(default)]
    pub paths: HashMap<String, OpenApiPathItem>,
}

/// OpenAPI `info` object.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenApiInfo {
    pub title: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

/// All HTTP operations defined for a single path.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenApiPathItem {
    pub get: Option<OpenApiOperation>,
    pub post: Option<OpenApiOperation>,
    pub put: Option<OpenApiOperation>,
    pub delete: Option<OpenApiOperation>,
    pub patch: Option<OpenApiOperation>,
}

/// A single HTTP operation (GET, POST, …).
#[derive(Debug, Clone, Deserialize)]
pub struct OpenApiOperation {
    /// Unique machine-readable ID (becomes the tool name when present).
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Vec<OpenApiParameter>,
    /// Request body (POST/PUT/PATCH).
    #[serde(rename = "requestBody")]
    pub request_body: Option<OpenApiRequestBody>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// An individual query / path / header parameter.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenApiParameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String, // "query", "path", "header", "cookie"
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    pub schema: Option<Value>,
}

/// Request body definition.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenApiRequestBody {
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    pub content: HashMap<String, OpenApiMediaType>,
}

/// Media type entry inside a request body.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenApiMediaType {
    pub schema: Option<Value>,
}

// ---------------------------------------------------------------------------
// Discovery result
// ---------------------------------------------------------------------------

/// A tool extracted from an OpenAPI spec, before it is registered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredTool {
    /// Suggested tool name (derived from `operationId` or path+method).
    pub name: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// JSON Schema for the tool's parameters.
    pub parameters: JsonSchema,
    /// HTTP method (GET, POST, …).
    pub method: String,
    /// Path template (e.g. `/users/{id}`).
    pub path: String,
    /// Tags from the OpenAPI spec.
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Discovery logic
// ---------------------------------------------------------------------------

/// Error type for OpenAPI discovery.
#[derive(Debug)]
pub enum OpenApiError {
    ParseError(String),
    RegistryError(RegistryError),
}

impl std::fmt::Display for OpenApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            OpenApiError::ParseError(s) => write!(f, "Failed to parse OpenAPI document: {}", s),
            OpenApiError::RegistryError(e) => {
                write!(f, "Registry error while importing tools: {}", e)
            }
        }
    }
}

impl std::error::Error for OpenApiError {}

impl From<RegistryError> for OpenApiError {
    fn from(e: RegistryError) -> Self {
        OpenApiError::RegistryError(e)
    }
}

/// Parse an OpenAPI JSON string and return all discovered tools.
pub fn discover_tools_from_openapi(json: &str) -> Result<Vec<DiscoveredTool>, OpenApiError> {
    let doc: OpenApiDocument =
        serde_json::from_str(json).map_err(|e| OpenApiError::ParseError(e.to_string()))?;

    let server_name = doc
        .info
        .as_ref()
        .and_then(|i| i.title.clone())
        .unwrap_or_else(|| "openapi".to_string());

    let mut tools = Vec::new();

    for (path, path_item) in &doc.paths {
        let ops: Vec<(&str, Option<&OpenApiOperation>)> = vec![
            ("GET", path_item.get.as_ref()),
            ("POST", path_item.post.as_ref()),
            ("PUT", path_item.put.as_ref()),
            ("DELETE", path_item.delete.as_ref()),
            ("PATCH", path_item.patch.as_ref()),
        ];

        for (method, op_opt) in ops {
            let Some(op) = op_opt else { continue };

            let name = derive_tool_name(op, method, path);
            let description = op.summary.clone().or_else(|| op.description.clone());

            let parameters = build_parameters_schema(op);

            let tool = DiscoveredTool {
                name,
                description,
                parameters,
                method: method.to_string(),
                path: path.clone(),
                tags: op.tags.clone(),
            };

            debug!(
                tool = tool.name.as_str(),
                method, path, "Discovered tool from OpenAPI spec"
            );
            tools.push(tool);
        }
    }

    debug!(
        count = tools.len(),
        server = server_name.as_str(),
        "OpenAPI tool discovery complete"
    );

    Ok(tools)
}

/// Import discovered tools from an OpenAPI spec JSON into a [`ToolRegistry`].
///
/// Returns the number of tools imported.
pub fn import_openapi_tools(
    registry: &ToolRegistry,
    json: &str,
    server_name: &str,
) -> Result<usize, OpenApiError> {
    let discovered = discover_tools_from_openapi(json)?;
    let count = discovered.len();

    for tool in discovered {
        let schema = ToolSchema {
            name: tool.name.clone(),
            description: tool.description.clone(),
            input_schema: tool.parameters,
            server: server_name.to_string(),
        };

        let mut metadata =
            ToolMetadata::new(tool.name.clone(), server_name.to_string(), tool.description);
        for tag in &tool.tags {
            metadata.add_tag(tag.clone());
        }

        if let Err(e) = registry.register_tool(schema, metadata) {
            warn!(tool = tool.name.as_str(), error = %e, "Failed to register OpenAPI tool");
        }
    }

    Ok(count)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derive a snake_case tool name from the operation.
pub fn derive_tool_name(op: &OpenApiOperation, method: &str, path: &str) -> String {
    if let Some(ref id) = op.operation_id {
        return sanitize_name(id);
    }
    // Fall back to method + path segments
    let path_part = path
        .trim_start_matches('/')
        .replace('/', "_")
        .replace(['{', '}'], "");
    format!("{}_{}", method.to_lowercase(), sanitize_name(&path_part))
}

/// Replace non-alphanumeric characters (except `_`) with `_`.
pub fn sanitize_name(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

/// Build a JSON Schema object from operation parameters and request body.
pub fn build_parameters_schema(op: &OpenApiOperation) -> JsonSchema {
    let mut properties: HashMap<String, JsonSchemaProperty> = HashMap::new();
    let mut required: Vec<String> = Vec::new();

    // Parameters (query, path, header, cookie)
    for param in &op.parameters {
        let prop_type = extract_param_type(param.schema.as_ref());
        let prop = JsonSchemaProperty {
            property_type: prop_type,
            description: param.description.clone(),
            default: None,
            enum_values: None,
        };
        properties.insert(param.name.clone(), prop);
        if param.required {
            required.push(param.name.clone());
        }
    }

    // Request body — flatten top-level properties if application/json schema is an object
    if let Some(ref body) = op.request_body {
        if let Some(media) = body.content.get("application/json") {
            if let Some(ref schema_val) = media.schema {
                if let Some(obj) = schema_val.get("properties").and_then(|p| p.as_object()) {
                    for (key, prop_val) in obj {
                        let prop_type = prop_val
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("string")
                            .to_string();
                        let desc = prop_val
                            .get("description")
                            .and_then(|d| d.as_str())
                            .map(|s| s.to_string());
                        properties.insert(
                            key.clone(),
                            JsonSchemaProperty {
                                property_type: prop_type,
                                description: desc,
                                default: None,
                                enum_values: None,
                            },
                        );
                    }
                }
                // Collect required from body schema
                if let Some(req_arr) = schema_val.get("required").and_then(|r| r.as_array()) {
                    for r in req_arr {
                        if let Some(name) = r.as_str() {
                            if !required.contains(&name.to_string()) {
                                required.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    JsonSchema {
        schema_type: "object".to_string(),
        properties: if properties.is_empty() {
            None
        } else {
            Some(properties)
        },
        required: if required.is_empty() {
            None
        } else {
            Some(required)
        },
        items: None,
        description: None,
    }
}

pub fn extract_param_type(schema: Option<&Value>) -> String {
    schema
        .and_then(|s| s.get("type"))
        .and_then(|t| t.as_str())
        .unwrap_or("string")
        .to_string()
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl OpenApiError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            OpenApiError::ParseError(..) => hudhudscript_errors::ErrorCode::OpenApiParseError,
            OpenApiError::RegistryError(..) => hudhudscript_errors::ErrorCode::OpenApiRegistryError,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<OpenApiError> for hudhudscript_errors::Error {
    fn from(e: OpenApiError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
