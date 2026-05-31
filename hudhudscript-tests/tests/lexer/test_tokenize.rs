use hudhudscript_ast::{Position, Span};
use hudhudscript_errors::ErrorCode;
use hudhudscript_lexer::lex_codes;
use hudhudscript_lexer::*;

// ============================================================================
// BASIC LITERAL TOKENS
// ============================================================================

#[test]
fn tokenize_integer_number() {
    let mut lexer = Lexer::new("42");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Number(42.0));
    assert_eq!(tokens[1].kind, TokenKind::Eof);
}

#[test]
fn tokenize_float_number() {
    let mut lexer = Lexer::new("3.14");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number(3.14));
}

#[test]
fn tokenize_zero() {
    let mut lexer = Lexer::new("0");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number(0.0));
}

#[test]
fn tokenize_large_integer() {
    let mut lexer = Lexer::new("1000000");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number(1_000_000.0));
}

#[test]
fn tokenize_small_float() {
    let mut lexer = Lexer::new("0.001");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number(0.001));
}

#[test]
fn tokenize_string_literal() {
    let mut lexer = Lexer::new(r#""hello""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String("hello".to_string()));
}

#[test]
fn tokenize_empty_string() {
    let mut lexer = Lexer::new(r#""""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String(String::new()));
}

#[test]
fn tokenize_string_with_spaces() {
    let mut lexer = Lexer::new(r#""hello world""#);
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::String("hello world".to_string()));
}

#[test]
fn tokenize_boolean_true() {
    let mut lexer = Lexer::new("true");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Boolean(true));
}

#[test]
fn tokenize_boolean_false() {
    let mut lexer = Lexer::new("false");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Boolean(false));
}

#[test]
fn tokenize_null() {
    let mut lexer = Lexer::new("null");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Null);
}

#[test]
fn tokenize_identifier_simple() {
    let mut lexer = Lexer::new("foo");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
}

#[test]
fn tokenize_identifier_underscore_prefix() {
    let mut lexer = Lexer::new("_foo");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("_foo".to_string()));
}

#[test]
fn tokenize_identifier_with_digits() {
    let mut lexer = Lexer::new("foo42bar");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Identifier("foo42bar".to_string())
    );
}

#[test]
fn tokenize_identifier_all_underscores() {
    let mut lexer = Lexer::new("___");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("___".to_string()));
}

// ============================================================================
// ALL KEYWORDS
// ============================================================================

#[test]
fn tokenize_keyword_agent() {
    let mut lexer = Lexer::new("agent");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("agent".to_string())
    );
}

#[test]
fn tokenize_keyword_task() {
    let mut lexer = Lexer::new("task");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("task".to_string())
    );
}

#[test]
fn tokenize_keyword_tool() {
    let mut lexer = Lexer::new("tool");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("tool".to_string())
    );
}

#[test]
fn tokenize_keyword_resource() {
    let mut lexer = Lexer::new("resource");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("resource".to_string())
    );
}

#[test]
fn tokenize_keyword_mcp() {
    let mut lexer = Lexer::new("mcp");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("mcp".to_string())
    );
}

#[test]
fn tokenize_keyword_server() {
    let mut lexer = Lexer::new("server");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("server".to_string())
    );
}

#[test]
fn tokenize_keyword_config() {
    let mut lexer = Lexer::new("config");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("config".to_string())
    );
}

#[test]
fn tokenize_keyword_import() {
    let mut lexer = Lexer::new("import");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("import".to_string())
    );
}

#[test]
fn tokenize_keyword_export() {
    let mut lexer = Lexer::new("export");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("export".to_string())
    );
}

#[test]
fn tokenize_keyword_if() {
    let mut lexer = Lexer::new("if");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("if".to_string())
    );
}

#[test]
fn tokenize_keyword_else() {
    let mut lexer = Lexer::new("else");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("else".to_string())
    );
}

#[test]
fn tokenize_keyword_while() {
    let mut lexer = Lexer::new("while");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("while".to_string())
    );
}

#[test]
fn tokenize_keyword_for() {
    let mut lexer = Lexer::new("for");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("for".to_string())
    );
}

#[test]
fn tokenize_keyword_return() {
    let mut lexer = Lexer::new("return");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("return".to_string())
    );
}

#[test]
fn tokenize_keyword_async() {
    let mut lexer = Lexer::new("async");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("async".to_string())
    );
}

#[test]
fn tokenize_keyword_await() {
    let mut lexer = Lexer::new("await");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("await".to_string())
    );
}

#[test]
fn tokenize_keyword_as() {
    let mut lexer = Lexer::new("as");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("as".to_string())
    );
}

#[test]
fn tokenize_keyword_from() {
    let mut lexer = Lexer::new("from");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("from".to_string())
    );
}

#[test]
fn tokenize_keyword_let() {
    let mut lexer = Lexer::new("let");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("let".to_string())
    );
}

#[test]
fn tokenize_keyword_const() {
    let mut lexer = Lexer::new("const");
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Keyword("const".to_string())
    );
}

#[test]
fn tokenize_all_keywords_in_sequence() {
    let keywords = [
        "agent", "task", "tool", "resource", "mcp", "server", "config", "import", "export", "if",
        "else", "while", "for", "return", "async", "await", "as", "from", "let", "const",
    ];
    for kw in &keywords {
        let mut lexer = Lexer::new(kw);
        let tok = lexer.next_token().unwrap();
        assert_eq!(
            tok.kind,
            TokenKind::Keyword(kw.to_string()),
            "Failed for keyword '{}'",
            kw
        );
    }
}

// ============================================================================
// ALL OPERATORS
// ============================================================================

#[test]
fn tokenize_plus() {
    let mut lexer = Lexer::new("+");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
}

#[test]
fn tokenize_minus() {
    let mut lexer = Lexer::new("-");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Minus);
}

#[test]
fn tokenize_star() {
    let mut lexer = Lexer::new("*");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Star);
}

#[test]
fn tokenize_slash() {
    let mut lexer = Lexer::new("10 / 2");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[1].kind, TokenKind::Slash);
}

#[test]
fn tokenize_percent() {
    let mut lexer = Lexer::new("%");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Percent);
}

#[test]
fn tokenize_equal() {
    let mut lexer = Lexer::new("=");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Equal);
}

#[test]
fn tokenize_equal_equal() {
    let mut lexer = Lexer::new("==");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::EqualEqual);
}

#[test]
fn tokenize_bang_equal() {
    let mut lexer = Lexer::new("!=");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::BangEqual);
}

#[test]
fn tokenize_less() {
    let mut lexer = Lexer::new("<");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Less);
}

#[test]
fn tokenize_less_equal() {
    let mut lexer = Lexer::new("<=");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LessEqual);
}

#[test]
fn tokenize_greater() {
    let mut lexer = Lexer::new(">");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Greater);
}

#[test]
fn tokenize_greater_equal() {
    let mut lexer = Lexer::new(">=");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::GreaterEqual);
}

#[test]
fn tokenize_amp_amp() {
    let mut lexer = Lexer::new("&&");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::AmpAmp);
}

#[test]
fn tokenize_pipe_pipe() {
    let mut lexer = Lexer::new("||");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::PipePipe);
}

#[test]
fn tokenize_bang() {
    let mut lexer = Lexer::new("!");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Bang);
}

#[test]
fn tokenize_all_operators_in_sequence() {
    let mut lexer = Lexer::new("+ - * % = == != < <= > >= && || !");
    let tokens = lexer.tokenize().unwrap();
    let expected = [
        TokenKind::Plus,
        TokenKind::Minus,
        TokenKind::Star,
        TokenKind::Percent,
        TokenKind::Equal,
        TokenKind::EqualEqual,
        TokenKind::BangEqual,
        TokenKind::Less,
        TokenKind::LessEqual,
        TokenKind::Greater,
        TokenKind::GreaterEqual,
        TokenKind::AmpAmp,
        TokenKind::PipePipe,
        TokenKind::Bang,
        TokenKind::Eof,
    ];
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(tokens[i].kind, *exp, "Mismatch at index {}", i);
    }
}

// ============================================================================
// ALL DELIMITERS
// ============================================================================

#[test]
fn tokenize_left_paren() {
    let mut lexer = Lexer::new("(");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LeftParen);
}

#[test]
fn tokenize_right_paren() {
    let mut lexer = Lexer::new(")");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RightParen);
}

#[test]
fn tokenize_left_brace() {
    let mut lexer = Lexer::new("{");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LeftBrace);
}

#[test]
fn tokenize_right_brace() {
    let mut lexer = Lexer::new("}");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RightBrace);
}

#[test]
fn tokenize_left_bracket() {
    let mut lexer = Lexer::new("[");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LeftBracket);
}

#[test]
fn tokenize_right_bracket() {
    let mut lexer = Lexer::new("]");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RightBracket);
}

#[test]
fn tokenize_semicolon() {
    let mut lexer = Lexer::new(";");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Semicolon);
}

#[test]
fn tokenize_colon() {
    let mut lexer = Lexer::new(":");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
}

#[test]
fn tokenize_comma() {
    let mut lexer = Lexer::new(",");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Comma);
}

#[test]
fn tokenize_dot() {
    let mut lexer = Lexer::new(".");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Dot);
}

#[test]
fn tokenize_all_delimiters() {
    let mut lexer = Lexer::new("(){}[];:,.");
    let tokens = lexer.tokenize().unwrap();
    let expected = [
        TokenKind::LeftParen,
        TokenKind::RightParen,
        TokenKind::LeftBrace,
        TokenKind::RightBrace,
        TokenKind::LeftBracket,
        TokenKind::RightBracket,
        TokenKind::Semicolon,
        TokenKind::Colon,
        TokenKind::Comma,
        TokenKind::Dot,
        TokenKind::Eof,
    ];
    for (i, exp) in expected.iter().enumerate() {
        assert_eq!(tokens[i].kind, *exp, "Delimiter mismatch at index {}", i);
    }
}

// ============================================================================
// ESCAPE SEQUENCES
// ============================================================================

#[test]
fn tokenize_escape_newline() {
    let mut lexer = Lexer::new(r#""a\nb""#);
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::String("a\nb".to_string()));
}

#[test]
fn tokenize_escape_tab() {
    let mut lexer = Lexer::new(r#""a\tb""#);
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::String("a\tb".to_string()));
}

#[test]
fn tokenize_escape_carriage_return() {
    let mut lexer = Lexer::new(r#""a\rb""#);
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::String("a\rb".to_string()));
}

#[test]
fn tokenize_escape_backslash() {
    let mut lexer = Lexer::new(r#""a\\b""#);
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::String("a\\b".to_string()));
}

#[test]
fn tokenize_escape_quote() {
    let mut lexer = Lexer::new(r#""a\"b""#);
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::String("a\"b".to_string()));
}

#[test]
fn tokenize_all_escapes_combined() {
    let mut lexer = Lexer::new(r#""hello\nworld\t!\rend\\slash\"quote""#);
    let tok = lexer.next_token().unwrap();
    assert_eq!(
        tok.kind,
        TokenKind::String("hello\nworld\t!\rend\\slash\"quote".to_string())
    );
}

// ============================================================================
// UNICODE / ARABIC-INDIC DIGITS
// ============================================================================

#[test]
fn tokenize_arabic_indic_123() {
    let mut lexer = Lexer::new("\u{0661}\u{0662}\u{0663}");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::Number(123.0));
}

#[test]
fn tokenize_arabic_indic_zero() {
    let mut lexer = Lexer::new("\u{0660}");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::Number(0.0));
}

#[test]
fn tokenize_arabic_indic_all_digits() {
    // digits 0-9 in Arabic-Indic = 0123456789
    let input: String = ('\u{0660}'..='\u{0669}').collect();
    let mut lexer = Lexer::new(&input);
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::Number(123456789.0));
}

#[test]
fn tokenize_mixed_arabic_ascii() {
    let mut lexer = Lexer::new("\u{0661}2\u{0663}");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::Number(123.0));
}

#[test]
fn tokenize_arabic_indic_float() {
    // 3.14 in Arabic-Indic
    let mut lexer = Lexer::new("\u{0663}.\u{0661}\u{0664}");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::Number(3.14));
}

// ============================================================================
// JAPANESE NUMERALS
// ============================================================================

#[test]
fn tokenize_japanese_zero_maru() {
    let mut lexer = Lexer::new("\u{3007}"); // 〇
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(0.0));
}

#[test]
fn tokenize_japanese_zero_rei() {
    let mut lexer = Lexer::new("\u{96F6}"); // 零
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(0.0));
}

#[test]
fn tokenize_japanese_one_through_nine() {
    let cases = [
        ('\u{4E00}', 1.0), // 一
        ('\u{4E8C}', 2.0), // 二
        ('\u{4E09}', 3.0), // 三
        ('\u{56DB}', 4.0), // 四
        ('\u{4E94}', 5.0), // 五
        ('\u{516D}', 6.0), // 六
        ('\u{4E03}', 7.0), // 七
        ('\u{516B}', 8.0), // 八
        ('\u{4E5D}', 9.0), // 九
    ];
    for (ch, expected) in &cases {
        let mut lexer = Lexer::new(&ch.to_string());
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Number(*expected),
            "Failed for '{}'",
            ch
        );
    }
}

#[test]
fn tokenize_japanese_ten() {
    let mut lexer = Lexer::new("\u{5341}"); // 十
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(10.0));
}

#[test]
fn tokenize_japanese_hundred() {
    let mut lexer = Lexer::new("\u{767E}"); // 百
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(100.0));
}

#[test]
fn tokenize_japanese_thousand() {
    let mut lexer = Lexer::new("\u{5343}"); // 千
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(1000.0));
}

#[test]
fn tokenize_japanese_ten_thousand() {
    let mut lexer = Lexer::new("\u{4E07}"); // 万
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Number(10000.0));
}

#[test]
fn tokenize_japanese_hundred_million() {
    let mut lexer = Lexer::new("\u{5104}"); // 億
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Number(100000000.0)
    );
}

// ============================================================================
// ALL ERROR TYPES
// ============================================================================

// v0.4.48 TAM CONSOLIDATION: LexError is now a type alias for the unified
// hudhudscript_errors::Error. Variant pattern matches now check err.code
// and err.context_get("char") for the offending character.

#[test]
fn error_unexpected_char() {
    let mut lexer = Lexer::new("@");
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexUnexpectedChar);
    assert_eq!(err.context_get("char"), Some("@"));
}

#[test]
fn error_unexpected_char_tilde() {
    let mut lexer = Lexer::new("~");
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexUnexpectedChar);
    assert_eq!(err.context_get("char"), Some("~"));
}

#[test]
fn error_unterminated_string() {
    let mut lexer = Lexer::new(r#""hello"#);
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexUnterminatedString);
}

#[test]
fn error_unterminated_string_after_backslash() {
    let mut lexer = Lexer::new("\"hello\\");
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexUnterminatedString);
}

#[test]
fn error_invalid_escape() {
    let mut lexer = Lexer::new(r#""hello\x""#);
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexInvalidEscape);
    assert_eq!(err.context_get("char"), Some("x"));
}

#[test]
fn error_invalid_escape_z() {
    let mut lexer = Lexer::new(r#""\z""#);
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexInvalidEscape);
    assert_eq!(err.context_get("char"), Some("z"));
}

#[test]
fn error_single_ampersand() {
    let mut lexer = Lexer::new("& ");
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexUnexpectedChar);
    assert_eq!(err.context_get("char"), Some("&"));
}

#[test]
fn error_single_pipe() {
    let mut lexer = Lexer::new("| ");
    let err = lexer.next_token().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexUnexpectedChar);
    assert_eq!(err.context_get("char"), Some("|"));
}

#[test]
fn error_unterminated_block_comment() {
    let mut lexer = Lexer::new("42 /* unterminated");
    let err = lexer.tokenize().unwrap_err();
    assert_eq!(err.code, ErrorCode::LexUnexpectedChar);
    assert_eq!(err.context_get("char"), Some("/"));
}

// ============================================================================
// ERROR POSITION EXTRACTION
// ============================================================================

#[test]
fn lex_error_position_unexpected_char() {
    let err = lex_codes::unexpected_char('@', Position::new(3, 5, 20));
    let pos = err.position.as_ref().unwrap();
    assert_eq!(pos.line, 3);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.offset, 20);
}

#[test]
fn lex_error_position_unterminated_string() {
    let err = lex_codes::unterminated_string(Position::new(1, 1, 0));
    assert_eq!(err.position.as_ref().unwrap().line, 1);
}

#[test]
fn lex_error_position_invalid_number() {
    let err = lex_codes::invalid_number(Position::new(2, 3, 10));
    let pos = err.position.as_ref().unwrap();
    assert_eq!(pos.line, 2);
    assert_eq!(pos.column, 3);
}

#[test]
fn lex_error_position_invalid_escape() {
    let err = lex_codes::invalid_escape('x', Position::new(4, 7, 30));
    assert_eq!(err.position.as_ref().unwrap().line, 4);
}

// ============================================================================
// ERROR DISPLAY
// ============================================================================

#[test]
fn lex_error_display_unexpected_char() {
    let err = lex_codes::unexpected_char('@', Position::new(1, 1, 0));
    let msg = format!("{}", err);
    assert!(msg.contains("Unexpected"));
    assert!(msg.contains('@'));
}

#[test]
fn lex_error_display_unterminated_string() {
    let msg = format!("{}", lex_codes::unterminated_string(Position::new(1, 1, 0)));
    assert!(msg.contains("Unterminated"));
}

#[test]
fn lex_error_display_invalid_number() {
    let msg = format!("{}", lex_codes::invalid_number(Position::new(1, 1, 0)));
    assert!(msg.contains("Invalid Number") || msg.contains("Invalid number"));
}

#[test]
fn lex_error_display_invalid_escape() {
    let msg = format!("{}", lex_codes::invalid_escape('z', Position::new(1, 1, 0)));
    assert!(msg.contains("Invalid"));
    assert!(msg.contains('z'));
}

// ============================================================================
// ERROR CONVERSION TO HUDHUD ERROR
// ============================================================================

#[test]
fn lex_error_carries_position_and_code() {
    // v0.4.48 TAM CONSOLIDATION: LexError IS Error now, so the previous
    // From<LexError> for HudHudError test no longer makes sense (the
    // conversion is identity). The test now verifies the unified Error
    // carries the catalog code and source position directly.
    let err = lex_codes::unexpected_char('@', Position::new(3, 5, 20));
    assert_eq!(err.code, ErrorCode::LexUnexpectedChar);
    assert_eq!(err.context_get("char"), Some("@"));
    let pos = err.position.as_ref().unwrap();
    assert_eq!(pos.line, 3);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.offset, 20);
}

// ============================================================================
// POSITION TRACKING
// ============================================================================

#[test]
fn position_first_token_line_1_col_1() {
    let mut lexer = Lexer::new("foo");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.position.line, 1);
    assert_eq!(tok.position.column, 1);
    assert_eq!(tok.position.offset, 0);
}

#[test]
fn position_after_newline() {
    let mut lexer = Lexer::new("foo\nbar");
    let _ = lexer.next_token().unwrap(); // foo
    let tok2 = lexer.next_token().unwrap(); // bar
    assert_eq!(tok2.position.line, 2);
    assert_eq!(tok2.position.column, 1);
}

#[test]
fn position_after_spaces() {
    let mut lexer = Lexer::new("   foo");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.position.line, 1);
    assert_eq!(tok.position.column, 4);
}

#[test]
fn position_multiple_lines() {
    let mut lexer = Lexer::new("a\nb\nc");
    let t1 = lexer.next_token().unwrap();
    assert_eq!(t1.position.line, 1);
    let t2 = lexer.next_token().unwrap();
    assert_eq!(t2.position.line, 2);
    let t3 = lexer.next_token().unwrap();
    assert_eq!(t3.position.line, 3);
}

#[test]
fn span_tracking_identifier() {
    let mut lexer = Lexer::new("foo");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.span.start.line, 1);
    assert_eq!(tok.span.start.column, 1);
    assert_eq!(tok.span.end.line, 1);
    assert_eq!(tok.span.end.column, 4); // end is exclusive
}

#[test]
fn span_tracking_string() {
    let mut lexer = Lexer::new(r#""hi""#);
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.span.start.column, 1);
    assert_eq!(tok.span.end.column, 5); // "hi" is 4 chars
}

// ============================================================================
// COMMENTS
// ============================================================================

#[test]
fn skip_line_comment() {
    let mut lexer = Lexer::new("42 // this is a comment\n43");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number(42.0));
    assert_eq!(tokens[1].kind, TokenKind::Number(43.0));
    assert_eq!(tokens[2].kind, TokenKind::Eof);
}

#[test]
fn skip_block_comment() {
    let mut lexer = Lexer::new("42 /* block */ 43");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number(42.0));
    assert_eq!(tokens[1].kind, TokenKind::Number(43.0));
}

#[test]
fn skip_multiline_block_comment() {
    let mut lexer = Lexer::new("1 /* multi\nline\ncomment */ 2");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number(1.0));
    assert_eq!(tokens[1].kind, TokenKind::Number(2.0));
}

// ============================================================================
// TOKEN CONSTRUCTOR
// ============================================================================

#[test]
fn token_new_fields() {
    let pos = Position::new(2, 3, 10);
    let span = Span::new(pos, Position::new(2, 5, 12));
    let tok = Token::new(TokenKind::Plus, pos, span);
    assert_eq!(tok.kind, TokenKind::Plus);
    assert_eq!(tok.position, pos);
    assert_eq!(tok.span, span);
}

// ============================================================================
// TOKENIZE FULL PROGRAMS
// ============================================================================

#[test]
fn tokenize_let_statement() {
    let mut lexer = Lexer::new("let x = 42;");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 6);
    assert_eq!(tokens[0].kind, TokenKind::Keyword("let".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Equal);
    assert_eq!(tokens[3].kind, TokenKind::Number(42.0));
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    assert_eq!(tokens[5].kind, TokenKind::Eof);
}

#[test]
fn tokenize_function_call() {
    let mut lexer = Lexer::new("foo(1, 2)");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::LeftParen);
    assert_eq!(tokens[2].kind, TokenKind::Number(1.0));
    assert_eq!(tokens[3].kind, TokenKind::Comma);
    assert_eq!(tokens[4].kind, TokenKind::Number(2.0));
    assert_eq!(tokens[5].kind, TokenKind::RightParen);
}

#[test]
fn tokenize_if_else() {
    let mut lexer = Lexer::new("if (x == 1) { } else { }");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Keyword("if".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::LeftParen);
    assert_eq!(tokens[3].kind, TokenKind::EqualEqual);
    assert_eq!(tokens[6].kind, TokenKind::LeftBrace);
    assert_eq!(tokens[8].kind, TokenKind::Keyword("else".to_string()));
}

#[test]
fn tokenize_empty_source() {
    let mut lexer = Lexer::new("");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn tokenize_whitespace_only() {
    let mut lexer = Lexer::new("   \n\t  \n  ");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn tokenize_unicode_identifier() {
    let mut lexer = Lexer::new("myVar");
    let tok = lexer.next_token().unwrap();
    assert_eq!(tok.kind, TokenKind::Identifier("myVar".to_string()));
}

// ============================================================================
// KEYWORD NORMALIZER
// ============================================================================

#[test]
fn normalize_keywords_turkish_agent() {
    let result = normalize_keywords("ajan");
    assert_eq!(result, "agent");
}

#[test]
fn normalize_keywords_turkish_if() {
    let result = normalize_keywords("\u{0065}\u{011F}\u{0065}\u{0072}"); // eğer
    assert_eq!(result, "if");
}

#[test]
fn normalize_keywords_turkish_else() {
    let result = normalize_keywords("de\u{011F}ilse"); // değilse
    assert_eq!(result, "else");
}

#[test]
fn normalize_keywords_turkish_else_if() {
    let result = normalize_keywords("de\u{011F}ilse ama"); // değilse ama
    assert_eq!(result, "else if");
}

#[test]
fn normalize_keywords_identity_on_english() {
    let result = normalize_keywords("let x = 42;");
    assert_eq!(result, "let x = 42;");
}

#[test]
fn normalize_keywords_preserves_identifiers() {
    // "ajan" inside a larger identifier should not be replaced
    let result = normalize_keywords("my_ajan_name");
    assert_eq!(result, "my_ajan_name");
}

#[test]
fn normalize_keywords_multiple_replacements() {
    let result = normalize_keywords("e\u{011F}er g\u{00F6}rev d\u{00F6}n");
    assert_eq!(result, "if task return");
}
