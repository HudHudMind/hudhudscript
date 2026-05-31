use crate::lex_error::lex_codes;
use crate::numerals::{
    arabic_to_ascii_digit, is_arabic_digit, is_japanese_numeral, japanese_numeral_char_to_number,
};
use crate::{is_ident_continue, is_ident_start, LexError, Token, TokenKind};
use hudhudscript_ast::{Position, Span};

pub struct Lexer {
    source: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    offset: usize,
}

impl Lexer {
    /// Create a new lexer
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            offset: 0,
        }
    }

    /// Get current position
    fn position(&self) -> Position {
        Position::new(self.line, self.column, self.offset)
    }

    /// Check if at end of source
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Peek current character
    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source[self.current])
        }
    }

    /// Peek next character
    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            None
        } else {
            Some(self.source[self.current + 1])
        }
    }

    /// Advance to next character
    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        let ch = self.source[self.current];
        self.current += 1;
        self.offset += 1;

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(ch)
    }

    /// Skip whitespace
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip single-line comment
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Skip multi-line comment
    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        let start = self.position();

        while !self.is_at_end() {
            if self.peek() == Some('*') && self.peek_next() == Some('/') {
                self.advance(); // *
                self.advance(); // /
                return Ok(());
            }
            self.advance();
        }

        Err(lex_codes::unexpected_char('/', start))
    }

    /// Lex a string literal
    fn lex_string(&mut self) -> Result<Token, LexError> {
        let start = self.position();
        self.advance(); // opening "

        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // closing "
                let end = self.position();
                return Ok(Token::new(
                    TokenKind::String(value),
                    start,
                    Span::new(start, end),
                ));
            }

            if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        value.push('\n');
                        self.advance();
                    }
                    Some('t') => {
                        value.push('\t');
                        self.advance();
                    }
                    Some('r') => {
                        value.push('\r');
                        self.advance();
                    }
                    Some('\\') => {
                        value.push('\\');
                        self.advance();
                    }
                    Some('"') => {
                        value.push('"');
                        self.advance();
                    }
                    Some(c) => {
                        return Err(lex_codes::invalid_escape(c, self.position()));
                    }
                    None => {
                        return Err(lex_codes::unterminated_string(start));
                    }
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        Err(lex_codes::unterminated_string(start))
    }

    /// Lex a number literal
    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.position();

        // Check if it's a Japanese numeral
        if let Some(ch) = self.peek() {
            if is_japanese_numeral(ch) {
                if let Some(num) = japanese_numeral_char_to_number(ch) {
                    self.advance();
                    let end = self.position();
                    return Ok(Token::new(
                        TokenKind::Number(num),
                        start,
                        Span::new(start, end),
                    ));
                }
            }
        }

        // Regular number parsing (ASCII or Arabic-Indic)
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || is_arabic_digit(ch) {
                // Convert Arabic-Indic digits to ASCII
                let ascii_digit = if is_arabic_digit(ch) {
                    arabic_to_ascii_digit(ch)
                } else {
                    ch
                };
                value.push(ascii_digit);
                self.advance();
            } else if ch == '.'
                && self
                    .peek_next()
                    .is_some_and(|c| c.is_ascii_digit() || is_arabic_digit(c))
            {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let num = value
            .parse::<f64>()
            .map_err(|_| lex_codes::invalid_number(start))?;

        let end = self.position();
        Ok(Token::new(
            TokenKind::Number(num),
            start,
            Span::new(start, end),
        ))
    }

    /// Lex an identifier or keyword
    fn lex_identifier(&mut self) -> Token {
        let start = self.position();
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let end = self.position();

        // Check if it's a keyword
        let kind = match value.as_str() {
            "agent" | "task" | "tool" | "resource" | "mcp" | "server" | "config" | "import"
            | "export" | "if" | "else" | "while" | "for" | "return" | "async" | "await" | "as"
            | "from" | "let" | "const" => TokenKind::Keyword(value),
            "true" => TokenKind::Boolean(true),
            "false" => TokenKind::Boolean(false),
            "null" => TokenKind::Null,
            _ => TokenKind::Identifier(value),
        };

        Token::new(kind, start, Span::new(start, end))
    }

    /// Get next token
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        // Handle comments
        if self.peek() == Some('/') {
            if self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                self.skip_line_comment();
                return self.next_token();
            } else if self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                self.skip_block_comment()?;
                return self.next_token();
            }
        }

        let start = self.position();

        if self.is_at_end() {
            return Ok(Token::new(TokenKind::Eof, start, Span::new(start, start)));
        }

        let ch = self.peek().unwrap();

        // String literals
        if ch == '"' {
            return self.lex_string();
        }

        // Number literals (ASCII, Arabic-Indic, or Japanese numerals)
        if ch.is_ascii_digit() || is_arabic_digit(ch) || is_japanese_numeral(ch) {
            return self.lex_number();
        }

        // Identifiers and keywords (ASCII fast-path in is_ident_start).
        if is_ident_start(ch) {
            return Ok(self.lex_identifier());
        }

        // Operators and delimiters
        self.advance();
        let _end = self.position();

        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::AmpAmp
                } else {
                    return Err(lex_codes::unexpected_char(ch, start));
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::PipePipe
                } else {
                    return Err(lex_codes::unexpected_char(ch, start));
                }
            }
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            _ => return Err(lex_codes::unexpected_char(ch, start)),
        };

        let final_end = self.position();
        Ok(Token::new(kind, start, Span::new(start, final_end)))
    }

    /// Tokenize entire source
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

// LexError == hudhudscript_errors::Error (type alias). The bridge methods
// (code, short_code, title, display_full) and From<LexError> for Error
// are no longer needed — they live on the unified Error type itself.
