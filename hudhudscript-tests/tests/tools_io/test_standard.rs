//! External tests for hudhudscript-tools-io — standard tools, database config, schemas

use hudhudscript_tools_io::database::{
    register_database_tools, DatabaseBackend, DatabaseConfig, DatabaseTool, QueryResult,
};
use hudhudscript_tools_io::standard::{register_standard_tools, StandardTool, ToolError};
use hudhudscript_tools_schema::registry::ToolRegistry;
use serde_json::json;

// ---------------------------------------------------------------------------
// Helper: create a populated registry
// ---------------------------------------------------------------------------

fn make_registry() -> ToolRegistry {
    let r = ToolRegistry::new();
    register_standard_tools(&r).unwrap();
    r
}

// ---------------------------------------------------------------------------
// Schema checks via registry (parameter_schema is private on StandardTool)
// ---------------------------------------------------------------------------

#[test]
fn file_read_schema_requires_path_property() {
    let reg = make_registry();
    let schema = reg.get_tool("file_read").expect("file_read not registered");
    let props = schema.input_schema.properties.as_ref().unwrap();
    assert!(
        props.contains_key("path"),
        "file_read must have 'path' property"
    );
    assert_eq!(props["path"].property_type, "string");

    let required = schema.input_schema.required.as_ref().unwrap();
    assert!(required.contains(&"path".to_string()));
}

#[test]
fn http_post_schema_has_url_body_headers() {
    let reg = make_registry();
    let schema = reg.get_tool("http_post").expect("http_post not registered");
    let props = schema.input_schema.properties.as_ref().unwrap();
    assert!(props.contains_key("url"));
    assert!(props.contains_key("body"));
    assert!(props.contains_key("headers"));

    let required = schema.input_schema.required.as_ref().unwrap();
    assert!(required.contains(&"url".to_string()));
    // body and headers are optional
    assert!(!required.contains(&"body".to_string()));
}

#[test]
fn json_parse_schema_requires_text() {
    let reg = make_registry();
    let schema = reg
        .get_tool("json_parse")
        .expect("json_parse not registered");
    let required = schema.input_schema.required.as_ref().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "text");
}

// ---------------------------------------------------------------------------
// StandardTool::call — json_parse success / failure
// ---------------------------------------------------------------------------

#[test]
fn json_parse_returns_parsed_value() {
    let tool = StandardTool::JsonParse;
    let result = tool.call(&json!({"text": r#"{"a":1,"b":[2,3]}"#})).unwrap();
    let value = &result["value"];
    assert_eq!(value["a"], json!(1));
    assert_eq!(value["b"], json!([2, 3]));
}

#[test]
fn json_parse_invalid_json_returns_error() {
    let tool = StandardTool::JsonParse;
    let err = tool.call(&json!({"text": "{not valid json"})).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("JSON parse error"),
        "expected JSON parse error, got: {msg}"
    );
}

#[test]
fn file_read_missing_path_returns_invalid_arguments() {
    let tool = StandardTool::FileRead;
    let err = tool.call(&json!({})).unwrap_err();
    match err {
        ToolError::InvalidArguments(msg) => assert!(msg.contains("path")),
        other => panic!("expected InvalidArguments, got: {other}"),
    }
}

#[test]
fn http_get_missing_url_returns_invalid_arguments() {
    let tool = StandardTool::HttpGet;
    let err = tool.call(&json!({})).unwrap_err();
    match err {
        ToolError::InvalidArguments(msg) => assert!(msg.contains("url")),
        other => panic!("expected InvalidArguments, got: {other}"),
    }
}

// ---------------------------------------------------------------------------
// register_standard_tools — registry integration
// ---------------------------------------------------------------------------

#[test]
fn register_standard_tools_populates_registry_with_all_tools() {
    let registry = ToolRegistry::new();
    let count = register_standard_tools(&registry).unwrap();
    assert_eq!(count, 6, "expected 6 standard tools");

    let names = registry.list_tools();
    for expected in &[
        "file_read",
        "http_get",
        "http_post",
        "http_put",
        "http_delete",
        "json_parse",
    ] {
        assert!(
            names.contains(&expected.to_string()),
            "registry should contain '{expected}', found: {names:?}"
        );
    }
}

#[test]
fn registered_standard_tool_metadata_has_standard_tag() {
    let reg = make_registry();
    let meta = reg
        .get_metadata("file_read")
        .expect("file_read metadata missing");
    assert!(
        meta.tags.contains(&"standard".to_string()),
        "file_read metadata should have 'standard' tag, got: {:?}",
        meta.tags
    );
    assert_eq!(meta.server, "built-in");
}

#[test]
fn registered_database_tool_metadata_has_database_tag() {
    let reg = ToolRegistry::new();
    register_database_tools(&reg, DatabaseConfig::sqlite(":memory:")).unwrap();
    let meta = reg
        .get_metadata("db_query")
        .expect("db_query metadata missing");
    assert!(meta.tags.contains(&"database".to_string()));
}

// ---------------------------------------------------------------------------
// DatabaseConfig constructors and defaults
// ---------------------------------------------------------------------------

#[test]
fn database_config_postgres_defaults() {
    let cfg = DatabaseConfig::postgres("postgres://localhost/test");
    assert_eq!(cfg.backend, DatabaseBackend::Postgres);
    assert_eq!(cfg.connection_string, "postgres://localhost/test");
    assert_eq!(cfg.max_connections, 10);
}

#[test]
fn database_config_sqlite_defaults() {
    let cfg = DatabaseConfig::sqlite("/tmp/test.db");
    assert_eq!(cfg.backend, DatabaseBackend::Sqlite);
    assert_eq!(cfg.connection_string, "sqlite:///tmp/test.db");
    assert_eq!(cfg.max_connections, 1);
}

// ---------------------------------------------------------------------------
// DatabaseBackend display + serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn database_backend_display_and_serde_roundtrip() {
    assert_eq!(DatabaseBackend::Postgres.to_string(), "postgres");
    assert_eq!(DatabaseBackend::Sqlite.to_string(), "sqlite");

    // serde round-trip
    let serialized = serde_json::to_string(&DatabaseBackend::Postgres).unwrap();
    assert_eq!(serialized, r#""postgres""#);
    let deserialized: DatabaseBackend = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, DatabaseBackend::Postgres);
}

// ---------------------------------------------------------------------------
// QueryResult::affected helper
// ---------------------------------------------------------------------------

#[test]
fn query_result_affected_has_empty_rows_and_columns() {
    let qr = QueryResult::affected(42, None);
    assert_eq!(qr.rows_affected, 42);
    assert!(qr.rows.is_empty());
    assert!(qr.columns.is_empty());
}

// ---------------------------------------------------------------------------
// DatabaseTool — real SQLite execution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn database_tool_executes_real_sqlite() {
    let cfg = DatabaseConfig::sqlite(":memory:");
    let tool = DatabaseTool::new(cfg);
    let result = tool
        .execute_query("SELECT 1 AS value", &[])
        .await
        .expect("query SQLite");
    assert_eq!(result.rows[0]["value"], json!(1));
    assert!(tool.list_tables().await.expect("list tables").is_empty());
}

// ---------------------------------------------------------------------------
// build_object_schema helper
// ---------------------------------------------------------------------------

#[test]
fn build_object_schema_produces_correct_required_and_properties() {
    let schema = hudhudscript_tools_io::standard::build_object_schema(&[
        ("name", "string", true, Some("The user name")),
        ("age", "integer", false, None),
        ("email", "string", true, Some("Email address")),
    ]);

    assert_eq!(schema["type"], "object");

    let required = schema["required"].as_array().unwrap();
    let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
    assert!(req_strs.contains(&"name"));
    assert!(req_strs.contains(&"email"));
    assert!(!req_strs.contains(&"age"));

    assert_eq!(schema["properties"]["name"]["type"], "string");
    assert_eq!(schema["properties"]["name"]["description"], "The user name");
    assert_eq!(schema["properties"]["age"]["type"], "integer");
    // age has no description
    assert!(schema["properties"]["age"].get("description").is_none());
}

// ---------------------------------------------------------------------------
// object_schema_with_required_strings helper
// ---------------------------------------------------------------------------

#[test]
fn object_schema_with_required_strings_builds_valid_json_schema() {
    let schema =
        hudhudscript_tools_io::standard::object_schema_with_required_strings(&["host", "port"]);

    assert_eq!(schema.schema_type, "object");
    let props = schema.properties.as_ref().unwrap();
    assert_eq!(props.len(), 2);
    assert_eq!(props["host"].property_type, "string");
    assert_eq!(props["port"].property_type, "string");

    let req = schema.required.as_ref().unwrap();
    assert!(req.contains(&"host".to_string()));
    assert!(req.contains(&"port".to_string()));
}
