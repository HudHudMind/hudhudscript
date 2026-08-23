use hudhudscript_tools_io::standard::*;
use hudhudscript_tools_io::*;
use serde_json::json;

// DatabaseBackend

#[test]
fn database_backend_display_postgres() {
    assert_eq!(format!("{}", DatabaseBackend::Postgres), "postgres");
}

#[test]
fn database_backend_display_sqlite() {
    assert_eq!(format!("{}", DatabaseBackend::Sqlite), "sqlite");
}

#[test]
fn database_backend_eq() {
    assert_eq!(DatabaseBackend::Postgres, DatabaseBackend::Postgres);
    assert_ne!(DatabaseBackend::Postgres, DatabaseBackend::Sqlite);
}

#[test]
fn database_backend_clone() {
    let b = DatabaseBackend::Sqlite;
    let b2 = b.clone();
    assert_eq!(b, b2);
}

#[test]
fn database_backend_serialize_deserialize() {
    let b = DatabaseBackend::Postgres;
    let json = serde_json::to_string(&b).unwrap();
    assert!(json.contains("postgres"));
    let back: DatabaseBackend = serde_json::from_str(&json).unwrap();
    assert_eq!(back, DatabaseBackend::Postgres);
}

// DatabaseConfig

#[test]
fn database_config_postgres() {
    let config = DatabaseConfig::postgres("postgres://localhost/testdb");
    assert_eq!(config.backend, DatabaseBackend::Postgres);
    assert_eq!(config.connection_string, "postgres://localhost/testdb");
    assert_eq!(config.max_connections, 10);
}

#[test]
fn database_config_sqlite() {
    let config = DatabaseConfig::sqlite("/tmp/test.db");
    assert_eq!(config.backend, DatabaseBackend::Sqlite);
    assert_eq!(config.connection_string, "sqlite:///tmp/test.db");
    assert_eq!(config.max_connections, 1);
}

#[test]
fn database_config_serialize_deserialize() {
    let config = DatabaseConfig::postgres("postgres://host/db");
    let json = serde_json::to_string(&config).unwrap();
    let back: DatabaseConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.backend, DatabaseBackend::Postgres);
    assert_eq!(back.connection_string, "postgres://host/db");
}

// QueryResult

#[test]
fn query_result_affected() {
    let result = QueryResult::affected(42, None);
    assert_eq!(result.rows_affected, 42);
    assert!(result.rows.is_empty());
    assert!(result.columns.is_empty());
}

#[test]
fn query_result_affected_zero() {
    let result = QueryResult::affected(0, None);
    assert_eq!(result.rows_affected, 0);
}

#[test]
fn query_result_with_rows() {
    let mut row = std::collections::HashMap::new();
    row.insert("id".to_string(), json!(1));
    row.insert("name".to_string(), json!("Alice"));
    let result = QueryResult {
        rows: vec![row],
        rows_affected: 0,
        columns: vec!["id".to_string(), "name".to_string()],
        column_types: vec!["INTEGER".to_string(), "TEXT".to_string()],
        last_insert_id: None,
        truncated: false,
    };
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.columns.len(), 2);
}

#[test]
fn query_result_serialize_deserialize() {
    let result = QueryResult::affected(10, None);
    let json = serde_json::to_string(&result).unwrap();
    let back: QueryResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.rows_affected, 10);
}

// ColumnInfo

#[test]
fn column_info_construction() {
    let col = ColumnInfo {
        name: "email".to_string(),
        data_type: "varchar".to_string(),
        nullable: true,
        primary_key: false,
        default: None,
    };
    assert_eq!(col.name, "email");
    assert_eq!(col.data_type, "varchar");
    assert!(col.nullable);
}

#[test]
fn column_info_serialize_deserialize() {
    let col = ColumnInfo {
        name: "age".to_string(),
        data_type: "integer".to_string(),
        nullable: false,
        primary_key: true,
        default: None,
    };
    let json = serde_json::to_string(&col).unwrap();
    let back: ColumnInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "age");
    assert!(!back.nullable);
}

// DatabaseError

#[test]
fn database_error_connection_failed() {
    let err = DatabaseError::ConnectionFailed("timeout".to_string());
    assert!(format!("{err}").contains("timeout"));
}

#[test]
fn database_error_query_failed() {
    let err = DatabaseError::QueryFailed("syntax error".to_string());
    assert!(format!("{err}").contains("syntax error"));
}

#[test]
fn database_error_unsupported_backend() {
    let err = DatabaseError::UnsupportedBackend("oracle".to_string());
    assert!(format!("{err}").contains("oracle"));
}

#[test]
fn database_error_feature_not_enabled() {
    let err = DatabaseError::FeatureNotEnabled;
    let msg = format!("{err}");
    assert!(msg.contains("feature") || msg.contains("db"));
}

#[test]
fn database_error_invalid_arguments() {
    let err = DatabaseError::InvalidArguments("missing SQL".to_string());
    assert!(format!("{err}").contains("missing SQL"));
}

// DatabaseTool

#[test]
fn database_tool_new_postgres() {
    let config = DatabaseConfig::postgres("postgres://localhost/test");
    let tool = DatabaseTool::new(config);
    assert_eq!(*tool.backend(), DatabaseBackend::Postgres);
}

#[test]
fn database_tool_new_sqlite() {
    let config = DatabaseConfig::sqlite("/tmp/test.db");
    let tool = DatabaseTool::new(config);
    assert_eq!(*tool.backend(), DatabaseBackend::Sqlite);
}

#[tokio::test]
async fn database_tool_executes_real_sqlite_query() {
    let config = DatabaseConfig::sqlite(":memory:");
    let tool = DatabaseTool::new(config);
    let result = tool
        .execute_query("SELECT 1 AS value", &[])
        .await
        .expect("query SQLite");
    assert_eq!(result.rows[0]["value"], json!(1));
}

#[tokio::test]
async fn database_tool_lists_real_sqlite_tables() {
    let config = DatabaseConfig::sqlite(":memory:");
    let tool = DatabaseTool::new(config);
    let result = tool.list_tables().await.expect("list SQLite tables");
    assert!(result.is_empty());
}

#[tokio::test]
async fn database_tool_describes_real_sqlite_table() {
    let config = DatabaseConfig::sqlite(":memory:");
    let tool = DatabaseTool::new(config);
    tool.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
        &[],
    )
    .await
    .expect("create users");
    let result = tool.describe_table("users").await.expect("describe users");
    assert_eq!(result.len(), 2);
    assert!(result[0].primary_key);
}

#[test]
fn tool_error_invalid_arguments() {
    let err = ToolError::InvalidArguments("missing field".to_string());
    assert!(format!("{err}").contains("missing field"));
}

#[test]
fn tool_error_execution_failed() {
    let err = ToolError::ExecutionFailed("crash".to_string());
    assert!(format!("{err}").contains("crash"));
}

#[test]
fn standard_tool_json_parse_success() {
    let result = StandardTool::JsonParse.call(&json!({"text": "{\"a\":1}"}));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["value"]["a"], 1);
}

#[test]
fn standard_tool_json_parse_invalid() {
    let result = StandardTool::JsonParse.call(&json!({"text": "not json {{{"}));
    assert!(result.is_err());
}

#[test]
fn standard_tool_json_parse_missing_arg() {
    let result = StandardTool::JsonParse.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn standard_tool_file_read_missing_path() {
    let result = StandardTool::FileRead.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn standard_tool_file_read_nonexistent() {
    let result =
        StandardTool::FileRead.call(&json!({"path": "/tmp/nonexistent_hudhud_test_file_xyz"}));
    assert!(result.is_err());
}

#[test]
fn standard_tool_http_get_missing_url() {
    let result = StandardTool::HttpGet.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn standard_tool_http_post_missing_url() {
    let result = StandardTool::HttpPost.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn standard_tool_http_put_missing_url() {
    let result = StandardTool::HttpPut.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn standard_tool_http_delete_missing_url() {
    let result = StandardTool::HttpDelete.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn build_object_schema_basic() {
    let schema = build_object_schema(&[
        ("name", "string", true, Some("User name")),
        ("age", "integer", false, None),
    ]);
    assert_eq!(schema["type"], "object");
    let props = schema["properties"].as_object().unwrap();
    assert!(props.contains_key("name"));
    assert!(props.contains_key("age"));
    assert_eq!(props["name"]["description"], "User name");
}

#[test]
fn build_object_schema_required_fields() {
    let schema = build_object_schema(&[
        ("a", "string", true, None),
        ("b", "string", false, None),
        ("c", "integer", true, None),
    ]);
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 2);
}

#[test]
fn build_object_schema_empty() {
    let schema = build_object_schema(&[]);
    assert_eq!(schema["type"], "object");
    let required = schema["required"].as_array().unwrap();
    assert!(required.is_empty());
}

#[test]
fn object_schema_with_required_strings_basic() {
    let schema = object_schema_with_required_strings(&["name", "email"]);
    assert_eq!(schema.schema_type, "object");
    let props = schema.properties.as_ref().unwrap();
    assert!(props.contains_key("name"));
    assert!(props.contains_key("email"));
    assert_eq!(props["name"].property_type, "string");
    let required = schema.required.as_ref().unwrap();
    assert_eq!(required.len(), 2);
}

#[test]
fn object_schema_with_required_strings_empty() {
    let schema = object_schema_with_required_strings(&[]);
    let props = schema.properties.as_ref().unwrap();
    assert!(props.is_empty());
}

#[test]
fn standard_tool_file_read_real_file() {
    // Read a known-to-exist file
    let result = StandardTool::FileRead.call(&json!({
        "path": "../Cargo.toml"
    }));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value["contents"].as_str().unwrap().contains("[workspace]"));
    assert_eq!(
        value["path"],
        "../Cargo.toml"
    );
}

#[test]
fn standard_tool_json_parse_array() {
    let result = StandardTool::JsonParse.call(&json!({"text": "[1,2,3]"}));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value["value"].is_array());
}

#[test]
fn standard_tool_json_parse_string() {
    let result = StandardTool::JsonParse.call(&json!({"text": "\"hello\""}));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["value"], "hello");
}

#[test]
fn standard_tool_json_parse_number() {
    let result = StandardTool::JsonParse.call(&json!({"text": "42"}));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["value"], 42);
}

#[test]
fn standard_tool_json_parse_boolean() {
    let result = StandardTool::JsonParse.call(&json!({"text": "true"}));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["value"], true);
}

#[test]
fn standard_tool_json_parse_null() {
    let result = StandardTool::JsonParse.call(&json!({"text": "null"}));
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value["value"].is_null());
}
