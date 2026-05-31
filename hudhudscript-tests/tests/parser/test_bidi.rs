//! External tests for `hudhudscript_parser::bidi` module.

use hudhudscript_parser::{contains_rtl, strip_bidi_controls};

#[test]
fn test_strip_bidi() {
    let source = "متغير\u{200F} x = 42;";
    let cleaned = strip_bidi_controls(source);
    assert_eq!(cleaned, "متغير x = 42;");
}

#[test]
fn test_detect_rtl() {
    assert!(contains_rtl("متغير"));
    assert!(contains_rtl("שלום"));
    assert!(!contains_rtl("hello"));
}

// ── strip_bidi_controls edge cases ─────────────────────────────────

#[test]
fn test_strip_empty_string() {
    assert_eq!(strip_bidi_controls(""), "");
}

#[test]
fn test_strip_no_bidi() {
    let s = "hello world 123";
    assert_eq!(strip_bidi_controls(s), s);
}

#[test]
fn test_strip_lrm() {
    assert_eq!(strip_bidi_controls("a\u{200E}b"), "ab");
}

#[test]
fn test_strip_rlm() {
    assert_eq!(strip_bidi_controls("a\u{200F}b"), "ab");
}

#[test]
fn test_strip_lre() {
    assert_eq!(strip_bidi_controls("x\u{202A}y"), "xy");
}

#[test]
fn test_strip_rle() {
    assert_eq!(strip_bidi_controls("x\u{202B}y"), "xy");
}

#[test]
fn test_strip_pdf() {
    assert_eq!(strip_bidi_controls("x\u{202C}y"), "xy");
}

#[test]
fn test_strip_lro() {
    assert_eq!(strip_bidi_controls("x\u{202D}y"), "xy");
}

#[test]
fn test_strip_rlo() {
    assert_eq!(strip_bidi_controls("x\u{202E}y"), "xy");
}

#[test]
fn test_strip_lri() {
    assert_eq!(strip_bidi_controls("x\u{2066}y"), "xy");
}

#[test]
fn test_strip_rli() {
    assert_eq!(strip_bidi_controls("x\u{2067}y"), "xy");
}

#[test]
fn test_strip_fsi() {
    assert_eq!(strip_bidi_controls("x\u{2068}y"), "xy");
}

#[test]
fn test_strip_pdi() {
    assert_eq!(strip_bidi_controls("x\u{2069}y"), "xy");
}

#[test]
fn test_strip_multiple_bidi_chars() {
    let s = "\u{200E}\u{200F}\u{202A}hello\u{2066}\u{2069}";
    assert_eq!(strip_bidi_controls(s), "hello");
}

#[test]
fn test_strip_preserves_non_bidi_unicode() {
    let s = "hello مرحبا שלום";
    assert_eq!(strip_bidi_controls(s), s);
}

// ── contains_rtl edge cases ────────────────────────────────────────

#[test]
fn test_rtl_empty_string() {
    assert!(!contains_rtl(""));
}

#[test]
fn test_rtl_arabic_extended_a() {
    // Arabic Extended-A range: U+08A0..U+08FF
    assert!(contains_rtl("\u{08A0}"));
}

#[test]
fn test_rtl_arabic_supplement() {
    // Arabic Supplement range: U+0750..U+077F
    assert!(contains_rtl("\u{0750}"));
}

#[test]
fn test_rtl_arabic_presentation_a() {
    // Arabic Presentation Forms-A: U+FB50..U+FDFF
    assert!(contains_rtl("\u{FB50}"));
}

#[test]
fn test_rtl_arabic_presentation_b() {
    // Arabic Presentation Forms-B: U+FE70..U+FEFF
    assert!(contains_rtl("\u{FE70}"));
}

#[test]
fn test_rtl_hebrew_main() {
    // Hebrew range: U+0590..U+05FF
    assert!(contains_rtl("\u{05D0}")); // Alef
}

#[test]
fn test_rtl_hebrew_alphabetic_pf() {
    // Hebrew Alphabetic Presentation Forms: U+FB1D..U+FB4F
    assert!(contains_rtl("\u{FB1D}"));
}

#[test]
fn test_rtl_mixed_ltr_and_rtl() {
    assert!(contains_rtl("hello مرحبا world"));
}

#[test]
fn test_rtl_ascii_only() {
    assert!(!contains_rtl("function foo() { return 42; }"));
}

#[test]
fn test_rtl_japanese_not_rtl() {
    assert!(!contains_rtl("日本語テスト"));
}

#[test]
fn test_rtl_chinese_not_rtl() {
    assert!(!contains_rtl("中文测试"));
}
