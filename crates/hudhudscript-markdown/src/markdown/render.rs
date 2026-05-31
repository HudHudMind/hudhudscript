use super::block::{parse_blocks, Block};
use crate::syntax::{self, Language};
use crate::theme::{Theme, BOLD, DIM, ITALIC, RESET, UNDERLINE};

/// Render parsed Markdown blocks to a terminal string with ANSI codes.
pub fn render(input: &str, theme: &Theme) -> String {
    let blocks = parse_blocks(input);
    render_blocks(&blocks, theme)
}

/// Render a list of blocks to a terminal string.
pub fn render_blocks(blocks: &[Block], theme: &Theme) -> String {
    let mut output = String::new();

    for (idx, block) in blocks.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }
        match block {
            Block::Heading { level, content } => {
                let color = match level {
                    1 => theme.h1.fg,
                    2 => theme.h2.fg,
                    _ => theme.h3.fg,
                };
                let prefix = "#".repeat(*level as usize);
                output.push_str(&format!(
                    "{}{} {} {}{}\n",
                    BOLD, color, prefix, content, RESET
                ));
            }
            Block::Paragraph(text) => {
                output.push_str(&render_inline(text, theme));
                output.push('\n');
            }
            Block::CodeBlock { lang, code } => {
                let lang_label = lang.as_deref().unwrap_or("");

                // Top border
                output.push_str(&format!(
                    "{}{}  {} {}\n",
                    theme.code_block_border.fg, DIM, lang_label, RESET
                ));

                // Highlighted code: only apply syntax highlighting when a
                // language is explicitly specified; otherwise emit raw text so
                // that the plain content is always a contiguous substring of
                // the output (important for tests and copy-paste).
                if let Some(lang_tag) = lang.as_deref() {
                    let language = Language::from_tag(lang_tag);
                    let highlighted = syntax::highlight_block(code, language, &theme.syntax);
                    for line in highlighted.lines() {
                        output.push_str(&format!(
                            "{}{}  {}{}  {}\n",
                            theme.code_block_border.fg, DIM, RESET, line, RESET
                        ));
                    }
                } else {
                    for line in code.lines() {
                        output.push_str(&format!(
                            "{}{}  {}{}  {}\n",
                            theme.code_block_border.fg, DIM, RESET, line, RESET
                        ));
                    }
                }

                // Bottom border
                output.push_str(&format!(
                    "{}{}  {}\n",
                    theme.code_block_border.fg, DIM, RESET
                ));
            }
            Block::Blockquote(text) => {
                for line in text.lines() {
                    output.push_str(&format!(
                        "{}{}  {} {} {}\n",
                        theme.blockquote.fg, DIM, RESET, line, RESET
                    ));
                }
            }
            Block::UnorderedList(items) => {
                for item in items {
                    output.push_str(&format!(
                        "  {}{} {}{}\n",
                        theme.list_marker.fg,
                        "\u{2022}", // bullet
                        RESET,
                        render_inline(item, theme)
                    ));
                }
            }
            Block::OrderedList(items) => {
                for (idx, item) in items.iter().enumerate() {
                    output.push_str(&format!(
                        "  {}{}. {}{}\n",
                        theme.list_marker.fg,
                        idx + 1,
                        RESET,
                        render_inline(item, theme)
                    ));
                }
            }
            Block::Table { headers, rows } => {
                render_table(&mut output, headers, rows, theme);
            }
            Block::HorizontalRule => {
                output.push_str(&format!(
                    "{}{}{}\n",
                    theme.horizontal_rule.fg,
                    "\u{2500}".repeat(40),
                    RESET
                ));
            }
        }
    }

    output
}

/// Render inline Markdown formatting (bold, italic, code, links).
pub fn render_inline(text: &str, theme: &Theme) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Inline code: `...`
        if chars[i] == '`' {
            i += 1;
            let start = i;
            while i < len && chars[i] != '`' {
                i += 1;
            }
            let code_text: String = chars[start..i].iter().collect();
            result.push_str(&format!("{}{}{}", theme.inline_code.fg, code_text, RESET));
            if i < len {
                i += 1; // skip closing `
            }
            continue;
        }

        // Bold: **...**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            i += 2;
            let start = i;
            while i < len && !(i + 1 < len && chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }
            let bold_text: String = chars[start..i].iter().collect();
            result.push_str(&format!("{}{}{}{}", BOLD, theme.bold.fg, bold_text, RESET));
            if i + 1 < len {
                i += 2; // skip closing **
            }
            continue;
        }

        // Italic: *...*
        if chars[i] == '*' && (i + 1 < len && chars[i + 1] != '*') {
            i += 1;
            let start = i;
            while i < len && chars[i] != '*' {
                i += 1;
            }
            let italic_text: String = chars[start..i].iter().collect();
            result.push_str(&format!(
                "{}{}{}{}",
                ITALIC, theme.italic.fg, italic_text, RESET
            ));
            if i < len {
                i += 1; // skip closing *
            }
            continue;
        }

        // Link: [text](url)
        if chars[i] == '[' {
            let bracket_start = i + 1;
            i += 1;
            while i < len && chars[i] != ']' {
                i += 1;
            }
            if i + 1 < len && chars[i] == ']' && chars[i + 1] == '(' {
                let link_text: String = chars[bracket_start..i].iter().collect();
                i += 2; // skip ](
                let url_start = i;
                while i < len && chars[i] != ')' {
                    i += 1;
                }
                let url: String = chars[url_start..i].iter().collect();
                result.push_str(&format!(
                    "{}{}{}{}({}{}{}{}){}",
                    UNDERLINE,
                    theme.link_text.fg,
                    link_text,
                    RESET,
                    DIM,
                    theme.link_url.fg,
                    url,
                    RESET,
                    RESET
                ));
                if i < len {
                    i += 1; // skip closing )
                }
                continue;
            } else {
                // Not a valid link, output the bracket
                result.push('[');
                result.push_str(&chars[bracket_start..i.min(len)].iter().collect::<String>());
                continue;
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

fn render_table(output: &mut String, headers: &[String], rows: &[Vec<String>], theme: &Theme) {
    // Calculate column widths
    let ncols = headers.len();
    let mut widths = vec![0usize; ncols];
    for (j, h) in headers.iter().enumerate() {
        widths[j] = widths[j].max(h.len());
    }
    for row in rows {
        for (j, cell) in row.iter().enumerate() {
            if j < ncols {
                widths[j] = widths[j].max(cell.len());
            }
        }
    }

    let border_color = theme.table_border.fg;

    // Header row
    output.push_str(border_color);
    output.push('\u{2502}');
    for (j, h) in headers.iter().enumerate() {
        let w = widths.get(j).copied().unwrap_or(0);
        output.push_str(&format!(" {}{:<w$}{} {}", BOLD, h, RESET, border_color));
        output.push('\u{2502}');
    }
    output.push_str(RESET);
    output.push('\n');

    // Separator
    output.push_str(border_color);
    output.push('\u{251c}');
    for (j, _) in headers.iter().enumerate() {
        let w = widths.get(j).copied().unwrap_or(0);
        output.push_str(&"\u{2500}".repeat(w + 2));
        if j < ncols - 1 {
            output.push('\u{253c}');
        }
    }
    output.push('\u{2524}');
    output.push_str(RESET);
    output.push('\n');

    // Body rows
    for row in rows {
        output.push_str(border_color);
        output.push('\u{2502}');
        for j in 0..ncols {
            let w = widths.get(j).copied().unwrap_or(0);
            let cell = row.get(j).map(|s| s.as_str()).unwrap_or("");
            output.push_str(&format!(" {}{:<w$}{} {}", RESET, cell, border_color, ""));
            output.push('\u{2502}');
        }
        output.push_str(RESET);
        output.push('\n');
    }
}
