//! Tests for hudhud-fs shared file builtin dispatch — type/resource error paths
//! (moved from src/file_ops.rs).

use hudhud_fs::file_ops::dispatch;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::ErrorCode;

fn temp_path(name: &str) -> String {
    std::env::temp_dir()
        .join(format!("hudhud-g10-{}-{}", name, std::process::id()))
        .to_string_lossy()
        .into_owned()
}

#[test]
fn file_read_null_is_runtime_type_error() {
    let error = dispatch("read", &[Value16::null()]).unwrap_err();
    assert_eq!(error.code, ErrorCode::RuntimeTypeError);
}

#[test]
fn file_read_number_is_runtime_type_error() {
    let error = dispatch("read", &[Value16::number(12.0)]).unwrap_err();
    assert_eq!(error.code, ErrorCode::RuntimeTypeError);
}

#[test]
fn file_read_missing_is_runtime_resource_error() {
    let error = dispatch("read", &[Value16::string(temp_path("missing"))]).unwrap_err();
    assert_eq!(error.code, ErrorCode::RuntimeResourceError);
}

#[test]
fn file_read_valid_fixture_is_unchanged() {
    let path = temp_path("fixture");
    std::fs::write(&path, "g10-content").unwrap();
    let value = dispatch("read", &[Value16::string(path)]).unwrap();
    assert_eq!(value.as_string(), Some("g10-content".to_string()));
}

#[test]
fn file_write_non_string_content_is_runtime_type_error() {
    let error = dispatch(
        "write",
        &[Value16::string(temp_path("typed")), Value16::number(12.0)],
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::RuntimeTypeError);
}
