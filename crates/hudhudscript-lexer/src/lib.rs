pub mod keyword_normalizer;
pub use keyword_normalizer::normalize_keywords;
mod charclass;
mod lex_error;
mod lexer;
mod numerals;
mod tokens;

pub use charclass::{is_ident_continue, is_ident_start};
pub use lex_error::lex_codes;
pub use lex_error::LexError;
pub use lexer::Lexer;
pub use numerals::japanese_numeral_to_number;
pub use tokens::{Token, TokenKind};
