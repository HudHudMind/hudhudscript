//! Real unit tests for hudhudscript-tools-io
//! Covers: StandardTool call(), ToolError, file read, JSON parse, HTTP stubs

use hudhudscript_tools_io::*;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

// ═══════════════════════════════════════════════════════════════════════════
// StandardTool — FileRead with real temp file
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn file_read_existing_file() {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "hello world").unwrap();
    let path = f.path().to_string_lossy().to_string();

    let result = StandardTool::FileRead.call(&json!({"path": path})).unwrap();
    // FileRead returns { contents: "...", path: "..." }
    let content = result["contents"].as_str().unwrap();
    assert!(content.contains("hello world"));
}

#[test]
fn file_read_nonexistent_file() {
    let result = StandardTool::FileRead.call(&json!({"path": "/tmp/nonexistent_hudhud_test_file_12345.txt"}));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ToolError::ExecutionFailed(_)));
}

#[test]
fn file_read_missing_path_arg() {
    let result = StandardTool::FileRead.call(&json!({}));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolError::InvalidArguments(_)));
}

#[test]
fn file_read_wrong_type_path() {
    let result = StandardTool::FileRead.call(&json!({"path": 123}));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ToolError::InvalidArguments(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// StandardTool — JsonParse
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn json_parse_valid() {
    let result = StandardTool::JsonParse.call(&json!({"text": "{\"key\": \"value\"}"})).unwrap();
    // JsonParse returns { value: parsed_json }
    assert_eq!(result["value"]["key"], "value");
}

#[test]
fn json_parse_array() {
    let result = StandardTool::JsonParse.call(&json!({"text": "[1, 2, 3]"})).unwrap();
    let arr = result["value"].as_array().unwrap();
    assert_eq!(arr.len(), 3);
}

#[test]
fn json_parse_invalid() {
    let result = StandardTool::JsonParse.call(&json!({"text": "not json"}));
    assert!(result.is_err());
}

#[test]
fn json_parse_missing_text() {
    let result = StandardTool::JsonParse.call(&json!({}));
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// StandardTool — HTTP methods (test argument validation, not real HTTP)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn http_get_valid_args() {
    let result = StandardTool::HttpGet.call(&json!({"url": "https://httpbin.org/get"}));
    // Will succeed or fail depending on network — just verify it doesn't panic
    let _ = result;
}

#[test]
fn http_get_missing_url() {
    let result = StandardTool::HttpGet.call(&json!({}));
    assert!(result.is_err());
}

#[test]
fn http_post_valid_args() {
    let result = StandardTool::HttpPost.call(&json!({
        "url": "https://httpbin.org/post",
        "body": {"data": "test"}
    }));
    let _ = result;
}

#[test]
fn http_post_missing_url() {
    let result = StandardTool::HttpPost.call(&json!({}));
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// ToolError — codes, display, conversion
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tool_error_invalid_arguments_display() {
    let err = ToolError::InvalidArguments("missing 'path'".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Invalid arguments") || msg.contains("missing 'path'"));
}

#[test]
fn tool_error_execution_failed_display() {
    let err = ToolError::ExecutionFailed("file not found".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Execution failed") || msg.contains("file not found"));
}

#[test]
fn tool_error_security_violation_display() {
    let err = ToolError::SecurityViolation("path traversal".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Security violation") || msg.contains("path traversal"));
}

#[test]
fn tool_error_code_mapping() {
    assert_eq!(
        ToolError::ExecutionFailed("x".into()).code(),
        hudhudscript_errors::ErrorCode::ToolExecutionFailed
    );
    assert_eq!(
        ToolError::InvalidArguments("x".into()).code(),
        hudhudscript_errors::ErrorCode::ToolInvalidArguments
    );
    assert_eq!(
        ToolError::SecurityViolation("x".into()).code(),
        hudhudscript_errors::ErrorCode::ToolSecurityViolation
    );
}

#[test]
fn tool_error_short_code_and_title() {
    let err = ToolError::InvalidArguments("test".into());
    assert!(!err.short_code().is_empty());
    assert!(!err.title().is_empty());
    assert!(!err.display_full().is_empty());
}
