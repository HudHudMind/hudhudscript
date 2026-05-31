use hudhudscript_ast::Position;
use hudhudscript_errors::{Error, ErrorCode, SourcePosition};

    fn pos_to_source(pos: Position) -> SourcePosition {
        SourcePosition::new(pos.line, pos.column, pos.offset)
    }

    pub fn unexpected_char(c: char, pos: Position) -> Error {
        Error::new(
            ErrorCode::LexUnexpectedChar,
            format!("Unexpected character '{}' at {:?}", c, pos),
        )
        .at(pos_to_source(pos))
        .with_context("char", c.to_string())
    }

    pub fn unterminated_string(pos: Position) -> Error {
        Error::new(
            ErrorCode::LexUnterminatedString,
            format!("Unterminated string at {:?}", pos),
        )
        .at(pos_to_source(pos))
    }

    pub fn invalid_number(pos: Position) -> Error {
        Error::new(
            ErrorCode::LexInvalidNumber,
            format!("Invalid number format at {:?}", pos),
        )
        .at(pos_to_source(pos))
    }

    pub fn invalid_escape(c: char, pos: Position) -> Error {
        Error::new(
            ErrorCode::LexInvalidEscape,
            format!("Invalid escape sequence '\\{}' at {:?}", c, pos),
        )
        .at(pos_to_source(pos))
        .with_context("char", c.to_string())
    }
