use hudhudscript_sandbox::FileSystemSandbox;
use hudhudscript_sandbox::SandboxConfig;
use hudhudscript_tools_schema::registry::RegistryError;
use hudhudscript_tools_schema::schema::{JsonSchema, ToolMetadata, ToolSchema};
use serde_json::Value;

use super::sandbox::{check_url_against_sandbox, AMBIENT_SANDBOX};
use super::ToolError;

/// Enumeration of the built-in standard tools
pub enum StandardTool {
    /// Read a file from the local filesystem
    FileRead,
    /// Perform an HTTP GET request
    HttpGet,
    /// Perform an HTTP POST request
    HttpPost,
    /// Perform an HTTP PUT request
    HttpPut,
    /// Perform an HTTP DELETE request
    HttpDelete,
    /// Parse a JSON string into a structured value
    JsonParse,
}

impl StandardTool {
    fn name(&self) -> &'static str {
        match self {
            StandardTool::FileRead => "file_read",
            StandardTool::HttpGet => "http_get",
            StandardTool::HttpPost => "http_post",
            StandardTool::HttpPut => "http_put",
            StandardTool::HttpDelete => "http_delete",
            StandardTool::JsonParse => "json_parse",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            StandardTool::FileRead => "Read the contents of a file at the given path",
            StandardTool::HttpGet => "Perform an HTTP GET request and return the response body",
            StandardTool::HttpPost => "Perform an HTTP POST request with a JSON body",
            StandardTool::HttpPut => "Perform an HTTP PUT request with a JSON body",
            StandardTool::HttpDelete => "Perform an HTTP DELETE request",
            StandardTool::JsonParse => "Parse a JSON string and return the structured value",
        }
    }

    fn parameter_schema(&self) -> Value {
        match self {
            StandardTool::FileRead => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute path to the file"
                    }
                },
                "required": ["path"]
            }),
            StandardTool::HttpGet | StandardTool::HttpDelete => serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to fetch"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Optional request headers"
                    }
                },
                "required": ["url"]
            }),
            StandardTool::HttpPost | StandardTool::HttpPut => serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to send request to"
                    },
                    "body": {
                        "type": "object",
                        "description": "JSON body to send"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Optional request headers"
                    }
                },
                "required": ["url"]
            }),
            StandardTool::JsonParse => serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "JSON string to parse"
                    }
                },
                "required": ["text"]
            }),
        }
    }

    /// Execute the standard tool without sandbox enforcement.
    pub fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let ambient = AMBIENT_SANDBOX.with(|s| s.borrow().clone());
        self.call_sandboxed(args, ambient.as_ref())
    }

    /// Execute the standard tool, enforcing sandbox policy when provided.
    pub fn call_sandboxed(
        &self,
        args: &Value,
        sandbox: Option<&SandboxConfig>,
    ) -> Result<Value, ToolError> {
        match self {
            StandardTool::FileRead => {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArguments("missing 'path'".into()))?;

                if let Some(cfg) = sandbox {
                    let fs_sandbox = FileSystemSandbox::new(cfg.filesystem.clone());
                    fs_sandbox.check_access(path, false).map_err(|e| {
                        ToolError::SecurityViolation(format!(
                            "file_read denied for path '{}': {}",
                            path, e
                        ))
                    })?;
                }

                match std::fs::read_to_string(path) {
                    Ok(contents) => Ok(serde_json::json!({ "contents": contents, "path": path })),
                    Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
                }
            }

            StandardTool::HttpGet => {
                let url_str = args["url"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArguments("missing 'url'".into()))?;

                if let Some(cfg) = sandbox {
                    check_url_against_sandbox(url_str, cfg)?;
                }

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let mut req = client.get(url_str);
                if let Some(headers) = args.get("headers").and_then(|h| h.as_object()) {
                    for (k, v) in headers {
                        if let Some(val) = v.as_str() {
                            req = req.header(k.as_str(), val);
                        }
                    }
                }
                let resp = req
                    .send()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let status = resp.status().as_u16();
                let body = resp
                    .text()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let json_body: Value =
                    serde_json::from_str(&body).unwrap_or(Value::String(body.clone()));
                Ok(serde_json::json!({
                    "status": status,
                    "ok": status >= 200 && status < 300,
                    "body": body,
                    "json": json_body
                }))
            }

            StandardTool::HttpPost | StandardTool::HttpPut => {
                let url_str = args["url"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArguments("missing 'url'".into()))?;

                if let Some(cfg) = sandbox {
                    check_url_against_sandbox(url_str, cfg)?;
                }

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let mut req = match self {
                    StandardTool::HttpPost => client.post(url_str),
                    _ => client.put(url_str),
                };
                if let Some(body) = args.get("body") {
                    req = req.json(body);
                }
                if let Some(headers) = args.get("headers").and_then(|h| h.as_object()) {
                    for (k, v) in headers {
                        if let Some(val) = v.as_str() {
                            req = req.header(k.as_str(), val);
                        }
                    }
                }
                let resp = req
                    .send()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let status = resp.status().as_u16();
                let body = resp
                    .text()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let json_body: Value =
                    serde_json::from_str(&body).unwrap_or(Value::String(body.clone()));
                Ok(serde_json::json!({
                    "status": status,
                    "ok": status >= 200 && status < 300,
                    "body": body,
                    "json": json_body
                }))
            }

            StandardTool::HttpDelete => {
                let url_str = args["url"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArguments("missing 'url'".into()))?;

                if let Some(cfg) = sandbox {
                    check_url_against_sandbox(url_str, cfg)?;
                }

                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let mut req = client.delete(url_str);
                if let Some(headers) = args.get("headers").and_then(|h| h.as_object()) {
                    for (k, v) in headers {
                        if let Some(val) = v.as_str() {
                            req = req.header(k.as_str(), val);
                        }
                    }
                }
                let resp = req
                    .send()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                let status = resp.status().as_u16();
                let body = resp
                    .text()
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                Ok(serde_json::json!({
                    "status": status,
                    "ok": status >= 200 && status < 300,
                    "body": body
                }))
            }

            StandardTool::JsonParse => {
                let text = args["text"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArguments("missing 'text'".into()))?;

                serde_json::from_str::<Value>(text)
                    .map(|v| serde_json::json!({ "value": v }))
                    .map_err(|e| ToolError::ExecutionFailed(format!("JSON parse error: {e}")))
            }
        }
    }

    /// Build a `ToolSchema` for this standard tool
    pub(crate) fn to_schema(&self) -> Result<ToolSchema, RegistryError> {
        let input_schema: JsonSchema = serde_json::from_value(self.parameter_schema())
            .map_err(|e| RegistryError::DiscoveryFailed(e.to_string()))?;

        Ok(ToolSchema {
            name: self.name().to_string(),
            description: Some(self.description().to_string()),
            input_schema,
            server: "built-in".to_string(),
        })
    }

    /// Build `ToolMetadata` for this standard tool
    pub(crate) fn to_metadata(&self) -> ToolMetadata {
        let mut meta = ToolMetadata::new(
            self.name().to_string(),
            "built-in".to_string(),
            Some(self.description().to_string()),
        );
        meta.add_tag("standard".to_string());
        meta
    }
}
