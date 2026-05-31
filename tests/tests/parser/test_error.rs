//! External tests for `hudhudscript_parser::error` module.
//!
//! v0.4.48 TAM CONSOLIDATION: ParseError is now a type alias for the unified
//! `hudhudscript_errors::Error`. Variant pattern matches now check `err.code`
//! and `err.context_get(...)` for the destructured fields.

use hudhudscript_ast::{Position, Span};
use hudhudscript_errors::ErrorCode;
use hudhudscript_lexer::{Token, TokenKind};
use hudhudscript_parser::{parse_codes, ParseError};

fn pos(line: usize, column: usize, offset: usize) -> Position {
    Position::new(line, column, offset)
}

fn span(sl: usize, sc: usize, so: usize, el: usize, ec: usize, eo: usize) -> Span {
    Span {
        start: Position::new(sl, sc, so),
        end: Position::new(el, ec, eo),
    }
}

// ── Constructor tests ──────────────────────────────────────────────

#[test]
fn test_unexpected_token_constructor() {
    let err = parse_codes::unexpected_token(
        "identifier",
        Token::new(TokenKind::Eof, Position::new(0, 0, 0), Span::default()),
        pos(1, 5, 4),
    );
    assert_eq!(err.code, ErrorCode::ParseUnexpectedToken);
    assert_eq!(err.context_get("expected"), Some("identifier"));
    assert_eq!(err.context_get("found"), Some("Eof"));
    let p = err.position.as_ref().unwrap();
    assert_eq!(p.line, 1);
    assert_eq!(p.column, 5);
    assert_eq!(p.offset, 4);
}

#[test]
fn test_unexpected_eof_constructor() {
    let err = parse_codes::unexpected_eof(pos(10, 1, 100));
    assert_eq!(err.code, ErrorCode::ParseUnexpectedEof);
    let p = err.position.as_ref().unwrap();
    assert_eq!(p.line, 10);
    assert_eq!(p.column, 1);
    assert_eq!(p.offset, 100);
}

#[test]
fn test_invalid_syntax_constructor() {
    let err = parse_codes::invalid_syntax("bad syntax", span(1, 1, 0, 1, 10, 9));
    assert_eq!(err.code, ErrorCode::ParseInvalidSyntax);
    assert_eq!(err.context_get("message"), Some("bad syntax"));
    let p = err.position.as_ref().unwrap();
    assert_eq!(p.line, 1);
}

#[test]
fn test_lexer_error_constructor() {
    let err = parse_codes::lexer_error("bad token");
    assert_eq!(err.code, ErrorCode::ParseLexerError);
    assert_eq!(err.context_get("message"), Some("bad token"));
}

// ── position field ─────────────────────────────────────────────────

#[test]
fn test_position_unexpected_token() {
    let err = parse_codes::unexpected_token(
        "x",
        Token::new(TokenKind::Eof, Position::new(0, 0, 0), Span::default()),
        pos(3, 7, 20),
    );
    let p = err.position.as_ref().unwrap();
    assert_eq!(p.line, 3);
    assert_eq!(p.column, 7);
    assert_eq!(p.offset, 20);
}

#[test]
fn test_position_unexpected_eof() {
    let err = parse_codes::unexpected_eof(pos(5, 1, 50));
    let p = err.position.as_ref().unwrap();
    assert_eq!(p.line, 5);
}

#[test]
fn test_position_invalid_syntax() {
    let err = parse_codes::invalid_syntax("err", span(2, 3, 10, 2, 15, 22));
    let p = err.position.as_ref().unwrap();
    assert_eq!(p.line, 2);
    assert_eq!(p.column, 3);
}

#[test]
fn test_position_lexer_error_returns_none() {
    let err = parse_codes::lexer_error("bad");
    assert!(err.position.is_none());
}

// ── Display / Error trait ──────────────────────────────────────────

#[test]
fn test_unexpected_token_display() {
    let err = parse_codes::unexpected_token(
        "identifier",
        Token::new(TokenKind::Eof, Position::new(0, 0, 0), Span::default()),
        pos(1, 1, 0),
    );
    let msg = err.to_string();
    assert!(msg.contains("Unexpected"));
    assert!(msg.contains("identifier"));
}

#[test]
fn test_unexpected_eof_display() {
    let err = parse_codes::unexpected_eof(pos(1, 1, 0));
    assert!(err.to_string().contains("Unexpected"));
}

#[test]
fn test_invalid_syntax_display() {
    let err = parse_codes::invalid_syntax("missing semicolon", Default::default());
    let msg = err.to_string();
    assert!(msg.contains("Invalid") || msg.contains("invalid"));
    assert!(msg.contains("missing semicolon"));
}

#[test]
fn test_lexer_error_display() {
    let err = parse_codes::lexer_error("invalid character");
    let msg = err.to_string();
    assert!(msg.contains("Lexer") || msg.contains("lex"));
    assert!(msg.contains("invalid character"));
}

// ── code field ─────────────────────────────────────────────────────

#[test]
fn test_to_diagnostic_with_position() {
    let err = parse_codes::unexpected_token(
        "x",
        Token::new(TokenKind::Eof, Position::new(0, 0, 0), Span::default()),
        pos(5, 10, 42),
    );
    assert_eq!(err.code, ErrorCode::ParseUnexpectedToken);
    assert!(err.message.contains("Unexpected"));
}

#[test]
fn test_to_diagnostic_lexer_no_position() {
    let err = parse_codes::lexer_error("bad");
    assert_eq!(err.code, ErrorCode::ParseLexerError);
}

// ── Clone and PartialEq ────────────────────────────────────────────

#[test]
fn test_parse_error_clone() {
    let err = parse_codes::invalid_syntax("test", Default::default());
    let cloned: ParseError = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn test_parse_error_ne() {
    let err1 = parse_codes::lexer_error("a");
    let err2 = parse_codes::lexer_error("b");
    assert_ne!(err1, err2);
}

// ── unified Error carries position and code ────────────────────────
//
// (v0.4.48 TAM CONSOLIDATION: ParseError IS Error now, so the previous
// From<ParseError> for HudHudError test no longer makes sense — the
// conversion is identity. The test below verifies the unified Error
// carries the catalog code and source position directly.)

#[test]
fn test_parse_error_carries_position_and_code() {
    let err = parse_codes::unexpected_eof(pos(3, 5, 20));
    assert_eq!(err.code, ErrorCode::ParseUnexpectedEof);
    let p = err.position.as_ref().unwrap();
    assert_eq!(p.line, 3);
    assert_eq!(p.column, 5);
    assert_eq!(p.offset, 20);
}

#[test]
fn test_lexer_error_no_position() {
    let err = parse_codes::lexer_error("bad token");
    assert_eq!(err.code, ErrorCode::ParseLexerError);
    assert!(err.position.is_none());
}
