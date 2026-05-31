use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ParseErrorCode {
    /// E0182 — Invalid syntax
    ParseInvalidSyntax = 182,
    /// E0183 — Lexer error surfaced during parsing
    ParseLexerError = 183,
    /// E0184 — Unexpected end of file while parsing
    ParseUnexpectedEof = 184,
    /// E0185 — Unexpected token in input
    ParseUnexpectedToken = 185,
}
