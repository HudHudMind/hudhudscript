use hudhudscript_markdown::markdown::*;
use hudhudscript_markdown::theme::dark_theme;

#[test]
fn parse_heading_levels() {
    let blocks = parse_blocks("# H1\n## H2\n### H3");
    assert_eq!(blocks.len(), 3);
    assert!(matches!(&blocks[0], Block::Heading { level: 1, content } if content == "H1"));
    assert!(matches!(&blocks[1], Block::Heading { level: 2, content } if content == "H2"));
    assert!(matches!(&blocks[2], Block::Heading { level: 3, content } if content == "H3"));
}

#[test]
fn parse_code_block_inline() {
    let input = "```rust\nfn main() {}\n```";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(
        matches!(&blocks[0], Block::CodeBlock { lang: Some(l), code } if l == "rust" && code == "fn main() {}")
    );
}

#[test]
fn parse_unordered_list_inline() {
    let input = "- item one\n- item two\n- item three";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::UnorderedList(items) if items.len() == 3));
}

#[test]
fn parse_ordered_list_inline() {
    let input = "1. first\n2. second\n3. third";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::OrderedList(items) if items.len() == 3));
}

#[test]
fn parse_blockquote_inline() {
    let input = "> quote line 1\n> quote line 2";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::Blockquote(_)));
}

#[test]
fn parse_horizontal_rule_inline() {
    let blocks = parse_blocks("---");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::HorizontalRule));
}

#[test]
fn parse_table_inline() {
    let input = "| A | B |\n|---|---|\n| 1 | 2 |";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(
        matches!(&blocks[0], Block::Table { headers, rows } if headers.len() == 2 && rows.len() == 1)
    );
}

#[test]
fn render_inline_bold_inline() {
    let theme = dark_theme();
    let result = render_inline("this is **bold** text", &theme);
    assert!(result.contains("bold"));
    assert!(result.contains('\x1b'));
}

#[test]
fn render_inline_italic_inline() {
    let theme = dark_theme();
    let result = render_inline("this is *italic* text", &theme);
    assert!(result.contains("italic"));
}

#[test]
fn render_inline_code_inline() {
    let theme = dark_theme();
    let result = render_inline("use `code` here", &theme);
    assert!(result.contains("code"));
}

#[test]
fn render_inline_link_inline() {
    let theme = dark_theme();
    let result = render_inline("[click](https://example.com)", &theme);
    assert!(result.contains("click"));
    assert!(result.contains("example.com"));
}

#[test]
fn full_render_inline() {
    let theme = dark_theme();
    let md = "# Hello\n\nSome **bold** text.\n\n```rust\nfn main() {}\n```\n";
    let output = render(md, &theme);
    assert!(output.contains("Hello"));
    assert!(output.contains("bold"));
    assert!(output.contains("main"));
}

#[test]
fn parse_heading_level_4_5_6() {
    let blocks = parse_blocks("#### H4\n##### H5\n###### H6");
    assert_eq!(blocks.len(), 3);
    assert!(matches!(&blocks[0], Block::Heading { level: 4, content } if content == "H4"));
    assert!(matches!(&blocks[1], Block::Heading { level: 5, content } if content == "H5"));
    assert!(matches!(&blocks[2], Block::Heading { level: 6, content } if content == "H6"));
}

#[test]
fn parse_heading_no_space_is_not_heading() {
    let blocks = parse_blocks("#notaheading");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::Paragraph(_)));
}

#[test]
fn parse_horizontal_rule_variants() {
    let blocks = parse_blocks("---\n\n***\n\n___");
    let hr_count = blocks
        .iter()
        .filter(|b| matches!(b, Block::HorizontalRule))
        .count();
    assert_eq!(hr_count, 3);
}

#[test]
fn parse_horizontal_rule_with_spaces() {
    let blocks = parse_blocks("- - -");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::HorizontalRule));
}

#[test]
fn parse_code_block_no_lang_inline() {
    let input = "```\nsome code\n```";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::CodeBlock { lang: None, code } if code == "some code"));
}

#[test]
fn parse_blockquote_content() {
    let input = "> line 1\n> line 2";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::Blockquote(text) = &blocks[0] {
        assert!(text.contains("line 1"));
        assert!(text.contains("line 2"));
    } else {
        panic!("expected Blockquote");
    }
}

#[test]
fn parse_unordered_list_star_marker() {
    let input = "* item a\n* item b";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::UnorderedList(items) if items.len() == 2));
}

#[test]
fn parse_unordered_list_plus_marker() {
    let input = "+ item a\n+ item b";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::UnorderedList(items) if items.len() == 2));
}

#[test]
fn parse_unordered_list_continuation_line() {
    let input = "- item one\n  continued\n- item two";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::UnorderedList(items) = &blocks[0] {
        assert_eq!(items.len(), 2);
        assert!(items[0].contains("continued"));
    } else {
        panic!("expected UnorderedList");
    }
}

#[test]
fn parse_ordered_list_content() {
    let input = "1. first\n2. second";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::OrderedList(items) = &blocks[0] {
        assert_eq!(items[0], "first");
        assert_eq!(items[1], "second");
    } else {
        panic!("expected OrderedList");
    }
}

#[test]
fn parse_ordered_list_continuation() {
    let input = "1. first item\n   continued\n2. second";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::OrderedList(items) = &blocks[0] {
        assert_eq!(items.len(), 2);
        assert!(items[0].contains("continued"));
    } else {
        panic!("expected OrderedList");
    }
}

#[test]
fn parse_table_content() {
    let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::Table { headers, rows } = &blocks[0] {
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0], "Name");
        assert_eq!(headers[1], "Age");
        assert_eq!(rows.len(), 2);
    } else {
        panic!("expected Table");
    }
}

#[test]
fn parse_paragraph_inline() {
    let input = "This is a paragraph.\nWith two lines.";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::Paragraph(text) = &blocks[0] {
        assert!(text.contains("This is a paragraph."));
        assert!(text.contains("With two lines."));
    } else {
        panic!("expected Paragraph");
    }
}

#[test]
fn parse_blank_lines_skipped() {
    let blocks = parse_blocks("\n\n\n");
    assert!(blocks.is_empty());
}

#[test]
fn render_heading_h1_uses_h1_color() {
    let theme = dark_theme();
    let output = render("# Title", &theme);
    assert!(output.contains(theme.h1.fg));
    assert!(output.contains("Title"));
}

#[test]
fn render_heading_h2_uses_h2_color() {
    let theme = dark_theme();
    let output = render("## Title", &theme);
    assert!(output.contains(theme.h2.fg));
}

#[test]
fn render_heading_h3_uses_h3_color() {
    let theme = dark_theme();
    let output = render("### Title", &theme);
    assert!(output.contains(theme.h3.fg));
}

#[test]
fn render_blockquote_inline() {
    let theme = dark_theme();
    let output = render("> quoted text", &theme);
    assert!(output.contains("quoted text"));
    assert!(output.contains(theme.blockquote.fg));
}

#[test]
fn render_unordered_list_inline() {
    let theme = dark_theme();
    let output = render("- item A\n- item B", &theme);
    assert!(output.contains("item A"));
    assert!(output.contains("item B"));
    assert!(output.contains("\u{2022}"));
}

#[test]
fn render_ordered_list_inline() {
    let theme = dark_theme();
    let output = render("1. first\n2. second", &theme);
    assert!(output.contains("first"));
    assert!(output.contains("second"));
    assert!(output.contains("1."));
    assert!(output.contains("2."));
}

#[test]
fn render_horizontal_rule_inline() {
    let theme = dark_theme();
    let output = render("---", &theme);
    assert!(output.contains("\u{2500}"));
}

#[test]
fn render_code_block_with_lang_inline() {
    let theme = dark_theme();
    let output = render("```rust\nlet x = 1;\n```", &theme);
    assert!(output.contains("rust"));
    assert!(output.contains("x"));
}

#[test]
fn render_code_block_without_lang_inline() {
    let theme = dark_theme();
    let output = render("```\nsome code\n```", &theme);
    assert!(output.contains("some code"));
}

#[test]
fn render_table_inline() {
    let theme = dark_theme();
    let output = render("| A | B |\n|---|---|\n| 1 | 2 |", &theme);
    assert!(output.contains("A"));
    assert!(output.contains("B"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("\u{2502}"));
}

#[test]
fn render_inline_invalid_link() {
    let theme = dark_theme();
    let result = render_inline("[not a link]text", &theme);
    assert!(result.contains("["));
    assert!(result.contains("not a link"));
}

#[test]
fn render_inline_unclosed_backtick() {
    let theme = dark_theme();
    let result = render_inline("use `code here", &theme);
    assert!(result.contains("code here"));
}

#[test]
fn is_table_separator_valid() {
    assert!(is_table_separator("|---|---|"));
    assert!(is_table_separator("| --- | :---: |"));
    assert!(!is_table_separator("| text | here |"));
}

#[test]
fn strip_ordered_prefix_valid() {
    assert_eq!(strip_ordered_prefix("1. text"), Some("text"));
    assert_eq!(strip_ordered_prefix("23. more text"), Some("more text"));
}

#[test]
fn strip_ordered_prefix_invalid() {
    assert_eq!(strip_ordered_prefix("abc"), None);
    assert_eq!(strip_ordered_prefix("1x text"), None);
    assert_eq!(strip_ordered_prefix(""), None);
    assert_eq!(strip_ordered_prefix("1."), None);
}

#[test]
fn render_multiple_blocks_spacing() {
    let theme = dark_theme();
    let output = render("# H1\n\nParagraph.\n\n---", &theme);
    assert!(output.contains("H1"));
    assert!(output.contains("Paragraph"));
    assert!(output.contains("\u{2500}"));
}

#[test]
fn parse_heading_level_too_deep() {
    let blocks = parse_blocks("####### Not heading");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::Paragraph(_)));
}

#[test]
fn parse_heading_empty_content() {
    let blocks = parse_blocks("# ");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::Heading { level: 1, content } if content.is_empty()));
}

#[test]
fn is_horizontal_rule_underscores() {
    assert!(is_horizontal_rule("___"));
    assert!(is_horizontal_rule("______"));
}

#[test]
fn is_horizontal_rule_stars() {
    assert!(is_horizontal_rule("***"));
    assert!(is_horizontal_rule("* * *"));
}

#[test]
fn is_horizontal_rule_not_enough() {
    assert!(!is_horizontal_rule("--"));
    assert!(!is_horizontal_rule("**"));
}

#[test]
fn is_table_separator_with_colons() {
    assert!(is_table_separator("|:---|:---:|---:|"));
}

#[test]
fn is_table_separator_plain_text_fails() {
    assert!(!is_table_separator("this is | text"));
}

#[test]
fn strip_ordered_prefix_large_number() {
    assert_eq!(strip_ordered_prefix("100. item"), Some("item"));
}

#[test]
fn strip_ordered_prefix_no_dot() {
    assert_eq!(strip_ordered_prefix("1 item"), None);
}

#[test]
fn strip_ordered_prefix_no_space_after_dot() {
    assert_eq!(strip_ordered_prefix("1.item"), None);
}

#[test]
fn render_inline_unclosed_bold() {
    let theme = dark_theme();
    let result = render_inline("**unclosed bold", &theme);
    assert!(result.contains("unclosed bold"));
}

#[test]
fn render_inline_unclosed_italic() {
    let theme = dark_theme();
    let result = render_inline("*unclosed italic", &theme);
    assert!(result.contains("unclosed italic"));
}

#[test]
fn render_inline_nested_formatting() {
    let theme = dark_theme();
    let result = render_inline("**bold with `code`**", &theme);
    assert!(result.contains("bold"));
    assert!(result.contains("code"));
}

#[test]
fn render_inline_plain_text_inline() {
    let theme = dark_theme();
    let result = render_inline("just plain text", &theme);
    assert_eq!(result, "just plain text");
}

#[test]
fn parse_table_row_trims_cells() {
    let row = parse_table_row("| hello | world |");
    assert_eq!(row, vec!["hello", "world"]);
}

#[test]
fn parse_table_row_no_outer_pipes() {
    let row = parse_table_row("a | b | c");
    assert_eq!(row, vec!["a", "b", "c"]);
}

#[test]
fn parse_code_block_unclosed() {
    let input = "```rust\nfn main() {}";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], Block::CodeBlock { .. }));
}

#[test]
fn render_heading_h4_uses_h3_color() {
    let theme = dark_theme();
    let output = render("#### H4 Heading", &theme);
    assert!(output.contains(theme.h3.fg));
    assert!(output.contains("H4 Heading"));
}

#[test]
fn parse_table_multiple_rows() {
    let input = "| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |\n| 7 | 8 | 9 |";
    let blocks = parse_blocks(input);
    assert_eq!(blocks.len(), 1);
    if let Block::Table { headers, rows } = &blocks[0] {
        assert_eq!(headers.len(), 3);
        assert_eq!(rows.len(), 3);
    } else {
        panic!("expected Table");
    }
}

#[test]
fn parse_mixed_blocks_inline() {
    let input = "# Title\n\n- item\n\n> quote\n\n1. ordered\n\n---\n\n```\ncode\n```";
    let blocks = parse_blocks(input);
    assert!(blocks.len() >= 5);
}
