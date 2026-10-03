//! Coverage tests for hudhudscript-errors `error_value.rs`: Error/ErrorPayload
//! construction, builder methods, catalog accessors, equality/serde contracts,
//! Display, full rendering (English + embedded Turkish), and source-snippet
//! rendering. Exact catalog metadata for E0118/E0120 is pinned from the
//! auto-generated table; Turkish strings are pinned from errors_tr.json.

use hudhudscript_errors::active_embedded_error_catalog;
use hudhudscript_errors::render_with_source;
use hudhudscript_errors::render_with_source_in_locale;
use hudhudscript_errors::{Error, ErrorCategory, ErrorPayload, ErrorCode, SourcePosition};

// E0118 / E0120 from the generated catalog (index = code - 1).
const INVALID_ESCAPE: ErrorCode = ErrorCode(118);
const UNEXPECTED_CHAR: ErrorCode = ErrorCode(120);

// Display/render_full fall back to catalog English only when no HUDHUD_LOCALE
// catalog is active; env-dependent assertions are guarded with this predicate.
fn english_only() -> bool {
    active_embedded_error_catalog().is_none()
}

// ── catalog accessors through an Error ────────────────────────────

#[test]
fn error_entry_accessors_pin_e0120_metadata() {
    let e = Error::from_code(UNEXPECTED_CHAR);
    assert_eq!(e.long_code(), "HHS_E_LEX_UNEXPECTED_CHAR");
    assert_eq!(e.short_code(), "E0120");
    assert_eq!(e.title(), "Unexpected character in source");
    assert_eq!(e.category(), ErrorCategory::Lex);
    assert_eq!(e.category().as_str(), "lex");
    assert_eq!(e.entry().short_code, "E0120");
    assert_eq!(e.since_version(), "0.1.0");
    assert_eq!(e.example_bad(), Some("let name = ‘Alice’;"));
    assert_eq!(e.example_good(), Some("let name = \"Alice\";"));
    assert_eq!(
        e.see_also(),
        ["HHS_E_LEX_INVALID_ESCAPE", "HHS_E_LEX_UNTERMINATED_STRING"]
    );
}

#[test]
fn error_entry_pins_descriptions_and_hints() {
    let e = Error::from_code(UNEXPECTED_CHAR);
    assert_eq!(
        e.short_description(),
        "The lexer encountered a character that does not start any valid token."
    );
    assert!(e.long_description().starts_with("While tokenizing the source file"));
    assert_eq!(e.hints().len(), 3);
    assert_eq!(e.hints()[0], "Check for smart quotes or em-dashes pasted from documents");
    assert_eq!(
        e.hints()[2],
        "Use \" for strings; ' is not a string delimiter in HudHudScript"
    );
    // Second code proves the ErrorCode(u32) -> entry mapping is positional.
    let esc = Error::from_code(INVALID_ESCAPE);
    assert_eq!(esc.short_code(), "E0118");
    assert_eq!(
        esc.short_description(),
        "A backslash in a string literal is followed by a character that is not a recognized escape."
    );
}

#[test]
fn error_code_display_is_short_code_plus_title() {
    assert_eq!(UNEXPECTED_CHAR.to_string(), "[E0120] Unexpected character in source");
    assert_eq!(
        INVALID_ESCAPE.to_string(),
        "[E0118] Invalid escape sequence in string literal"
    );
}

// ── constructors and builders ─────────────────────────────────────

#[test]
fn new_keeps_exact_message_with_no_position_context_or_payload() {
    let e = Error::new(INVALID_ESCAPE, "unsupported escape sequence");
    assert_eq!(e.code, INVALID_ESCAPE);
    assert_eq!(e.message, "unsupported escape sequence");
    assert_eq!(e.position, None);
    assert!(e.context.is_empty());
    assert!(e.payload.is_none());
}

#[test]
fn from_code_seeds_message_from_catalog_short_description() {
    let e = Error::from_code(UNEXPECTED_CHAR);
    assert_eq!(
        e.message,
        "The lexer encountered a character that does not start any valid token."
    );
}

#[test]
fn at_and_maybe_at_position_builders() {
    let p1 = SourcePosition::new(1, 2, 3);
    let p2 = SourcePosition::new(9, 8, 7);
    assert_eq!(Error::new(INVALID_ESCAPE, "m").at(p1.clone()).position, Some(p1.clone()));
    assert_eq!(Error::new(INVALID_ESCAPE, "m").maybe_at(None).position, None);
    assert_eq!(Error::new(INVALID_ESCAPE, "m").maybe_at(Some(p2.clone())).position, Some(p2.clone()));
    // maybe_at(None) must not clear an already attached position.
    let kept = Error::new(INVALID_ESCAPE, "m").at(p1.clone()).maybe_at(None);
    assert_eq!(kept.position, Some(p1.clone()));
    // Overwriting with maybe_at(Some) takes the new position.
    let swapped = Error::new(INVALID_ESCAPE, "m").at(p1).maybe_at(Some(p2.clone()));
    assert_eq!(swapped.position, Some(p2));
}

#[test]
fn with_context_appends_and_context_get_returns_first_match() {
    let e = Error::new(UNEXPECTED_CHAR, "m")
        .with_context("variable", "x")
        .with_context("variable", "y")
        .with_context("limit", "10");
    assert_eq!(e.context.len(), 3);
    assert_eq!(e.context_get("variable"), Some("x"));
    assert_eq!(e.context_get("limit"), Some("10"));
    assert_eq!(e.context_get("missing"), None);
}

#[test]
fn payload_downcast_and_as_arc() {
    let plain = Error::new(UNEXPECTED_CHAR, "custom failure");
    let err = plain.clone().with_payload(9001u64);
    // Payload does not disturb the message or other fields.
    assert_eq!(err.message, "custom failure");
    assert_eq!(err.payload_ref::<u64>(), Some(&9001));
    assert_eq!(err.payload_ref::<i32>(), None);
    assert!(plain.payload.is_none());

    let p = ErrorPayload::new(String::from("boom"));
    let arc = p.as_arc().expect("arc present");
    assert_eq!(arc.downcast_ref::<String>(), Some(&"boom".to_string()));
    assert!(ErrorPayload::empty().as_arc().is_none());
}

#[test]
fn payload_debug_rendering_and_defaults() {
    assert_eq!(format!("{:?}", ErrorPayload::empty()), "ErrorPayload(None)");
    assert_eq!(format!("{:?}", ErrorPayload::new(1i32)), "ErrorPayload(<opaque>)");
    assert!(ErrorPayload::default().is_none());
    assert!(!ErrorPayload::new(0u8).is_none());
}

#[test]
fn payload_is_excluded_from_equality() {
    let a = Error::new(UNEXPECTED_CHAR, "m").with_payload(1u8);
    let b = Error::new(UNEXPECTED_CHAR, "m").with_payload(2u64);
    assert_eq!(a, b, "payload must not participate in Error equality");
    assert_eq!(ErrorPayload::new(1u32), ErrorPayload::new(2u32));
    let c = Error::new(UNEXPECTED_CHAR, "different message");
    assert_ne!(a, c);
    let d = Error::new(INVALID_ESCAPE, "m");
    assert_ne!(a, d, "different codes must not compare equal");
}

#[test]
fn payload_serde_contract_is_null_and_default() {
    assert_eq!(serde_json::to_string(&ErrorPayload::empty()).unwrap(), "null");
    assert_eq!(serde_json::to_string(&ErrorPayload::new(7u16)).unwrap(), "null");
    // The Deserialize impl returns an empty payload without consuming any
    // input, so serde_json then reports every byte of a non-empty document
    // as trailing characters — deserializing "123" is an error, not a value.
    let revived: Result<ErrorPayload, serde_json::Error> = serde_json::from_str("123");
    let err = revived.expect_err("non-empty input is never consumed");
    assert!(
        err.to_string().contains("trailing characters"),
        "got: {}",
        err
    );
}

#[test]
fn error_json_roundtrip_preserves_code_message_position_context() {
    let minimal = Error::from_code(UNEXPECTED_CHAR);
    assert_eq!(
        serde_json::to_string(&minimal).unwrap(),
        "{\"code\":120,\"message\":\"The lexer encountered a character that does not start any valid token.\",\"position\":null,\"context\":[]}"
    );
    let full = minimal
        .clone()
        .at(SourcePosition::new(2, 5, 14).with_file("main.hud"))
        .with_context("variable", "x")
        .with_payload(42u32);
    let revived: Error = serde_json::from_str(&serde_json::to_string(&full).unwrap()).unwrap();
    assert_eq!(revived, full);
    assert_eq!(revived.message, full.message);
    assert_eq!(revived.position, full.position);
    assert_eq!(revived.context, full.context);
    assert!(revived.payload.is_none(), "payload is skipped by serde");
}

#[test]
fn error_satisfies_std_error_trait() {
    if !english_only() {
        return;
    }
    let err = Error::new(INVALID_ESCAPE, "boom");
    let boxed: Box<dyn std::error::Error> = Box::new(err.clone());
    assert_eq!(boxed.to_string(), err.to_string());
}

// ── Display ───────────────────────────────────────────────────────

#[test]
fn display_renders_short_code_title_and_message() {
    if !english_only() {
        return;
    }
    let e = Error::from_code(UNEXPECTED_CHAR);
    assert_eq!(
        e.to_string(),
        "[E0120] Unexpected character in source: The lexer encountered a character that does not start any valid token."
    );
}

#[test]
fn display_appends_position_and_keeps_custom_message() {
    if !english_only() {
        return;
    }
    let e = Error::new(INVALID_ESCAPE, "unsupported escape '\\q'")
        .at(SourcePosition::new(4, 9, 40));
    assert_eq!(
        e.to_string(),
        "[E0118] Invalid escape sequence in string literal: unsupported escape '\\q' at 4:9"
    );
}

#[test]
fn display_with_file_position_shows_path() {
    if !english_only() {
        return;
    }
    let e = Error::new(UNEXPECTED_CHAR, "stray '@'")
        .at(SourcePosition::new(2, 5, 14).with_file("src/main.hud"));
    assert_eq!(
        e.to_string(),
        "[E0120] Unexpected character in source: stray '@' at src/main.hud:2:5"
    );
}

// ── full rendering (explicit locales keep these deterministic) ────

#[test]
fn render_full_english_layout_for_from_code() {
    let out = Error::from_code(UNEXPECTED_CHAR).render_full_in_locale("en");
    assert_eq!(out.lines().next(), Some("[E0120] Unexpected character in source"));
    // Canonical short-description line appears exactly once (body suppressed).
    assert_eq!(
        out.matches("\n  The lexer encountered a character that does not start any valid token.\n").count(),
        1
    );
    assert!(out.contains("\n  hints:\n"));
    assert!(out.contains("    • Check for smart quotes or em-dashes pasted from documents\n"));
    assert!(out.contains("\n  example (incorrect):\n    let name = ‘Alice’;\n"));
    assert!(out.contains("\n  example (corrected):\n    let name = \"Alice\";\n"));
    assert!(out.contains("\n  see also: HHS_E_LEX_INVALID_ESCAPE HHS_E_LEX_UNTERMINATED_STRING\n"));
    assert_eq!(
        out.lines().last(),
        Some("  long code: HHS_E_LEX_UNEXPECTED_CHAR  |  category: lex  |  since: 0.1.0")
    );
}

#[test]
fn render_full_prints_custom_body_message() {
    let e = Error::new(UNEXPECTED_CHAR, "stray '@' after identifier");
    let out = e.render_full_in_locale("en");
    assert!(out.contains("\n  stray '@' after identifier\n"));
    assert_eq!(
        out.matches("\n  The lexer encountered a character that does not start any valid token.\n").count(),
        1
    );
    assert_eq!(out.lines().next(), Some("[E0120] Unexpected character in source"));
}

#[test]
fn render_full_strips_english_code_title_prefix_from_message() {
    let e = Error::new(
        UNEXPECTED_CHAR,
        "[E0120] Unexpected character in source — near '@'",
    );
    let out = e.render_full_in_locale("en");
    assert!(out.contains("\n  near '@'\n"));
    assert!(!out.contains("— near '@'"), "prefix must be stripped from body");
}

#[test]
fn render_full_renders_position_lines() {
    let plain = Error::from_code(UNEXPECTED_CHAR).at(SourcePosition::new(3, 7, 20));
    assert!(plain.render_full_in_locale("en").contains("\n  at 3:7\n"));
    let filed = Error::from_code(UNEXPECTED_CHAR)
        .at(SourcePosition::new(3, 7, 20).with_file("main.hud"));
    assert!(filed.render_full_in_locale("en").contains("\n  at main.hud:3:7\n"));
}

#[test]
fn render_full_turkish_uses_embedded_translation() {
    let out = Error::from_code(UNEXPECTED_CHAR).render_full_in_locale("tr");
    assert_eq!(out.lines().next(), Some("[E0120] Kaynakta beklenmeyen karakter"));
    assert!(out.contains("\n  Lexer beklenmeyen bir karakterle karşılaştı.\n"));
    assert!(out.contains("\n  Kaynak dosyada tanınmayan sembol.\n"));
    assert!(out.contains("    • Karakter kodlamasını kontrol edin\n"));
    assert!(out.contains("    • Gizli karakterleri arayın\n"));
    assert_eq!(out.matches("    • ").count(), 2, "tr catalog has 2 hints");
    assert!(!out.contains("Unexpected character in source"), "English title must be replaced");
    // Catalog-only fields stay English.
    assert!(out.contains("\n  example (incorrect):\n    let name = ‘Alice’;\n"));
    assert_eq!(
        out.lines().last(),
        Some("  long code: HHS_E_LEX_UNEXPECTED_CHAR  |  category: lex  |  since: 0.1.0")
    );
}

#[test]
fn render_full_strips_localized_prefix_from_message() {
    let e = Error::new(
        UNEXPECTED_CHAR,
        "[E0120] Kaynakta beklenmeyen karakter — fazladan bilgi",
    );
    let out = e.render_full_in_locale("tr");
    assert!(out.contains("\n  fazladan bilgi\n"));
    assert!(!out.contains("— fazladan bilgi"));
}

#[test]
fn localized_falls_back_to_english_for_unknown_locale() {
    let e = Error::from_code(UNEXPECTED_CHAR);
    for locale in ["en", "zz", "en-US"] {
        let loc = e.localized(locale);
        assert_eq!(loc.title, "Unexpected character in source", "locale {locale}");
        assert_eq!(
            loc.short_description,
            "The lexer encountered a character that does not start any valid token."
        );
        assert_eq!(loc.hints.len(), 3);
        assert_eq!(loc.hints[1], "Some editors insert zero-width or BOM characters — re-save as plain UTF-8");
    }
}

// ── source snippet rendering ──────────────────────────────────────

#[test]
fn render_with_source_aligns_caret_under_column() {
    let src = "let a = 1;\nlet b = '@';\n";
    let e = Error::from_code(UNEXPECTED_CHAR).at(SourcePosition::new(2, 5, 11));
    let out = render_with_source_in_locale(&e, src, "en");
    assert!(out.ends_with("\n  |\n  2| let b = '@';\n  |     ^\n"));
}

#[test]
fn render_with_source_omits_caret_for_zero_column() {
    let src = "let a = 1;\n";
    let e = Error::from_code(UNEXPECTED_CHAR).at(SourcePosition::new(1, 0, 0));
    let out = render_with_source_in_locale(&e, src, "en");
    assert!(out.ends_with("\n  |\n  1| let a = 1;\n  |\n"));
}

#[test]
fn render_with_source_out_of_range_line_is_noop() {
    let e = Error::from_code(UNEXPECTED_CHAR).at(SourcePosition::new(99, 1, 0));
    let out = render_with_source_in_locale(&e, "let a = 1;\n", "en");
    assert_eq!(out, e.render_full_in_locale("en"));
    let zero = Error::from_code(UNEXPECTED_CHAR).at(SourcePosition::new(0, 0, 0));
    assert_eq!(
        render_with_source_in_locale(&zero, "let a = 1;\n", "en"),
        zero.render_full_in_locale("en")
    );
}

#[test]
fn render_with_source_default_variant_matches_layout() {
    if !english_only() {
        return;
    }
    let src = "let a = 1;\nlet b = '@';\n";
    let e = Error::from_code(UNEXPECTED_CHAR).at(SourcePosition::new(2, 5, 11));
    let out = render_with_source(&e, src);
    assert!(out.ends_with("\n  |\n  2| let b = '@';\n  |     ^\n"));
}

// ── SourcePosition display used by error rendering ────────────────

#[test]
fn source_position_display_with_and_without_file() {
    assert_eq!(SourcePosition::new(3, 7, 20).to_string(), "3:7");
    assert_eq!(
        SourcePosition::new(3, 7, 20).with_file("src/main.hud").to_string(),
        "src/main.hud:3:7"
    );
}
