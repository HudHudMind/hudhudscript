use hudhudscript_tools::database::{
    register_database_tools, ColumnInfo, DatabaseBackend, DatabaseConfig, DatabaseError,
    DatabaseTool, QueryResult,
};
use hudhudscript_tools::registry::ToolRegistry;

#[test]
fn test_database_backend_display() {
    assert_eq!(DatabaseBackend::Postgres.to_string(), "postgres");
    assert_eq!(DatabaseBackend::Sqlite.to_string(), "sqlite");
}

#[test]
fn test_database_config_postgres() {
    let cfg = DatabaseConfig::postgres("postgres://localhost/mydb");
    assert_eq!(cfg.backend, DatabaseBackend::Postgres);
    assert_eq!(cfg.connection_string, "postgres://localhost/mydb");
    assert_eq!(cfg.max_connections, Some(5));
}

#[test]
fn test_database_config_sqlite() {
    let cfg = DatabaseConfig::sqlite("/tmp/test.db");
    assert_eq!(cfg.backend, DatabaseBackend::Sqlite);
    assert_eq!(cfg.connection_string, "/tmp/test.db");
    assert_eq!(cfg.max_connections, Some(1));
}

#[test]
fn test_query_result_affected() {
    let result = QueryResult::affected(42);
    assert_eq!(result.rows_affected, 42);
    assert!(result.rows.is_empty());
    assert!(result.columns.is_empty());
}

#[tokio::test]
async fn test_execute_query_without_db_feature() {
    let tool = DatabaseTool::new(DatabaseConfig::sqlite(":memory:"));
    let result = tool.execute_query("SELECT 1", &[]).await;
    // Without the `db` feature this must return FeatureNotEnabled
    #[cfg(not(feature = "db"))]
    assert!(matches!(result, Err(DatabaseError::FeatureNotEnabled)));
    #[cfg(feature = "db")]
    let _ = result; // may succeed or fail depending on environment
}

#[tokio::test]
async fn test_list_tables_without_db_feature() {
    let tool = DatabaseTool::new(DatabaseConfig::sqlite(":memory:"));
    let result = tool.list_tables().await;
    #[cfg(not(feature = "db"))]
    assert!(matches!(result, Err(DatabaseError::FeatureNotEnabled)));
    #[cfg(feature = "db")]
    let _ = result;
}

#[test]
fn test_register_database_tools() {
    let registry = ToolRegistry::new();
    let count = register_database_tools(&registry).unwrap();
    assert_eq!(count, 2);
    assert!(registry.get_tool("db_execute_query").is_some());
    assert!(registry.get_tool("db_list_tables").is_some());
}

#[test]
fn test_database_tool_backend_accessor() {
    let tool = DatabaseTool::new(DatabaseConfig::postgres("pg://localhost"));
    assert_eq!(*tool.backend(), DatabaseBackend::Postgres);
}

#[test]
fn test_column_info_struct() {
    let col = ColumnInfo {
        name: "email".to_string(),
        data_type: "varchar".to_string(),
        nullable: true,
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
async fn test_describe_table_without_db_feature() {
    let tool = DatabaseTool::new(DatabaseConfig::sqlite(":memory:"));
    let result = tool.describe_table("test").await;
    #[cfg(not(feature = "db"))]
    assert!(matches!(result, Err(DatabaseError::FeatureNotEnabled)));
    #[cfg(feature = "db")]
    let _ = result;
}

#[test]
fn test_register_database_tools_metadata() {
    let registry = ToolRegistry::new();
    register_database_tools(&registry).unwrap();

    let meta = registry.get_metadata("db_execute_query").unwrap();
    assert_eq!(meta.server, "built-in");
    assert!(meta.tags.contains(&"database".to_string()));
    assert!(meta.tags.contains(&"standard".to_string()));
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
    assert_eq!(deserialized.max_connections, Some(5));
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
    register_database_tools(&registry).unwrap();

    let exec_tool = registry.get_tool("db_execute_query").unwrap();
    assert_eq!(exec_tool.server, "built-in");
    assert!(exec_tool.description.is_some());

    let list_tool = registry.get_tool("db_list_tables").unwrap();
    assert_eq!(list_tool.server, "built-in");
}
