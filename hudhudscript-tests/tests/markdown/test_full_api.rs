//! Real unit tests for hudhudscript-markdown — theme, syntax, streaming, markdown

use hudhudscript_markdown::markdown::block::{parse_heading, Block};
use hudhudscript_markdown::markdown::helpers::*;
use hudhudscript_markdown::markdown::render::*;
use hudhudscript_markdown::markdown::*;
use hudhudscript_markdown::streaming::*;
use hudhudscript_markdown::syntax::*;
use hudhudscript_markdown::theme::*;

// ── Theme ────────────────────────────────────────────────────────────────

#[test]
fn theme_dark_has_colors() {
    let theme = dark_theme();
    assert!(!theme.h1.fg.is_empty());
    assert!(!theme.bold.fg.is_empty());
    assert!(!theme.inline_code.fg.is_empty());
    assert!(!theme.blockquote.fg.is_empty());
}

#[test]
fn theme_light_has_colors() {
    let theme = light_theme();
    assert!(!theme.h1.fg.is_empty());
    assert!(!theme.bold.fg.is_empty());
}

#[test]
fn color_new_constructor() {
    let c = Color::new("\x1b[31m");
    assert_eq!(c.fg, "\x1b[31m");
}

#[test]
fn syntax_colors_are_ansi() {
    let theme = dark_theme();
    assert!(theme.syntax.keyword.fg.starts_with("\x1b["));
    assert!(theme.syntax.string.fg.starts_with("\x1b["));
    assert!(theme.syntax.comment.fg.starts_with("\x1b["));
}

#[test]
fn ansi_constants_are_escape_sequences() {
    assert!(RESET.contains("\x1b["));
    assert!(BOLD.contains("\x1b["));
    assert!(ITALIC.contains("\x1b["));
}

// ── Syntax highlighting ──────────────────────────────────────────────────

#[test]
fn highlight_rust_keywords() {
    let theme = dark_theme();
    let result = highlight_line("fn main() {", Language::Rust, &theme.syntax);
    assert!(result.contains("fn") || result.contains("main"));
}

#[test]
fn language_from_tag() {
    assert!(matches!(Language::from_tag("rust"), Language::Rust));
    assert!(matches!(Language::from_tag("py"), Language::Python));
    assert!(matches!(
        Language::from_tag("hudhud"),
        Language::HudHudScript
    ));
    assert!(matches!(Language::from_tag("unknown"), Language::Generic));
}

// ── Streaming ────────────────────────────────────────────────────────────

#[test]
fn detect_state_code_block_open() {
    let state = detect_state("```rust");
    assert!(matches!(state, StreamState::InCodeBlock { .. }));
}

#[test]
fn detect_state_plain_text() {
    let state = detect_state("hello world");
    assert!(matches!(state, StreamState::Normal));
}

#[test]
fn streaming_renderer_creation() {
    let theme = dark_theme();
    let renderer = StreamingRenderer::new(theme);
    let _ = renderer;
}

#[test]
fn line_offsets_iteration() {
    let text = "line1\nline2\nline3";
    let offsets = LineOffsets::new(text);
    let lines: Vec<_> = offsets.into_iter().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].1, "line1");
    assert_eq!(lines[1].1, "line2");
    assert_eq!(lines[2].1, "line3");
}

#[test]
fn line_offsets_single_line() {
    let offsets = LineOffsets::new("only one line");
    let lines: Vec<_> = offsets.into_iter().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].1, "only one line");
}

// ── Markdown parsers ─────────────────────────────────────────────────────

#[test]
fn is_horizontal_rule_valid() {
    assert!(is_horizontal_rule("---"));
    assert!(is_horizontal_rule("***"));
    assert!(is_horizontal_rule("___"));
    assert!(!is_horizontal_rule("not a rule"));
}

#[test]
fn parse_heading_level_1() {
    let block = hudhudscript_markdown::markdown::block::parse_heading("# Hello").unwrap();
    match block {
        Block::Heading { level, content } => {
            assert_eq!(level, 1);
            assert_eq!(content, "Hello");
        }
        _ => panic!("expected heading"),
    }
}

#[test]
fn parse_heading_level_3() {
    let block = hudhudscript_markdown::markdown::block::parse_heading("### Sub section").unwrap();
    match block {
        Block::Heading { level, content } => {
            assert_eq!(level, 3);
            assert_eq!(content, "Sub section");
        }
        _ => panic!("expected heading"),
    }
}

#[test]
fn parse_heading_not_a_heading() {
    assert!(hudhudscript_markdown::markdown::block::parse_heading("just text").is_none());
    assert!(hudhudscript_markdown::markdown::block::parse_heading("").is_none());
}

#[test]
fn parse_blocks_multiple() {
    let input = "# Title\n\nSome paragraph.\n\n- list item";
    let blocks = hudhudscript_markdown::markdown::block::parse_blocks(input);
    assert!(!blocks.is_empty());
}

#[test]
fn parse_table_separator() {
    assert!(is_table_separator("|---|---|"));
    assert!(is_table_separator("|:---|:---:|"));
    assert!(!is_table_separator("not a separator"));
}

#[test]
fn parse_table_row_basic() {
    let cells = parse_table_row("| A | B | C |");
    assert_eq!(cells, vec!["A", "B", "C"]);
}

#[test]
fn parse_table_row_no_pipes() {
    let cells = parse_table_row("single cell");
    assert_eq!(cells.len(), 1);
}

#[test]
fn strip_ordered_prefix_number() {
    assert_eq!(strip_ordered_prefix("1. item"), Some("item"));
    assert_eq!(strip_ordered_prefix("42. answer"), Some("answer"));
}

#[test]
fn strip_ordered_prefix_not_ordered() {
    assert_eq!(strip_ordered_prefix("- bullet"), None);
    assert_eq!(strip_ordered_prefix("plain"), None);
}

// ── Markdown render ──────────────────────────────────────────────────────

#[test]
fn render_simple_text() {
    let theme = dark_theme();
    let output = render("hello world", &theme);
    assert!(output.contains("hello world"));
}

#[test]
fn render_heading() {
    let theme = dark_theme();
    let output = render("# Title", &theme);
    assert!(output.contains("Title"));
}

#[test]
fn render_bold_inline() {
    let theme = dark_theme();
    let output = render_inline("**bold text**", &theme);
    assert!(output.contains("bold text"));
}

#[test]
fn render_code_inline() {
    let theme = dark_theme();
    let output = render_inline("`code`", &theme);
    assert!(output.contains("code"));
}

#[test]
fn render_link_inline() {
    let theme = dark_theme();
    let output = render_inline("[click](https://example.com)", &theme);
    assert!(output.contains("click"));
}

#[test]
fn render_italic_inline() {
    let theme = dark_theme();
    let output = render_inline("*italic*", &theme);
    assert!(output.contains("italic"));
}
