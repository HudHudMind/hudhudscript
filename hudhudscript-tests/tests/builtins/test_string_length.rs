//! `.length` intrinsic on strings and arrays — VM parity coverage
//! (ported from the deleted `tests/integration/string_length_test.rs`
//!  sibling commit 7598d0b^, rewired onto the VM-backed test harness).
//!
//! The VM's GetProperty op now handles `Value::String`, `Value::Array`,
//! `Value::Set`, and `Value::Map` for `.length` matching the old
//! interpreter semantics.  String length is Unicode char count, not byte
//! count.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_parser::parse;

fn eval(source: &str) -> Value {
    let ast = parse(source).unwrap_or_else(|e| panic!("Parse error:\n{e}"));
    let mut interp = Interpreter::new();
    interp
        .execute(&ast)
        .unwrap_or_else(|e| panic!("Runtime error:\n{e:?}"));
    interp.get_variable("r").unwrap()
}

// ── String ────────────────────────────────────────────────────────────

#[test]
fn string_length_ascii() {
    assert_eq!(eval(r#"let r = "hello".length;"#), Value16::number(5.0));
}

#[test]
fn string_length_unicode_emoji() {
    // "merhaba 🎉" = 9 chars (1 emoji char, not 4 bytes).
    assert_eq!(eval(r#"let r = "merhaba 🎉".length;"#), Value16::number(9.0));
}

#[test]
fn string_length_cjk() {
    // "你好世界" = 4 characters.
    assert_eq!(eval(r#"let r = "你好世界".length;"#), Value16::number(4.0));
}

#[test]
fn string_length_turkish() {
    assert_eq!(eval(r#"let r = "İstanbul".length;"#), Value16::number(8.0));
}

#[test]
fn string_length_arabic() {
    assert_eq!(eval(r#"let r = "مرحبا".length;"#), Value16::number(5.0));
}

#[test]
fn string_length_empty() {
    assert_eq!(eval(r#"let r = "".length;"#), Value16::number(0.0));
}

#[test]
fn string_length_mixed_unicode() {
    // "a🎉b🎉c" = 5 characters.
    assert_eq!(eval(r#"let r = "a🎉b🎉c".length;"#), Value16::number(5.0));
}

#[test]
fn string_length_via_variable() {
    // Same value even when the string is bound first.
    assert_eq!(
        eval(r#"let s = "merhaba 🎉"; let r = s.length;"#),
        Value16::number(9.0)
    );
}

// ── Array ─────────────────────────────────────────────────────────────

#[test]
fn array_length_populated() {
    assert_eq!(eval("let r = [1, 2, 3, 4, 5].length;"), Value16::number(5.0));
}

#[test]
fn array_length_empty() {
    assert_eq!(eval("let r = [].length;"), Value16::number(0.0));
}

#[test]
fn array_length_nested() {
    // Top-level array holds 3 nested arrays.
    assert_eq!(
        eval("let r = [[1,2], [3,4], [5,6]].length;"),
        Value16::number(3.0)
    );
}
