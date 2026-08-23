use hudhudscript_tools::registry::ToolRegistry;
use hudhudscript_tools::standard::{
    build_object_schema, object_schema_with_required_strings, register_standard_tools, CustomTool,
    StandardTool, ToolError,
};
use serde_json::{json, Value};

#[test]
fn test_json_parse_tool() {
    let result = StandardTool::JsonParse
        .call(&json!({ "text": r#"{"key": "value"}"# }))
        .unwrap();
    assert_eq!(result["value"]["key"], "value");
}

#[test]
fn test_json_parse_invalid() {
    let result = StandardTool::JsonParse.call(&json!({ "text": "not json" }));
    assert!(result.is_err());
}

#[test]
fn test_register_standard_tools() {
    let registry = ToolRegistry::new();
    let count = register_standard_tools(&registry).unwrap();
    assert_eq!(count, 6);
    // Core tools
    assert!(registry.get_tool("file_read").is_some());
    assert!(registry.get_tool("http_get").is_some());
    assert!(registry.get_tool("http_post").is_some());
    assert!(registry.get_tool("http_put").is_some());
    assert!(registry.get_tool("http_delete").is_some());
    assert!(registry.get_tool("json_parse").is_some());
    // Database tools require an explicit DatabaseConfig so secrets and access
    // cannot be introduced by the generic standard-tool registration path.
    assert!(registry.get_tool("db_query").is_none());
    assert!(registry.get_tool("db_list_tables").is_none());
    // Git tools (Issue #22)
    assert!(registry.get_tool("git_status").is_some());
    assert!(registry.get_tool("git_commit").is_some());
    assert!(registry.get_tool("git_push").is_some());
    assert!(registry.get_tool("git_branch").is_some());
    assert!(registry.get_tool("git_checkout").is_some());
    assert!(registry.get_tool("git_log").is_some());
}

#[test]
fn test_custom_tool_trait() {
    struct GreetTool;

    impl CustomTool for GreetTool {
        fn name(&self) -> &str {
            "greet"
        }
        fn description(&self) -> &str {
            "Greet someone"
        }
        fn server(&self) -> &str {
            "built-in"
        }
        fn parameter_schema(&self) -> Value {
            json!({
                "type": "object",
                "properties": { "name": { "type": "string" } },
                "required": ["name"]
            })
        }
        fn call(&self, args: &Value) -> Result<Value, ToolError> {
            let name = args["name"].as_str().unwrap_or("world");
            Ok(json!({ "greeting": format!("Hello, {}!", name) }))
        }
    }

    let registry = ToolRegistry::new();
    let tool = GreetTool;
    tool.register(&registry).unwrap();

    assert!(registry.get_tool("greet").is_some());

    let result = tool.call(&json!({ "name": "Alice" })).unwrap();
    assert_eq!(result["greeting"], "Hello, Alice!");
}

#[test]
fn test_build_object_schema() {
    let schema = build_object_schema(&[
        ("path", "string", true, Some("File path")),
        ("encoding", "string", false, None),
    ]);
    assert_eq!(schema["type"], "object");
    assert!(schema["required"]
        .as_array()
        .unwrap()
        .contains(&json!("path")));
}

#[test]
fn test_standard_tool_json_parse_array() {
    let result = StandardTool::JsonParse
        .call(&json!({ "text": "[1, 2, 3]" }))
        .unwrap();
    assert_eq!(result["value"][0], 1);
    assert_eq!(result["value"][1], 2);
    assert_eq!(result["value"][2], 3);
}

#[test]
fn test_standard_tool_json_parse_missing_text() {
    let result = StandardTool::JsonParse.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn test_standard_tool_file_read_missing_path() {
    let result = StandardTool::FileRead.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn test_standard_tool_file_read_nonexistent() {
    let result = StandardTool::FileRead.call(&json!({"path": "/tmp/nonexistent_hudhud_99999.txt"}));
    assert!(result.is_err());
}

#[test]
fn test_object_schema_with_required_strings() {
    let schema = object_schema_with_required_strings(&["a", "b", "c"]);
    assert_eq!(schema.schema_type, "object");
    let props = schema.properties.as_ref().unwrap();
    assert_eq!(props.len(), 3);
    for key in &["a", "b", "c"] {
        assert_eq!(props[*key].property_type, "string");
    }
    let required = schema.required.as_ref().unwrap();
    assert_eq!(required.len(), 3);
}

#[test]
fn test_tool_error_display() {
    let err = ToolError::InvalidArguments("bad".to_string());
    assert!(err.to_string().contains("Invalid arguments: bad"));

    let err = ToolError::ExecutionFailed("timeout".to_string());
    assert!(err.to_string().contains("Execution failed: timeout"));
}

#[test]
fn test_register_standard_tools_includes_metadata() {
    let registry = ToolRegistry::new();
    register_standard_tools(&registry).unwrap();

    let meta = registry.get_metadata("file_read").unwrap();
    assert_eq!(meta.server, "built-in");
    assert!(meta.tags.contains(&"standard".to_string()));
}
