//! Tests for StackFrame construction and display.

use hudhudscript_exception::StackFrame;

#[test]
fn frame_new_basic() {
    let frame = StackFrame::new("my_function");
    assert_eq!(frame.function, "my_function");
    assert!(frame.file.is_none());
    assert!(frame.line.is_none());
    assert!(frame.column.is_none());
}

#[test]
fn frame_with_location() {
    let frame = StackFrame::new("parse").at("main.hud", 10, 5);
    assert_eq!(frame.function, "parse");
    assert_eq!(frame.file, Some("main.hud".to_string()));
    assert_eq!(frame.line, Some(10));
    assert_eq!(frame.column, Some(5));
}

#[test]
fn frame_display_basic() {
    let frame = StackFrame::new("compute");
    let display = format!("{}", frame);
    assert_eq!(display, "    at compute");
}

#[test]
fn frame_display_with_file() {
    let frame = StackFrame::new("run").at("script.hud", 3, 0);
    let display = format!("{}", frame);
    assert!(display.contains("script.hud"));
    assert!(display.contains(":3"));
}

#[test]
fn frame_display_with_file_line_column() {
    let frame = StackFrame::new("eval").at("test.hud", 42, 7);
    let display = format!("{}", frame);
    assert!(display.contains("test.hud:42:7"));
}

#[test]
fn frame_clone_eq() {
    let frame = StackFrame::new("f").at("a.hud", 1, 1);
    let cloned = frame.clone();
    assert_eq!(frame, cloned);
}

#[test]
fn frame_serialize_roundtrip() {
    let frame = StackFrame::new("serialize_me").at("test.hud", 5, 3);
    let json = serde_json::to_string(&frame).unwrap();
    let deser: StackFrame = serde_json::from_str(&json).unwrap();
    assert_eq!(frame, deser);
}

#[test]
fn frame_anonymous() {
    let frame = StackFrame::new("<anonymous>");
    assert_eq!(frame.function, "<anonymous>");
}

#[test]
fn frame_global() {
    let frame = StackFrame::new("<global>");
    let display = format!("{}", frame);
    assert!(display.contains("<global>"));
}
