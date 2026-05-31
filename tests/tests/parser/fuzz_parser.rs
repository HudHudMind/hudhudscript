//! Fuzz / property-based tests for hudhudscript-parser (Issue #973)
//!
//! These tests use proptest to generate random and semi-structured inputs,
//! feeding them into the parser to verify it never panics.
//! The parser should either return Ok(...) or Err(...) — never crash.

use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategy helpers
// ---------------------------------------------------------------------------

/// Completely random byte strings interpreted as UTF-8 (lossy).
fn arbitrary_utf8() -> impl Strategy<Value = String> {
    prop::collection::vec(any::<u8>(), 0..512)
        .prop_map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

/// Random strings from a reduced ASCII alphabet that includes common
/// programming-language punctuation so we get higher coverage of lexer paths.
fn code_like_ascii() -> impl Strategy<Value = String> {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             0123456789+-*/=<>!&|^~%@#$?.,:;(){}[]\\'\"\n\r\t "
        .chars()
        .collect();
    prop::collection::vec(prop::sample::select(chars), 0..256)
        .prop_map(|cs| cs.into_iter().collect::<String>())
}

/// Strings built from actual HudHudScript keywords mixed with random tokens.
fn keyword_soup() -> impl Strategy<Value = String> {
    let keywords = vec![
        "let",
        "var",
        "const",
        "if",
        "else",
        "while",
        "for",
        "fn",
        "return",
        "break",
        "continue",
        "class",
        "new",
        "true",
        "false",
        "null",
        "async",
        "await",
        "try",
        "catch",
        "throw",
        "match",
        "import",
        "export",
        "enum",
        "switch",
        "case",
        "default",
        // Turkish keywords
        "eger",
        "yoksa",
        "dongu",
        "fonksiyon",
        "degisken",
        // Some punctuation / structural tokens
        "{",
        "}",
        "(",
        ")",
        "[",
        "]",
        ";",
        "=",
        "==",
        "!=",
        "=>",
        "->",
        "::",
        ".",
        ",",
        ":",
        "+",
        "-",
        "*",
        "/",
        "\"hello\"",
        "42",
        "3.14",
        "// comment\n",
        "/* block */",
    ];

    prop::collection::vec(prop::sample::select(keywords), 0..30).prop_map(|tokens| tokens.join(" "))
}

/// Strings with Unicode from various scripts (Arabic, Japanese, CJK, etc.)
fn multilingual_input() -> impl Strategy<Value = String> {
    let fragments = vec![
        "متغير",
        "اگر",
        "変数",
        "関数",
        "변수",
        "함수",
        "переменная",
        "если",
        "μεταβλητή",
        "ตัวแปร",
        "let",
        "x",
        "=",
        "42",
        ";",
        "{",
        "}",
        "(",
        ")",
        "\n",
        " ",
        "\t",
        "//",
        "/*",
        "*/",
        "\"مرحبا\"",
        "\"こんにちは\"",
        "\"안녕하세요\"",
    ];

    prop::collection::vec(prop::sample::select(fragments), 0..40)
        .prop_map(|tokens| tokens.join(" "))
}

/// Valid-ish program fragments that should parse successfully or at least
/// exercise deeper parser paths.
fn semi_valid_programs() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple variable declarations
        "[a-z]{1,8}".prop_map(|name| format!("let {} = 42;", name)),
        // Function declarations
        "[a-z]{1,8}".prop_map(|name| format!("fn {}() {{ return 1; }}", name)),
        // If statements
        Just("if (true) { let x = 1; } else { let y = 2; }".to_string()),
        // While loops
        Just("while (true) { break; }".to_string()),
        // Nested expressions
        "[a-z]{1,4}".prop_map(|v| format!("let {} = (1 + 2) * (3 - 4) / 5;", v)),
        // Empty input
        Just("".to_string()),
        // Only whitespace
        "[ \\t\\n\\r]{0,64}",
        // Only comments
        Just("// this is a comment\n/* block comment */".to_string()),
    ]
}

// ---------------------------------------------------------------------------
// Property tests — the parser must NEVER panic
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Feeding completely random UTF-8 strings must not panic.
    #[test]
    fn fuzz_parse_random_utf8(input in arbitrary_utf8()) {
        // We don't care about the result — only that it doesn't panic.
        let _ = hudhudscript_parser::parse(&input);
    }

    /// Feeding code-like ASCII strings must not panic.
    #[test]
    fn fuzz_parse_code_like_ascii(input in code_like_ascii()) {
        let _ = hudhudscript_parser::parse(&input);
    }

    /// Feeding keyword soups must not panic.
    #[test]
    fn fuzz_parse_keyword_soup(input in keyword_soup()) {
        let _ = hudhudscript_parser::parse(&input);
    }

    /// Feeding multilingual/Unicode input must not panic.
    #[test]
    fn fuzz_parse_multilingual(input in multilingual_input()) {
        let _ = hudhudscript_parser::parse(&input);
    }

    /// Semi-valid programs should parse without panicking.
    #[test]
    fn fuzz_parse_semi_valid(input in semi_valid_programs()) {
        let _ = hudhudscript_parser::parse(&input);
    }

    /// parse_with_recovery must not panic on any input.
    #[test]
    fn fuzz_parse_with_recovery_random(input in arbitrary_utf8()) {
        let _ = hudhudscript_parser::parse_with_recovery(&input);
    }

    /// parse_with_lang_directive must not panic on any input.
    #[test]
    fn fuzz_parse_with_lang_directive_random(input in arbitrary_utf8()) {
        let _ = hudhudscript_parser::parse_with_lang_directive(&input);
    }

    /// Strings with embedded null bytes must not panic.
    #[test]
    fn fuzz_parse_with_null_bytes(
        prefix in "[a-z ]{0,32}",
        suffix in "[a-z ]{0,32}"
    ) {
        let input = format!("{}\0{}", prefix, suffix);
        let _ = hudhudscript_parser::parse(&input);
    }

    /// Moderately nested parentheses/braces must not panic.
    /// NOTE: depth > ~100 causes stack overflow in the PEG parser (known issue).
    /// A separate non-proptest test documents this limitation below.
    #[test]
    fn fuzz_parse_nesting(depth in 1usize..20) {
        let open_parens: String = "(".repeat(depth);
        let close_parens: String = ")".repeat(depth);
        let input = format!("let x = {}1{};", open_parens, close_parens);
        let _ = hudhudscript_parser::parse(&input);

        let open_braces: String = "{".repeat(depth);
        let close_braces: String = "}".repeat(depth);
        let input2 = format!("{}let x = 1;{}", open_braces, close_braces);
        let _ = hudhudscript_parser::parse(&input2);
    }

    /// Long strings should not cause excessive memory usage or panics.
    #[test]
    fn fuzz_parse_long_identifiers(len in 1usize..2000) {
        let ident: String = "a".repeat(len);
        let input = format!("let {} = 1;", ident);
        let _ = hudhudscript_parser::parse(&input);
    }
}

// ---------------------------------------------------------------------------
// Known-issue documentation tests (outside proptest)
// ---------------------------------------------------------------------------

/// Documents that deeply nested input (depth >= ~100) causes a stack overflow
/// in the PEG-based parser. This is a known limitation tracked separately.
/// The test is ignored by default so it does not abort the test suite.
#[test]
#[ignore = "known issue: PEG parser stack overflow at depth ~100+ (Issue #973 finding)"]
fn known_issue_deep_nesting_stack_overflow() {
    let depth = 150;
    let open: String = "(".repeat(depth);
    let close: String = ")".repeat(depth);
    let input = format!("let x = {}1{};", open, close);
    // This will stack-overflow on current parser; kept as documentation.
    let _ = hudhudscript_parser::parse(&input);
}
