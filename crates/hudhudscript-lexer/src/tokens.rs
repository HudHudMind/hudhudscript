use hudhudscript_ast::{Position, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    Null,

    // Identifiers and Keywords
    Identifier(String),
    Keyword(String),

    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Equal,        // =
    EqualEqual,   // ==
    BangEqual,    // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    AmpAmp,       // &&
    PipePipe,     // ||
    Bang,         // !

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Semicolon,    // ;
    Colon,        // :
    Comma,        // ,
    Dot,          // .

    // Special
    Eof,
}

/// Token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub position: Position,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, position: Position, span: Span) -> Self {
        Self {
            kind,
            position,
            span,
        }
    }
}
