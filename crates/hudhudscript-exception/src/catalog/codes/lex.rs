use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum LexExceptionCode {
    /// E0118 — Invalid escape sequence in string literal
    LexInvalidEscape = 118,
    /// E0119 — Malformed numeric literal
    LexInvalidNumber = 119,
    /// E0120 — Unexpected character in source
    LexUnexpectedChar = 120,
    /// E0121 — Unterminated string literal
    LexUnterminatedString = 121,
}
