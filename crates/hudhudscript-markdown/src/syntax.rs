//! Basic syntax highlighting for code blocks.
//!
//! Provides keyword-based highlighting for common languages without
//! external dependencies. Each language defines its own keyword sets
//! and comment/string syntax.

use crate::theme::{SyntaxColors, RESET};

/// Supported languages for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    HudHudScript,
    /// Generic/unknown language - minimal highlighting.
    Generic,
}

impl Language {
    /// Detect language from a fenced code block language tag.
    pub fn from_tag(tag: &str) -> Self {
        match tag.trim().to_lowercase().as_str() {
            "rust" | "rs" => Language::Rust,
            "javascript" | "js" => Language::JavaScript,
            "typescript" | "ts" => Language::TypeScript,
            "python" | "py" => Language::Python,
            "hudhudscript" | "hudhud" | "hhs" => Language::HudHudScript,
            _ => Language::Generic,
        }
    }

    /// Return the keyword list for this language.
    pub fn keywords(&self) -> &[&str] {
        match self {
            Language::Rust => &[
                "fn", "let", "mut", "const", "if", "else", "match", "for", "while", "loop",
                "return", "break", "continue", "struct", "enum", "impl", "trait", "pub", "use",
                "mod", "crate", "self", "super", "where", "type", "as", "in", "ref", "move",
                "async", "await", "dyn", "static", "unsafe", "extern",
            ],
            Language::JavaScript | Language::TypeScript => &[
                "function",
                "const",
                "let",
                "var",
                "if",
                "else",
                "for",
                "while",
                "do",
                "return",
                "break",
                "continue",
                "class",
                "extends",
                "new",
                "this",
                "super",
                "import",
                "export",
                "from",
                "default",
                "switch",
                "case",
                "throw",
                "try",
                "catch",
                "finally",
                "async",
                "await",
                "yield",
                "typeof",
                "instanceof",
                "in",
                "of",
                "delete",
                "void",
                "null",
                "undefined",
                "true",
                "false",
            ],
            Language::Python => &[
                "def", "class", "if", "elif", "else", "for", "while", "return", "break",
                "continue", "import", "from", "as", "with", "try", "except", "finally", "raise",
                "pass", "lambda", "yield", "global", "nonlocal", "assert", "del", "in", "not",
                "and", "or", "is", "None", "True", "False", "async", "await",
            ],
            Language::HudHudScript => &[
                "fn",
                "let",
                "const",
                "if",
                "else",
                "for",
                "while",
                "return",
                "break",
                "continue",
                "class",
                "extends",
                "new",
                "this",
                "super",
                "import",
                "export",
                "from",
                "match",
                "enum",
                "struct",
                "trait",
                "impl",
                "pub",
                "async",
                "await",
                "culture",
                "law",
                "council",
                "action",
                "condition",
                "actor",
                "spawn",
                "send",
                "receive",
                "true",
                "false",
                "null",
            ],
            Language::Generic => &[
                "if", "else", "for", "while", "return", "function", "class", "import", "true",
                "false", "null",
            ],
        }
    }

    /// Return the type keyword list for this language.
    pub fn type_keywords(&self) -> &[&str] {
        match self {
            Language::Rust => &[
                "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                "usize", "f32", "f64", "bool", "char", "str", "String", "Vec", "Option", "Result",
                "Box", "Rc", "Arc", "Self",
            ],
            Language::TypeScript => &[
                "string", "number", "boolean", "any", "void", "never", "unknown", "object",
                "Array", "Promise", "Record", "Partial", "Required", "Readonly",
            ],
            Language::JavaScript => &[],
            Language::Python => &[
                "int", "float", "str", "bool", "list", "dict", "tuple", "set",
            ],
            Language::HudHudScript => &[
                "int", "float", "string", "bool", "void", "any", "Array", "Map", "Set",
            ],
            Language::Generic => &[],
        }
    }

    /// Return the single-line comment prefix(es) for this language.
    pub fn line_comment_prefix(&self) -> &[&str] {
        match self {
            Language::Rust
            | Language::JavaScript
            | Language::TypeScript
            | Language::HudHudScript => &["//"],
            Language::Python => &["#"],
            Language::Generic => &["//", "#"],
        }
    }
}

/// Highlight a single line of source code, returning a string with ANSI codes.
pub fn highlight_line(line: &str, lang: Language, colors: &SyntaxColors) -> String {
    let mut result = String::with_capacity(line.len() * 2);
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    // Check for line comment first
    for prefix in lang.line_comment_prefix() {
        if line.trim_start().starts_with(prefix) {
            let leading: String = chars.iter().take_while(|c| c.is_whitespace()).collect();
            result.push_str(&leading);
            result.push_str(colors.comment.fg);
            result.push_str(line.trim_start());
            result.push_str(RESET);
            return result;
        }
    }

    while i < len {
        let ch = chars[i];

        // String literals (double quotes)
        if ch == '"' {
            result.push_str(colors.string.fg);
            result.push('"');
            i += 1;
            while i < len {
                let c = chars[i];
                result.push(c);
                i += 1;
                if c == '\\' && i < len {
                    result.push(chars[i]);
                    i += 1;
                } else if c == '"' {
                    break;
                }
            }
            result.push_str(RESET);
            continue;
        }

        // String literals (single quotes)
        if ch == '\'' {
            result.push_str(colors.string.fg);
            result.push('\'');
            i += 1;
            while i < len {
                let c = chars[i];
                result.push(c);
                i += 1;
                if c == '\\' && i < len {
                    result.push(chars[i]);
                    i += 1;
                } else if c == '\'' {
                    break;
                }
            }
            result.push_str(RESET);
            continue;
        }

        // Numbers
        if ch.is_ascii_digit() && (i == 0 || !chars[i - 1].is_alphanumeric()) {
            result.push_str(colors.number.fg);
            while i < len
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
            {
                result.push(chars[i]);
                i += 1;
            }
            result.push_str(RESET);
            continue;
        }

        // Identifiers / keywords
        if ch.is_alphabetic() || ch == '_' {
            let start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            if lang.keywords().contains(&word.as_str()) {
                result.push_str(colors.keyword.fg);
                result.push_str(&word);
                result.push_str(RESET);
            } else if lang.type_keywords().contains(&word.as_str()) {
                result.push_str(colors.type_name.fg);
                result.push_str(&word);
                result.push_str(RESET);
            } else if i < len && chars[i] == '(' {
                // Looks like a function call
                result.push_str(colors.function.fg);
                result.push_str(&word);
                result.push_str(RESET);
            } else {
                result.push_str(colors.plain.fg);
                result.push_str(&word);
                result.push_str(RESET);
            }
            continue;
        }

        // Operators
        if "=+-*/<>!&|^%~?:".contains(ch) {
            result.push_str(colors.operator.fg);
            result.push(ch);
            result.push_str(RESET);
            i += 1;
            continue;
        }

        // Everything else (whitespace, brackets, etc.)
        result.push(ch);
        i += 1;
    }

    result
}

/// Highlight a full code block (multiple lines).
pub fn highlight_block(code: &str, lang: Language, colors: &SyntaxColors) -> String {
    let mut result = String::new();
    for (idx, line) in code.lines().enumerate() {
        if idx > 0 {
            result.push('\n');
        }
        result.push_str(&highlight_line(line, lang, colors));
    }
    result
}
