use hudhudscript_markdown::markdown::*;
use hudhudscript_markdown::streaming::*;
use hudhudscript_markdown::syntax::*;
use hudhudscript_markdown::theme::*;

// =========================================================================
// Block parsing — headings
// =========================================================================

#[test]
fn parse_h1() {
    let blocks = parse_blocks("# Hello");
    assert_eq!(blocks.len(), 1);
    assert_eq!(
        blocks[0],
        Block::Heading {
            level: 1,
            content: "Hello".to_string()
        }
    );
}

#[test]
fn parse_h2() {
    let blocks = parse_blocks("## Sub Heading");
    assert_eq!(blocks.len(), 1);
    if let Block::Heading { level, content } = &blocks[0] {
        assert_eq!(*level, 2);
        assert_eq!(content, "Sub Heading");
    } else {
        panic!("expected Heading");
    }
}

#[test]
fn parse_h6() {
    let blocks = parse_blocks("###### Deep");
    assert_eq!(blocks.len(), 1);
    if let Block::Heading { level, .. } = &blocks[0] {
        assert_eq!(*level, 6);
    }
}

#[test]
fn parse_heading_no_space_not_heading() {
    let blocks = parse_blocks("#NoSpace");
    // Should be parsed as a paragraph, not a heading
    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0], Block::Paragraph(_)));
}

// =========================================================================
// Block parsing — paragraphs
// =========================================================================

#[test]
fn parse_paragraph() {
    let blocks = parse_blocks("This is a paragraph.\nWith two lines.");
    assert_eq!(blocks.len(), 1);
    if let Block::Paragraph(text) = &blocks[0] {
        assert!(text.contains("This is a paragraph."));
        assert!(text.contains("With two lines."));
    } else {
        panic!("expected Paragraph");
    }
}

#[test]
fn parse_empty_input() {
    let blocks = parse_blocks("");
    assert!(blocks.is_empty());
}

#[test]
fn parse_blank_lines_only() {
    let blocks = parse_blocks("\n\n\n");
    assert!(blocks.is_empty());
}

// =========================================================================
// Block parsing — code blocks
// =========================================================================

#[test]
fn parse_code_block_with_lang() {
    let input = "```rust\nfn main() {}\n```";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::CodeBlock { lang, code } = &blocks[0] {
        assert_eq!(lang.as_deref(), Some("rust"));
        assert_eq!(code, "fn main() {}");
    } else {
        panic!("expected CodeBlock");
    }
}

#[test]
fn parse_code_block_no_lang() {
    let input = "```\nhello world\n```";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::CodeBlock { lang, code } = &blocks[0] {
        assert!(lang.is_none());
        assert_eq!(code, "hello world");
    } else {
        panic!("expected CodeBlock");
    }
}

#[test]
fn parse_code_block_multiline() {
    let input = "```python\nfor i in range(10):\n    print(i)\n```";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::CodeBlock { code, .. } = &blocks[0] {
        assert!(code.contains("for i in range(10):"));
        assert!(code.contains("print(i)"));
    }
}

// =========================================================================
// Block parsing — blockquotes
// =========================================================================

#[test]
fn parse_blockquote() {
    let input = "> This is a quote\n> Second line";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::Blockquote(text) = &blocks[0] {
        assert!(text.contains("This is a quote"));
        assert!(text.contains("Second line"));
    } else {
        panic!("expected Blockquote");
    }
}

#[test]
fn parse_blockquote_single_line() {
    let blocks = parse_blocks("> Just one line");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0], Block::Blockquote(_)));
}

// =========================================================================
// Block parsing — lists
// =========================================================================

#[test]
fn parse_unordered_list_dash() {
    let input = "- Item A\n- Item B\n- Item C";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::UnorderedList(items) = &blocks[0] {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "Item A");
    } else {
        panic!("expected UnorderedList");
    }
}

#[test]
fn parse_unordered_list_asterisk() {
    let input = "* First\n* Second";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::UnorderedList(items) = &blocks[0] {
        assert_eq!(items.len(), 2);
    }
}

#[test]
fn parse_unordered_list_plus() {
    let input = "+ Alpha\n+ Beta";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(blocks[0], Block::UnorderedList(_)));
}

#[test]
fn parse_ordered_list() {
    let input = "1. First\n2. Second\n3. Third";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::OrderedList(items) = &blocks[0] {
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "First");
    } else {
        panic!("expected OrderedList");
    }
}

// =========================================================================
// Block parsing — tables
// =========================================================================

#[test]
fn parse_table() {
    let input = "| Name | Age |\n| --- | --- |\n| Alice | 30 |\n| Bob | 25 |";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::Table { headers, rows } = &blocks[0] {
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0], "Name");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "Alice");
    } else {
        panic!("expected Table");
    }
}

#[test]
fn parse_table_single_row() {
    let input = "| H1 | H2 |\n|---|---|\n| A | B |";
    let blocks = parse_blocks(input);
    if let Block::Table { rows, .. } = &blocks[0] {
        assert_eq!(rows.len(), 1);
    }
}

// =========================================================================
// Block parsing — horizontal rule
// =========================================================================

#[test]
fn parse_horizontal_rule_dashes() {
    let blocks = parse_blocks("---");
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], Block::HorizontalRule);
}

#[test]
fn parse_horizontal_rule_asterisks() {
    let blocks = parse_blocks("***");
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], Block::HorizontalRule);
}

#[test]
fn parse_horizontal_rule_underscores() {
    let blocks = parse_blocks("___");
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], Block::HorizontalRule);
}

// =========================================================================
// Block parsing — mixed content
// =========================================================================

#[test]
fn parse_mixed_blocks() {
    let input = "# Title\n\nParagraph text.\n\n- Item 1\n- Item 2\n\n---";
    let blocks = parse_blocks(input);
    assert!(blocks.len() >= 4);
    assert!(matches!(blocks[0], Block::Heading { .. }));
    assert!(matches!(blocks[1], Block::Paragraph(_)));
    assert!(matches!(blocks[2], Block::UnorderedList(_)));
    assert!(matches!(blocks[3], Block::HorizontalRule));
}

// =========================================================================
// Inline rendering
// =========================================================================

#[test]
fn render_inline_bold() {
    let theme = dark_theme();
    let result = render_inline("**bold text**", &theme);
    assert!(result.contains("bold text"));
    assert!(result.contains(BOLD));
}

#[test]
fn render_inline_italic() {
    let theme = dark_theme();
    let result = render_inline("*italic text*", &theme);
    assert!(result.contains("italic text"));
    assert!(result.contains(ITALIC));
}

#[test]
fn render_inline_code() {
    let theme = dark_theme();
    let result = render_inline("`code`", &theme);
    assert!(result.contains("code"));
    assert!(result.contains(theme.inline_code.fg));
}

#[test]
fn render_inline_link() {
    let theme = dark_theme();
    let result = render_inline("[text](https://example.com)", &theme);
    assert!(result.contains("text"));
    assert!(result.contains("https://example.com"));
    assert!(result.contains(UNDERLINE));
}

#[test]
fn render_inline_plain() {
    let theme = dark_theme();
    let result = render_inline("just plain text", &theme);
    assert_eq!(result, "just plain text");
}

// =========================================================================
// Full render
// =========================================================================

#[test]
fn render_heading() {
    let theme = dark_theme();
    let output = render("# Hello World", &theme);
    assert!(output.contains("Hello World"));
    assert!(output.contains(BOLD));
}

#[test]
fn render_code_block() {
    let theme = dark_theme();
    let input = "```rust\nlet x = 42;\n```";
    let output = render(input, &theme);
    assert!(output.contains("42"));
    assert!(output.contains("rust"));
}

#[test]
fn render_horizontal_rule_contains_line() {
    let theme = dark_theme();
    let output = render("---", &theme);
    assert!(output.contains("\u{2500}"));
}

#[test]
fn render_unordered_list_bullet() {
    let theme = dark_theme();
    let output = render("- Item A\n- Item B", &theme);
    assert!(output.contains("\u{2022}")); // bullet character
    assert!(output.contains("Item A"));
}

#[test]
fn render_ordered_list_numbers() {
    let theme = dark_theme();
    let output = render("1. First\n2. Second", &theme);
    assert!(output.contains("1."));
    assert!(output.contains("2."));
}

// =========================================================================
// Theme
// =========================================================================

#[test]
fn dark_theme_has_distinct_heading_colors() {
    let t = dark_theme();
    // h1 and h2 should have different colors
    assert_ne!(t.h1.fg, t.h2.fg);
}

#[test]
fn light_theme_has_distinct_heading_colors() {
    let t = light_theme();
    assert_ne!(t.h1.fg, t.h2.fg);
}

#[test]
fn color_new() {
    let c = Color::new("\x1b[31m");
    assert_eq!(c.fg, "\x1b[31m");
}

#[test]
fn theme_clone() {
    let t = dark_theme();
    let t2 = t.clone();
    assert_eq!(t.h1.fg, t2.h1.fg);
}

#[test]
fn ansi_constants_defined() {
    assert_eq!(RESET, "\x1b[0m");
    assert_eq!(BOLD, "\x1b[1m");
    assert_eq!(ITALIC, "\x1b[3m");
    assert_eq!(UNDERLINE, "\x1b[4m");
    assert_eq!(DIM, "\x1b[2m");
}

// =========================================================================
// Language
// =========================================================================

#[test]
fn language_from_tag_rust() {
    assert_eq!(Language::from_tag("rust"), Language::Rust);
    assert_eq!(Language::from_tag("rs"), Language::Rust);
}

#[test]
fn language_from_tag_javascript() {
    assert_eq!(Language::from_tag("javascript"), Language::JavaScript);
    assert_eq!(Language::from_tag("js"), Language::JavaScript);
}

#[test]
fn language_from_tag_python() {
    assert_eq!(Language::from_tag("python"), Language::Python);
    assert_eq!(Language::from_tag("py"), Language::Python);
}

#[test]
fn language_from_tag_hudhudscript() {
    assert_eq!(Language::from_tag("hudhudscript"), Language::HudHudScript);
    assert_eq!(Language::from_tag("hhs"), Language::HudHudScript);
}

#[test]
fn language_from_tag_unknown() {
    assert_eq!(Language::from_tag("cobol"), Language::Generic);
}

#[test]
fn language_from_tag_case_insensitive() {
    assert_eq!(Language::from_tag("RUST"), Language::Rust);
    assert_eq!(Language::from_tag("JavaScript"), Language::JavaScript);
}

// =========================================================================
// Syntax highlighting
// =========================================================================

#[test]
fn highlight_rust_keyword() {
    let theme = dark_theme();
    let result = highlight_line("fn main() {}", Language::Rust, &theme.syntax);
    assert!(result.contains("fn"));
    assert!(result.contains(theme.syntax.keyword.fg));
}

#[test]
fn highlight_string() {
    let theme = dark_theme();
    let result = highlight_line("let x = \"hello\";", Language::Rust, &theme.syntax);
    assert!(result.contains(theme.syntax.string.fg));
}

#[test]
fn highlight_comment() {
    let theme = dark_theme();
    let result = highlight_line("// comment", Language::Rust, &theme.syntax);
    assert!(result.contains(theme.syntax.comment.fg));
}

#[test]
fn highlight_number() {
    let theme = dark_theme();
    let result = highlight_line("42", Language::Rust, &theme.syntax);
    assert!(result.contains(theme.syntax.number.fg));
}

#[test]
fn highlight_block_multiline() {
    let theme = dark_theme();
    let code = "fn main() {\n    let x = 1;\n}";
    let result = highlight_block(code, Language::Rust, &theme.syntax);
    assert_eq!(result.lines().count(), 3);
}

#[test]
fn highlight_empty() {
    let theme = dark_theme();
    assert_eq!(highlight_line("", Language::Rust, &theme.syntax), "");
}

// =========================================================================
// StreamingRenderer
// =========================================================================

#[test]
fn streaming_renderer_new() {
    let renderer = StreamingRenderer::new(dark_theme());
    assert!(renderer.full_text().is_empty());
}

#[test]
fn streaming_renderer_push_paragraph() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    let output = renderer.push("Hello world\n\n");
    // After a blank line the paragraph should be renderable
    assert!(output.contains("Hello world"));
}

#[test]
fn streaming_renderer_push_incomplete_no_output() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    let output = renderer.push("Hello world");
    // No blank line yet, so nothing should be rendered
    assert!(output.is_empty());
}

#[test]
fn streaming_renderer_finish_flushes() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    renderer.push("Some text");
    let output = renderer.finish();
    assert!(output.contains("Some text"));
}

#[test]
fn streaming_renderer_code_block_buffered() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    // Push an unclosed code block
    let out1 = renderer.push("```rust\nfn main() {}");
    // Should not render yet (code block not closed)
    assert!(out1.is_empty());
    // Close the code block
    let out2 = renderer.push("\n```\n\n");
    assert!(out2.contains("main"));
}

#[test]
fn streaming_renderer_full_text_accumulates() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    renderer.push("first ");
    renderer.push("second");
    assert_eq!(renderer.full_text(), "first second");
}

#[test]
fn streaming_renderer_rerender() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    renderer.push("# Title\n\nParagraph\n\n");
    let re = renderer.rerender();
    assert!(re.contains("Title"));
    assert!(re.contains("Paragraph"));
}

#[test]
fn streaming_renderer_finish_empty_buffer() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    let output = renderer.finish();
    assert!(output.is_empty());
}

#[test]
fn streaming_renderer_finish_whitespace_only() {
    let mut renderer = StreamingRenderer::new(dark_theme());
    renderer.push("   \n  \n");
    let output = renderer.finish();
    assert!(output.is_empty());
}
