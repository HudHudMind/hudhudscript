//! Color theme definitions for terminal Markdown rendering.
//!
//! Provides dark and light terminal themes with ANSI escape code colors
//! for headers, emphasis, code blocks, and other Markdown elements.

/// ANSI color code wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// ANSI escape sequence for this color (foreground).
    pub fg: &'static str,
}

impl Color {
    /// Create a color from an ANSI foreground code.
    pub const fn new(fg: &'static str) -> Self {
        Self { fg }
    }
}

/// A terminal color theme for Markdown rendering.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Color for H1 headers.
    pub h1: Color,
    /// Color for H2 headers.
    pub h2: Color,
    /// Color for H3+ headers.
    pub h3: Color,
    /// Color for bold text.
    pub bold: Color,
    /// Color for italic text.
    pub italic: Color,
    /// Color for inline code.
    pub inline_code: Color,
    /// Color for code block border/chrome.
    pub code_block_border: Color,
    /// Color for blockquote bar and text.
    pub blockquote: Color,
    /// Color for list bullet/number.
    pub list_marker: Color,
    /// Color for link URL.
    pub link_url: Color,
    /// Color for link text.
    pub link_text: Color,
    /// Color for table borders.
    pub table_border: Color,
    /// Color for horizontal rules.
    pub horizontal_rule: Color,
    /// Syntax highlighting colors for code blocks.
    pub syntax: SyntaxColors,
}

/// Syntax highlighting color set.
#[derive(Debug, Clone)]
pub struct SyntaxColors {
    /// Keywords (fn, let, if, etc.)
    pub keyword: Color,
    /// String literals.
    pub string: Color,
    /// Numeric literals.
    pub number: Color,
    /// Comments.
    pub comment: Color,
    /// Types / type annotations.
    pub type_name: Color,
    /// Function names in calls/definitions.
    pub function: Color,
    /// Operators.
    pub operator: Color,
    /// Default/plain text in code.
    pub plain: Color,
}

// -- ANSI codes ---------------------------------------------------------

/// Reset all formatting.
pub const RESET: &str = "\x1b[0m";
/// Bold on.
pub const BOLD: &str = "\x1b[1m";
/// Italic on.
pub const ITALIC: &str = "\x1b[3m";
/// Underline on.
pub const UNDERLINE: &str = "\x1b[4m";
/// Dim/faint on.
pub const DIM: &str = "\x1b[2m";

// Foreground colors
const FG_BLACK: &str = "\x1b[30m";
const FG_RED: &str = "\x1b[31m";
const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
const FG_BLUE: &str = "\x1b[34m";
const FG_MAGENTA: &str = "\x1b[35m";
const FG_CYAN: &str = "\x1b[36m";
const FG_WHITE: &str = "\x1b[37m";
const FG_BRIGHT_BLACK: &str = "\x1b[90m";
const FG_BRIGHT_RED: &str = "\x1b[91m";
const FG_BRIGHT_GREEN: &str = "\x1b[92m";
const FG_BRIGHT_YELLOW: &str = "\x1b[93m";
const FG_BRIGHT_BLUE: &str = "\x1b[94m";
const FG_BRIGHT_MAGENTA: &str = "\x1b[95m";
const FG_BRIGHT_CYAN: &str = "\x1b[96m";

/// Return the default dark terminal theme.
pub fn dark_theme() -> Theme {
    Theme {
        h1: Color::new(FG_BRIGHT_CYAN),
        h2: Color::new(FG_BRIGHT_GREEN),
        h3: Color::new(FG_BRIGHT_YELLOW),
        bold: Color::new(FG_WHITE),
        italic: Color::new(FG_WHITE),
        inline_code: Color::new(FG_BRIGHT_YELLOW),
        code_block_border: Color::new(FG_BRIGHT_BLACK),
        blockquote: Color::new(FG_BRIGHT_BLACK),
        list_marker: Color::new(FG_BRIGHT_CYAN),
        link_url: Color::new(FG_BLUE),
        link_text: Color::new(FG_BRIGHT_CYAN),
        table_border: Color::new(FG_BRIGHT_BLACK),
        horizontal_rule: Color::new(FG_BRIGHT_BLACK),
        syntax: SyntaxColors {
            keyword: Color::new(FG_MAGENTA),
            string: Color::new(FG_GREEN),
            number: Color::new(FG_BRIGHT_YELLOW),
            comment: Color::new(FG_BRIGHT_BLACK),
            type_name: Color::new(FG_CYAN),
            function: Color::new(FG_BRIGHT_BLUE),
            operator: Color::new(FG_BRIGHT_RED),
            plain: Color::new(FG_WHITE),
        },
    }
}

/// Return the default light terminal theme.
pub fn light_theme() -> Theme {
    Theme {
        h1: Color::new(FG_BLUE),
        h2: Color::new(FG_GREEN),
        h3: Color::new(FG_YELLOW),
        bold: Color::new(FG_BLACK),
        italic: Color::new(FG_BLACK),
        inline_code: Color::new(FG_RED),
        code_block_border: Color::new(FG_BRIGHT_BLACK),
        blockquote: Color::new(FG_BRIGHT_BLACK),
        list_marker: Color::new(FG_BLUE),
        link_url: Color::new(FG_BLUE),
        link_text: Color::new(FG_CYAN),
        table_border: Color::new(FG_BRIGHT_BLACK),
        horizontal_rule: Color::new(FG_BRIGHT_BLACK),
        syntax: SyntaxColors {
            keyword: Color::new(FG_MAGENTA),
            string: Color::new(FG_GREEN),
            number: Color::new(FG_RED),
            comment: Color::new(FG_BRIGHT_BLACK),
            type_name: Color::new(FG_CYAN),
            function: Color::new(FG_BLUE),
            operator: Color::new(FG_BRIGHT_MAGENTA),
            plain: Color::new(FG_BLACK),
        },
    }
}
