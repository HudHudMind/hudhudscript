//! Real unit tests for hudhudscript-tools-io — error types, data types

use hudhudscript_tools_io::*;

#[test]
fn tool_error_display_all_variants() {
    let errors: Vec<ToolError> = vec![
        ToolError::InvalidArguments("bad arg".into()),
        ToolError::ExecutionFailed("oops".into()),
        ToolError::SecurityViolation("blocked".into()),
    ];
    for e in &errors {
        assert!(!format!("{}", e).is_empty());
    }
}

#[test]
fn tool_error_code_is_valid() {
    let e = ToolError::ExecutionFailed("x".into());
    assert!(!e.code().short_code().is_empty());
}

#[test]
fn database_error_display_all_variants() {
    let errors: Vec<DatabaseError> = vec![
        DatabaseError::ConnectionFailed("timeout".into()),
        DatabaseError::QueryFailed("syntax error".into()),
        DatabaseError::UnsupportedBackend("oracle".into()),
        DatabaseError::FeatureNotEnabled,
        DatabaseError::InvalidArguments("bad".into()),
    ];
    for e in &errors {
        assert!(!format!("{}", e).is_empty());
    }
}

#[test]
fn database_error_code_is_valid() {
    let e = DatabaseError::ConnectionFailed("x".into());
    assert!(!e.code().short_code().is_empty());
}

#[test]
fn database_backend_variants() {
    assert!(matches!(DatabaseBackend::Sqlite, DatabaseBackend::Sqlite));
    assert!(matches!(
        DatabaseBackend::Postgres,
        DatabaseBackend::Postgres
    ));
}

#[test]
fn query_result_empty() {
    let result = QueryResult {
        columns: vec![],
        rows: vec![],
        rows_affected: 0,
    };
    assert!(result.columns.is_empty());
    assert!(result.rows.is_empty());
    assert_eq!(result.rows_affected, 0);
}
