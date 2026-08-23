use hudhudscript_tools::database::{
    register_database_tools, ColumnInfo, DatabaseBackend, DatabaseConfig, DatabaseError,
    DatabaseTool, QueryResult,
};
use hudhudscript_tools::registry::ToolRegistry;
use serde_json::json;

#[test]
fn test_database_backend_display() {
    assert_eq!(DatabaseBackend::Postgres.to_string(), "postgres");
    assert_eq!(DatabaseBackend::Mysql.to_string(), "mysql");
    assert_eq!(DatabaseBackend::Sqlite.to_string(), "sqlite");
}

#[test]
fn test_database_config_postgres() {
    let cfg = DatabaseConfig::postgres("postgres://localhost/mydb");
    assert_eq!(cfg.backend, DatabaseBackend::Postgres);
    assert_eq!(cfg.connection_string, "postgres://localhost/mydb");
    assert_eq!(cfg.max_connections, 10);
}

#[test]
fn test_database_config_sqlite() {
    let cfg = DatabaseConfig::sqlite("/tmp/test.db");
    assert_eq!(cfg.backend, DatabaseBackend::Sqlite);
    assert_eq!(cfg.connection_string, "sqlite:///tmp/test.db");
    assert_eq!(cfg.max_connections, 1);
}

#[test]
fn test_query_result_affected() {
    let result = QueryResult::affected(42, None);
    assert_eq!(result.rows_affected, 42);
    assert!(result.rows.is_empty());
    assert!(result.columns.is_empty());
}

#[tokio::test]
async fn test_execute_query_uses_real_sqlite() {
    let tool = DatabaseTool::new(DatabaseConfig::sqlite(":memory:"));
    let result = tool
        .execute_query("SELECT 1 AS value", &[])
        .await
        .expect("query SQLite");
    assert_eq!(result.rows[0]["value"], json!(1));
}

#[tokio::test]
async fn test_list_tables_uses_real_sqlite() {
    let tool = DatabaseTool::new(DatabaseConfig::sqlite(":memory:"));
    let result = tool.list_tables().await.expect("list SQLite tables");
    assert!(result.is_empty());
}

#[test]
fn test_register_database_tools() {
    let registry = ToolRegistry::new();
    let count = register_database_tools(&registry, DatabaseConfig::sqlite(":memory:")).unwrap();
    assert_eq!(count, 5);
    assert!(registry.get_tool("db_query").is_some());
    assert!(registry.get_tool("db_execute").is_some());
    assert!(registry.get_tool("db_list_tables").is_some());
    assert!(registry.get_tool("db_describe_table").is_some());
    assert!(registry.get_tool("db_migrate").is_some());
}

#[test]
fn test_database_tool_backend_accessor() {
    let tool = DatabaseTool::new(DatabaseConfig::postgres("postgres://localhost/app"));
    assert_eq!(*tool.backend(), DatabaseBackend::Postgres);
}

#[test]
fn test_column_info_struct() {
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
fn test_database_error_display() {
    let err = DatabaseError::ConnectionFailed("refused".to_string());
    assert!(err.to_string().contains("Connection failed: refused"));

    let err = DatabaseError::FeatureNotEnabled;
    assert!(err.to_string().contains("db"));
}

#[tokio::test]
async fn test_describe_missing_sqlite_table() {
    let tool = DatabaseTool::new(DatabaseConfig::sqlite(":memory:"));
    let result = tool
        .describe_table("test")
        .await
        .expect("describe SQLite table");
    assert!(result.is_empty());
}

#[test]
fn test_register_database_tools_metadata() {
    let registry = ToolRegistry::new();
    register_database_tools(&registry, DatabaseConfig::sqlite(":memory:")).unwrap();

    let meta = registry.get_metadata("db_query").unwrap();
    assert_eq!(meta.server, "built-in");
    assert!(meta.tags.contains(&"database".to_string()));
}

#[test]
fn test_database_backend_serde_roundtrip() {
    let pg_json = serde_json::to_string(&DatabaseBackend::Postgres).unwrap();
    assert_eq!(pg_json, "\"postgres\"");
    let pg: DatabaseBackend = serde_json::from_str(&pg_json).unwrap();
    assert_eq!(pg, DatabaseBackend::Postgres);

    let sq_json = serde_json::to_string(&DatabaseBackend::Sqlite).unwrap();
    assert_eq!(sq_json, "\"sqlite\"");
    let sq: DatabaseBackend = serde_json::from_str(&sq_json).unwrap();
    assert_eq!(sq, DatabaseBackend::Sqlite);
}

#[test]
fn test_query_result_serialization() {
    let result = QueryResult {
        rows: vec![{
            let mut row = std::collections::HashMap::new();
            row.insert("id".to_string(), serde_json::json!(1));
            row.insert("name".to_string(), serde_json::json!("Alice"));
            row
        }],
        rows_affected: 0,
        columns: vec!["id".to_string(), "name".to_string()],
        column_types: vec!["INTEGER".to_string(), "TEXT".to_string()],
        last_insert_id: None,
        truncated: false,
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: QueryResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.rows.len(), 1);
    assert_eq!(deserialized.columns.len(), 2);
    assert_eq!(deserialized.rows_affected, 0);
}

#[test]
fn test_column_info_serialization() {
    let col = ColumnInfo {
        name: "age".to_string(),
        data_type: "integer".to_string(),
        nullable: false,
        primary_key: true,
        default: None,
    };
    let json = serde_json::to_string(&col).unwrap();
    let deserialized: ColumnInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "age");
    assert_eq!(deserialized.data_type, "integer");
    assert!(!deserialized.nullable);
}

#[test]
fn test_database_config_serialization() {
    let cfg = DatabaseConfig::postgres("postgres://localhost/db");
    let json = serde_json::to_string(&cfg).unwrap();
    let deserialized: DatabaseConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.backend, DatabaseBackend::Postgres);
    assert_eq!(deserialized.connection_string, "postgres://localhost/db");
    assert_eq!(deserialized.max_connections, 10);
}

#[test]
fn test_database_error_all_variants_display() {
    let errs = vec![
        (
            DatabaseError::ConnectionFailed("no host".to_string()),
            "Connection failed: no host",
        ),
        (
            DatabaseError::QueryFailed("syntax error".to_string()),
            "Query execution failed: syntax error",
        ),
        (
            DatabaseError::UnsupportedBackend("mysql".to_string()),
            "Unsupported backend: mysql",
        ),
        (
            DatabaseError::InvalidArguments("bad param".to_string()),
            "Invalid arguments: bad param",
        ),
    ];
    for (err, expected) in errs {
        assert!(err.to_string().contains(expected));
    }
}

#[test]
fn test_register_database_tools_schemas_have_required_fields() {
    let registry = ToolRegistry::new();
    register_database_tools(&registry, DatabaseConfig::sqlite(":memory:")).unwrap();

    let exec_tool = registry.get_tool("db_query").unwrap();
    assert_eq!(exec_tool.server, "built-in");
    assert!(exec_tool.description.is_some());

    let list_tool = registry.get_tool("db_list_tables").unwrap();
    assert_eq!(list_tool.server, "built-in");
}
