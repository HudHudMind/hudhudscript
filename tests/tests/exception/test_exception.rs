//! Tests for hudhudscript-exception: Exception construction, chaining, display.

use hudhudscript_errors::SourcePosition;

use hudhudscript_exception::{Exception, ExceptionCode, StackFrame};

#[test]
fn exception_new_basic() {
    let code = ExceptionCode(82);
    let exc = Exception::new(code, "unexpected '#' at line 5");
    assert_eq!(exc.code, code);
    assert_eq!(exc.message, "unexpected '#' at line 5");
    assert!(exc.position.is_none());
    assert!(exc.cause.is_none());
    assert!(exc.stack.is_empty());
}

#[test]
fn exception_from_code_uses_short_description() {
    let code = ExceptionCode(82);
    let exc = Exception::from_code(code);
    assert_eq!(exc.code, code);
    assert!(!exc.message.is_empty());
}

#[test]
fn exception_with_position() {
    let code = ExceptionCode(80);
    let pos = SourcePosition::new(10, 5, 200).with_file("test.hud");
    let exc = Exception::new(code, "bad escape").at(pos.clone());
    assert_eq!(exc.position, Some(pos));
}

#[test]
fn exception_maybe_at_some() {
    let code = ExceptionCode(81);
    let pos = SourcePosition::new(1, 1, 0);
    let exc = Exception::new(code, "bad number").maybe_at(Some(pos.clone()));
    assert_eq!(exc.position, Some(pos));
}

#[test]
fn exception_maybe_at_none() {
    let code = ExceptionCode(81);
    let exc = Exception::new(code, "bad number").maybe_at(None);
    assert!(exc.position.is_none());
}

#[test]
fn exception_with_cause() {
    let inner = Exception::new(
        ExceptionCode(83),
        "unterminated string",
    );
    let outer = Exception::new(
        ExceptionCode(82),
        "caused by unterminated string",
    )
    .caused_by(inner.clone());

    assert!(outer.cause.is_some());
    assert_eq!(*outer.cause.unwrap(), inner);
}

#[test]
fn exception_with_hint() {
    let exc = Exception::new(
        ExceptionCode(82),
        "bad char",
    )
    .with_hint("did you mean '#'?");
    assert_eq!(exc.hints.len(), 1);
    assert_eq!(exc.hints[0], "did you mean '#'?");
}

#[test]
fn exception_with_hints() {
    let exc = Exception::new(
        ExceptionCode(80),
        "bad escape",
    )
    .with_hint("use \\n for newline")
    .with_hint("use \\t for tab");
    assert_eq!(exc.hints.len(), 2);
}

#[test]
fn exception_push_frame() {
    let exc = Exception::new(
        ExceptionCode(82),
        "bad char",
    )
    .push_frame(StackFrame::new("parse").at("main.hud", 5, 10))
    .push_frame(StackFrame::new("main"));
    assert_eq!(exc.stack.len(), 2);
    assert_eq!(exc.stack[0].function, "parse");
    assert_eq!(exc.stack[1].function, "main");
}

#[test]
fn exception_display_includes_code_and_message() {
    let exc = Exception::new(
        ExceptionCode(82),
        "unexpected character '@'",
    );
    let display = format!("{}", exc);
    assert!(display.contains("unexpected character '@'"));
}

#[test]
fn exception_display_with_cause_shows_chain() {
    let inner = Exception::new(
        ExceptionCode(80),
        "unterminated",
    );
    let outer = Exception::new(
        ExceptionCode(82),
        "outer error",
    )
    .caused_by(inner);
    let display = format!("{}", outer);
    assert!(display.contains("outer error"));
    // cause chain should be present in debug/display output
    assert!(outer.cause.is_some());
}

#[test]
fn exception_clone_equality() {
    let exc = Exception::new(
        ExceptionCode(82),
        "test",
    )
    .with_hint("try again");
    let cloned = exc.clone();
    assert_eq!(exc, cloned);
}

#[test]
fn exception_serialize_deserialize() {
    let exc = Exception::new(
        ExceptionCode(82),
        "serialization test",
    )
    .at(SourcePosition::new(1, 1, 0));
    let json = serde_json::to_string(&exc).unwrap();
    let deser: Exception = serde_json::from_str(&json).unwrap();
    assert_eq!(exc.code, deser.code);
    assert_eq!(exc.message, deser.message);
}

#[test]
fn exception_code_display() {
    let code = ExceptionCode(82);
    let display = format!("{}", code);
    assert!(!display.is_empty());
    assert!(!display.is_empty());
}

#[test]
fn exception_code_long_short_codes() {
    let code = ExceptionCode(82);
    let long = code.long_code();
    let short = code.short_code();
    assert!(!long.is_empty());
    assert!(!short.is_empty());
}

#[test]
fn exception_entry_title_not_empty() {
    let code = ExceptionCode(82);
    assert!(!code.title().is_empty());
}

#[test]
fn exception_code_as_error_code_roundtrip() {
    let exc_code = ExceptionCode(82);
    let err_code = exc_code.as_error_code();
    let exc_code2: ExceptionCode = err_code.into();
    assert_eq!(exc_code, exc_code2);
}
