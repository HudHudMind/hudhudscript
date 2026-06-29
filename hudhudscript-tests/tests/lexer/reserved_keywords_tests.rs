//! Reserved keyword canonical table tests.
//!
//! Moved from `hudhudscript-lexer/src/keyword_normalizer/reserved_keywords.rs`
//! inline test module as part of I2-A2.

use hudhudscript_lexer::is_reserved_keyword;

#[test]
fn canonical_english_reserved() {
    assert!(is_reserved_keyword("let"));
    assert!(is_reserved_keyword("function"));
    assert!(is_reserved_keyword("promise"));
}

#[test]
fn turkish_reserved() {
    assert!(is_reserved_keyword("değişken"));
    assert!(is_reserved_keyword("ajan"));
}

#[test]
fn operator_reserved() {
    assert!(is_reserved_keyword("ve"));
    assert!(is_reserved_keyword("veya"));
}

#[test]
fn grammar_declared_reserved() {
    assert!(is_reserved_keyword("action"));
    assert!(is_reserved_keyword("subject"));
    assert!(is_reserved_keyword("memory"));
    assert!(is_reserved_keyword("store"));
    assert!(is_reserved_keyword("rule"));
    assert!(is_reserved_keyword("protocol"));
    assert!(is_reserved_keyword("swarm"));
    assert!(is_reserved_keyword("when"));
    assert!(is_reserved_keyword("on"));
    assert!(is_reserved_keyword("trigger"));
    assert!(is_reserved_keyword("parallel"));
    assert!(is_reserved_keyword("sequential"));
}

#[test]
fn builtin_names_remain_non_reserved() {
    // These are explicit non-reserved builtins handled elsewhere.
    assert!(!is_reserved_keyword("print"));
    assert!(!is_reserved_keyword("execute"));
    assert!(!is_reserved_keyword("values"));
    assert!(!is_reserved_keyword("allow"));
    assert!(!is_reserved_keyword("deny"));
    assert!(!is_reserved_keyword("merge"));
}

#[test]
fn non_reserved_identifier() {
    assert!(!is_reserved_keyword("foo"));
    assert!(!is_reserved_keyword("bar_1"));
}
