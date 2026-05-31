//! Parser error types.
//!
//! v0.4.48 — TAM CONSOLIDATION (Anayasa Kural 1 İstisna authorized).
//! `ParseError` is now a type alias for the unified
//! [`hudhudscript_errors::Error`]. The four former variants
//! (`UnexpectedToken`, `UnexpectedEof`, `InvalidSyntax`, `LexerError`) are
//! constructed via the [`parse_codes`] module. Downstream code that used to
//! match on enum variants now matches on `error.code` against
//! `ErrorCode::Parse*`, and reads variant fields via `error.context_get`.

use hudhudscript_ast::{Position, Span};
use hudhudscript_lexer::Token;

pub type ParseResult<T> = Result<T, ParseError>;

/// Parser error — type alias for the unified [`hudhudscript_errors::Error`].
pub type ParseError = hudhudscript_errors::Error;

/// Constructor functions for parse-stage errors. Each builds an `Error`
/// with the appropriate catalog code, formatted message, source position,
/// and any context fields needed for downstream pattern matching.
pub mod parse_codes {
    use super::{Position, Span, Token};
    use hudhudscript_errors::{Error, ErrorCode, SourcePosition};

    fn pos_to_source(pos: Position) -> SourcePosition {
        SourcePosition::new(pos.line, pos.column, pos.offset)
    }

    pub fn unexpected_token(
        expected: impl Into<String>,
        found: Token,
        position: Position,
    ) -> Error {
        let expected = expected.into();
        let found_kind = format!("{:?}", found.kind);
        Error::new(
            ErrorCode::ParseUnexpectedToken,
            format!(
                "Unexpected token: expected {}, found {:?} at {:?}",
                expected, found, position
            ),
        )
        .at(pos_to_source(position))
        .with_context("expected", expected)
        .with_context("found", found_kind)
    }

    pub fn unexpected_eof(position: Position) -> Error {
        Error::new(
            ErrorCode::ParseUnexpectedEof,
            format!("Unexpected end of file at {:?}", position),
        )
        .at(pos_to_source(position))
    }

    pub fn invalid_syntax(message: impl Into<String>, span: Span) -> Error {
        let message = message.into();
        Error::new(
            ErrorCode::ParseInvalidSyntax,
            format!("Invalid syntax: {} at {:?}", message, span),
        )
        .at(pos_to_source(span.start))
        .with_context("message", message)
    }

    pub fn lexer_error(message: impl Into<String>) -> Error {
        let message = message.into();
        Error::new(
            ErrorCode::ParseLexerError,
            format!("Lexer error: {}", message),
        )
        .with_context("message", message)
    }
}
