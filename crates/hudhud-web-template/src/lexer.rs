//! Template lexer — tokenizes Jinja2-style template source.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    // {{ ... }}
    VarStart,
    VarEnd,
    // {% ... %}
    TagStart,
    TagEnd,
    // Values
    Ident(String),
    String(String),
    Number(f64),
    Dot,
    Pipe,
    Comma,
    Colon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    // Operators
    Eq,  // ==
    Neq, // !=
    Lt,  // <
    Gt,  // >
    Le,  // <=
    Ge,  // >=
    And, // &&
    Or,  // ||
    Not, // !
    // Keywords
    If,
    Elif,
    Else,
    EndIf,
    For,
    In,
    EndFor,
    Extends,
    Block,
    EndBlock,
    Include,
    Comment,
    EndComment,
    Eof,
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut pos = 0;
    let len = chars.len();
    let mut text_buf = String::new();
    let mut in_expr = false;

    fn flush_text(buf: &mut String, tokens: &mut Vec<Token>) {
        if !buf.is_empty() {
            tokens.push(Token::Text(buf.clone()));
            buf.clear();
        }
    }

    while pos < len {
        // {{ var }}
        if pos + 1 < len && chars[pos] == '{' && chars[pos + 1] == '{' {
            flush_text(&mut text_buf, &mut tokens);
            tokens.push(Token::VarStart);
            in_expr = true;
            pos += 2;
            continue;
        }
        // }}
        if pos + 1 < len && chars[pos] == '}' && chars[pos + 1] == '}' {
            flush_text(&mut text_buf, &mut tokens);
            tokens.push(Token::VarEnd);
            in_expr = false;
            pos += 2;
            continue;
        }
        // {% tag %}
        if pos + 1 < len && chars[pos] == '{' && chars[pos + 1] == '%' {
            flush_text(&mut text_buf, &mut tokens);
            tokens.push(Token::TagStart);
            in_expr = true;
            pos += 2;
            // skip whitespace after tag start
            while pos < len && chars[pos].is_whitespace() {
                pos += 1;
            }
            continue;
        }
        // %}
        if pos + 1 < len && chars[pos] == '%' && chars[pos + 1] == '}' {
            flush_text(&mut text_buf, &mut tokens);
            tokens.push(Token::TagEnd);
            in_expr = false;
            pos += 2;
            continue;
        }
        // {# comment #}
        if pos + 1 < len && chars[pos] == '{' && chars[pos + 1] == '#' {
            flush_text(&mut text_buf, &mut tokens);
            pos += 2;
            // skip until #}
            while pos + 1 < len && !(chars[pos] == '#' && chars[pos + 1] == '}') {
                pos += 1;
            }
            if pos + 1 < len {
                pos += 2; // skip #}
            }
            continue;
        }

        // Inside tag/variable expression: tokenize
        if in_expr {
            // we might be inside a tag or variable expression
            // skip whitespace
            if chars[pos].is_whitespace() {
                pos += 1;
                continue;
            }

            // String literal "..." or '...'
            if chars[pos] == '"' || chars[pos] == '\'' {
                flush_text(&mut text_buf, &mut tokens);
                let quote = chars[pos];
                pos += 1;
                let mut s = String::new();
                while pos < len && chars[pos] != quote {
                    if chars[pos] == '\\' && pos + 1 < len {
                        pos += 1;
                        match chars[pos] {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            '\\' => s.push('\\'),
                            '"' => s.push('"'),
                            '\'' => s.push('\''),
                            c => {
                                s.push('\\');
                                s.push(c);
                            }
                        }
                    } else {
                        s.push(chars[pos]);
                    }
                    pos += 1;
                }
                if pos < len {
                    pos += 1; // closing quote
                }
                tokens.push(Token::String(s));
                continue;
            }

            // Number
            if chars[pos].is_ascii_digit() {
                flush_text(&mut text_buf, &mut tokens);
                let mut num_str = String::new();
                while pos < len && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                    num_str.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(Token::Number(num_str.parse().unwrap_or(0.0)));
                continue;
            }

            // Operators
            if pos + 1 < len {
                match (chars[pos], chars[pos + 1]) {
                    ('=', '=') => {
                        flush_text(&mut text_buf, &mut tokens);
                        tokens.push(Token::Eq);
                        pos += 2;
                        continue;
                    }
                    ('!', '=') => {
                        flush_text(&mut text_buf, &mut tokens);
                        tokens.push(Token::Neq);
                        pos += 2;
                        continue;
                    }
                    ('<', '=') => {
                        flush_text(&mut text_buf, &mut tokens);
                        tokens.push(Token::Le);
                        pos += 2;
                        continue;
                    }
                    ('>', '=') => {
                        flush_text(&mut text_buf, &mut tokens);
                        tokens.push(Token::Ge);
                        pos += 2;
                        continue;
                    }
                    ('&', '&') => {
                        flush_text(&mut text_buf, &mut tokens);
                        tokens.push(Token::And);
                        pos += 2;
                        continue;
                    }
                    ('|', '|') => {
                        flush_text(&mut text_buf, &mut tokens);
                        tokens.push(Token::Or);
                        pos += 2;
                        continue;
                    }
                    _ => {}
                }
            }

            // Single char operators
            match chars[pos] {
                '<' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::Lt);
                    pos += 1;
                    continue;
                }
                '>' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::Gt);
                    pos += 1;
                    continue;
                }
                '!' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::Not);
                    pos += 1;
                    continue;
                }
                '.' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::Dot);
                    pos += 1;
                    continue;
                }
                '|' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::Pipe);
                    pos += 1;
                    continue;
                }
                ',' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::Comma);
                    pos += 1;
                    continue;
                }
                ':' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::Colon);
                    pos += 1;
                    continue;
                }
                '(' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::LParen);
                    pos += 1;
                    continue;
                }
                ')' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::RParen);
                    pos += 1;
                    continue;
                }
                '[' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::LBracket);
                    pos += 1;
                    continue;
                }
                ']' => {
                    flush_text(&mut text_buf, &mut tokens);
                    tokens.push(Token::RBracket);
                    pos += 1;
                    continue;
                }
                _ => {}
            }

            // Identifier or keyword
            if chars[pos].is_alphabetic() || chars[pos] == '_' {
                flush_text(&mut text_buf, &mut tokens);
                let mut ident = String::new();
                while pos < len && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                    ident.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(keyword_or_ident(&ident));
                continue;
            }
        }

        // Regular text character
        text_buf.push(chars[pos]);
        pos += 1;
    }

    flush_text(&mut text_buf, &mut tokens);
    tokens.push(Token::Eof);
    tokens
}

fn keyword_or_ident(s: &str) -> Token {
    match s {
        "if" => Token::If,
        "elif" => Token::Elif,
        "else" => Token::Else,
        "endif" => Token::EndIf,
        "for" => Token::For,
        "in" => Token::In,
        "endfor" => Token::EndFor,
        "extends" => Token::Extends,
        "block" => Token::Block,
        "endblock" => Token::EndBlock,
        "include" => Token::Include,
        "comment" => Token::Comment,
        "endcomment" => Token::EndComment,
        "true" => Token::Number(1.0),
        "false" => Token::Number(0.0),
        "null" => Token::String(String::new()),
        _ => Token::Ident(s.to_string()),
    }
}
