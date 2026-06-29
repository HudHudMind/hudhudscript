//! Tests for hudhud-print sprintf formatter (moved from crate-inline tests).

use hudhud_print::format::{sprintf, FmtArg};

#[test]
fn test_sprintf_string() {
    let result = sprintf("hello %s", &[FmtArg::Str("world".into())]).unwrap();
    assert_eq!(result, "hello world");
}

#[test]
fn test_sprintf_int() {
    let result = sprintf("count: %d", &[FmtArg::Int(42)]).unwrap();
    assert_eq!(result, "count: 42");
}

#[test]
fn test_sprintf_float() {
    let result = sprintf("pi = %f", &[FmtArg::Float(3.14)]).unwrap();
    assert!(result.starts_with("pi = 3.14"));
}

#[test]
fn test_sprintf_percent() {
    let result = sprintf("100%% done", &[]).unwrap();
    assert_eq!(result, "100% done");
}

#[test]
fn test_sprintf_missing_arg() {
    let result = sprintf("%s %s", &[FmtArg::Str("a".into())]);
    assert!(result.is_err());
}

#[test]
fn test_sprintf_wrong_type() {
    let result = sprintf("%d", &[FmtArg::Str("not int".into())]);
    assert!(result.is_err());
}
