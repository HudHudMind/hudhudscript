//! Tests for ExceptionCode catalog lookup (category, hints).

use hudhudscript_exception::{ExceptionCategory, ExceptionCode};

// Valid ExceptionCode indices from the catalog table ranges:
// Lex: 80-83, Parse: 159-164, Runtime: 179-212

#[test]
fn lex_code_has_correct_category() {
    let code = ExceptionCode(80); // first lex entry
    assert_eq!(code.category(), ExceptionCategory::Lex);
    assert!(!code.title().is_empty());
}

#[test]
fn parse_code_has_correct_category() {
    let code = ExceptionCode(159); // first parse entry
    assert_eq!(code.category(), ExceptionCategory::Parse);
    assert!(!code.title().is_empty());
}

#[test]
fn runtime_code_has_correct_category() {
    let code = ExceptionCode(179); // first runtime entry
    assert_eq!(code.category(), ExceptionCategory::Runtime);
    assert!(!code.title().is_empty());
}

#[test]
fn catalog_has_hints() {
    let code = ExceptionCode(82); // unexpected char lex entry
    let hints = code.hints();
    assert!(!hints.is_empty(), "lex errors should have hints");
}

#[test]
fn catalog_short_description_not_empty() {
    let code = ExceptionCode(80); // invalid escape
    let desc = code.short_description();
    assert!(!desc.is_empty());
}

#[test]
fn catalog_long_description_not_empty() {
    let code = ExceptionCode(82); // unexpected char lex entry
    let desc = code.long_description();
    assert!(!desc.is_empty());
}

#[test]
fn exception_category_display() {
    assert_eq!(format!("{}", ExceptionCategory::Lex), "lex");
    assert_eq!(format!("{}", ExceptionCategory::Parse), "parse");
    assert_eq!(format!("{}", ExceptionCategory::Runtime), "runtime");
}
