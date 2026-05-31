//! Tests for the unified Error type from hudhudscript-errors.
//! Covers: Error construction, ErrorCode catalog lookup, SourcePosition,
//! context fields, Display formatting, and Severity/Diagnostic conversion.

use hudhudscript_errors::{Error, ErrorCode, ErrorPayload, Severity, SourcePosition};

#[test]
fn test_basic_error_construction() {
    let e = Error {
        code: ErrorCode::RuntimeTypeError,
        message: "expected number".to_string(),
        position: None,
        context: vec![],
        payload: ErrorPayload::empty(),
    };
    let entry = e.code.entry();
    assert_eq!(entry.short_code, "E0250");
    assert!(entry.title.to_lowercase().contains("type"));
    assert_eq!(e.message, "expected number");
    assert!(e.position.is_none());
}

#[test]
fn test_error_with_context() {
    let e = Error {
        code: ErrorCode::RuntimeUndefinedVariable,
        message: "Undefined variable: x".to_string(),
        position: None,
        context: vec![("name".to_string(), "x".to_string())],
        payload: ErrorPayload::empty(),
    };
    assert_eq!(e.context.len(), 1);
    assert_eq!(e.context[0], ("name".to_string(), "x".to_string()));
}

#[test]
fn test_error_with_position() {
    let pos = SourcePosition::new(10, 5, 200).with_file("test.hud");
    let e = Error {
        code: ErrorCode::RuntimeUndefinedVariable,
        message: "undefined variable 'x'".to_string(),
        position: Some(pos.clone()),
        context: vec![],
        payload: ErrorPayload::empty(),
    };
    assert_eq!(e.position, Some(pos));
    let entry = e.code.entry();
    assert_eq!(entry.short_code, "E0251");
}

#[test]
fn test_source_position_with_file() {
    let pos = SourcePosition::new(5, 3, 100).with_file("main.hud");
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 3);
    assert_eq!(pos.offset, 100);
    assert_eq!(pos.file_path, Some("main.hud".to_string()));
}

#[test]
fn test_source_position_without_file() {
    let pos = SourcePosition::new(1, 1, 0);
    assert!(pos.file_path.is_none());
}

#[test]
fn test_error_code_catalog_lookup() {
    let entry = ErrorCode::RuntimeCallError.entry();
    assert!(!entry.short_code.is_empty());
    assert!(!entry.title.is_empty());
    assert!(!entry.short_description.is_empty());
}

#[test]
fn test_error_display_contains_message() {
    let e = Error {
        code: ErrorCode::RuntimeTypeError,
        message: "expected number, got string".to_string(),
        position: None,
        context: vec![],
        payload: ErrorPayload::empty(),
    };
    let display = format!("{}", e);
    assert!(
        display.contains("expected number"),
        "Display should contain message: {}",
        display
    );
}

#[test]
fn test_error_payload_empty() {
    let payload = ErrorPayload::empty();
    assert!(payload.is_none());
    assert!(payload.downcast_ref::<String>().is_none());
}

#[test]
fn test_error_payload_with_value() {
    let payload = ErrorPayload::new(42i32);
    assert!(!payload.is_none());
    assert_eq!(payload.downcast_ref::<i32>(), Some(&42));
    assert!(payload.downcast_ref::<String>().is_none());
}

#[test]
fn test_error_equality_ignores_payload() {
    let e1 = Error {
        code: ErrorCode::RuntimeTypeError,
        message: "msg".to_string(),
        position: None,
        context: vec![],
        payload: ErrorPayload::new(1i32),
    };
    let e2 = Error {
        code: ErrorCode::RuntimeTypeError,
        message: "msg".to_string(),
        position: None,
        context: vec![],
        payload: ErrorPayload::new(2i32),
    };
    assert_eq!(
        e1, e2,
        "Errors with same code/message should be equal regardless of payload"
    );
}

#[test]
fn test_severity_variants_exist() {
    // Just verify the Severity enum is usable
    let _e = Severity::Error;
    let _w = Severity::Warning;
    let _i = Severity::Info;
}
