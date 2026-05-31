use super::helpers::{
    is_horizontal_rule, is_ordered_list_start, is_table_separator, parse_table_row,
    strip_ordered_prefix,
};

/// Parsed Markdown block element.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    /// Heading with level (1-6) and inline content.
    Heading { level: u8, content: String },
    /// Paragraph of inline content.
    Paragraph(String),
    /// Fenced code block with optional language tag.
    CodeBlock { lang: Option<String>, code: String },
    /// Blockquote containing raw text (may have nested inlines).
    Blockquote(String),
    /// Unordered list items.
    UnorderedList(Vec<String>),
    /// Ordered list items.
    OrderedList(Vec<String>),
    /// Table with header row and body rows.
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    /// Horizontal rule.
    HorizontalRule,
}

/// Parse Markdown text into a list of block elements.
pub fn parse_blocks(input: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let len = lines.len();
    let mut i = 0;

    while i < len {
        let line = lines[i];
        let trimmed = line.trim();

        // Blank line - skip
        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Horizontal rule (---, ***, ___)
        if is_horizontal_rule(trimmed) {
            blocks.push(Block::HorizontalRule);
            i += 1;
            continue;
        }

        // Heading
        if let Some(heading) = parse_heading(trimmed) {
            blocks.push(heading);
            i += 1;
            continue;
        }

        // Fenced code block
        if trimmed.starts_with("```") {
            let lang_tag = trimmed.strip_prefix("```").unwrap().trim();
            let lang = if lang_tag.is_empty() {
                None
            } else {
                Some(lang_tag.to_string())
            };
            let mut code_lines = Vec::new();
            i += 1;
            while i < len {
                if lines[i].trim() == "```" {
                    i += 1;
                    break;
                }
                code_lines.push(lines[i]);
                i += 1;
            }
            blocks.push(Block::CodeBlock {
                lang,
                code: code_lines.join("\n"),
            });
            continue;
        }

        // Blockquote
        if trimmed.starts_with('>') {
            let mut quote_lines = Vec::new();
            while i < len && lines[i].trim().starts_with('>') {
                let content = lines[i].trim().strip_prefix('>').unwrap_or("").trim_start();
                quote_lines.push(content.to_string());
                i += 1;
            }
            blocks.push(Block::Blockquote(quote_lines.join("\n")));
            continue;
        }

        // Table (detect by looking for pipes and a separator row)
        if trimmed.contains('|') && i + 1 < len && is_table_separator(lines[i + 1].trim()) {
            let headers = parse_table_row(trimmed);
            i += 2; // skip header + separator
            let mut rows = Vec::new();
            while i < len && lines[i].trim().contains('|') {
                rows.push(parse_table_row(lines[i].trim()));
                i += 1;
            }
            blocks.push(Block::Table { headers, rows });
            continue;
        }

        // Unordered list
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let mut items = Vec::new();
            while i < len {
                let t = lines[i].trim();
                if t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ") {
                    items.push(t[2..].to_string());
                    i += 1;
                } else if t.is_empty() {
                    break;
                } else {
                    // Continuation line
                    if let Some(last) = items.last_mut() {
                        last.push(' ');
                        last.push_str(t);
                    }
                    i += 1;
                }
            }
            blocks.push(Block::UnorderedList(items));
            continue;
        }

        // Ordered list
        if is_ordered_list_start(trimmed) {
            let mut items = Vec::new();
            while i < len {
                let t = lines[i].trim();
                if let Some(rest) = strip_ordered_prefix(t) {
                    items.push(rest.to_string());
                    i += 1;
                } else if t.is_empty() {
                    break;
                } else {
                    if let Some(last) = items.last_mut() {
                        last.push(' ');
                        last.push_str(t);
                    }
                    i += 1;
                }
            }
            blocks.push(Block::OrderedList(items));
            continue;
        }

        // Default: paragraph (collect consecutive non-blank, non-special lines)
        let mut para_lines = Vec::new();
        while i < len {
            let t = lines[i].trim();
            if t.is_empty()
                || parse_heading(t).is_some()
                || t.starts_with("```")
                || t.starts_with('>')
                || is_horizontal_rule(t)
            {
                break;
            }
            para_lines.push(t);
            i += 1;
        }
        if !para_lines.is_empty() {
            blocks.push(Block::Paragraph(para_lines.join(" ")));
        }
    }

    blocks
}

/// Parse a heading line into a Block::Heading.
pub fn parse_heading(line: &str) -> Option<Block> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &line[level..];
    if !rest.starts_with(' ') && !rest.is_empty() {
        return None;
    }
    Some(Block::Heading {
        level: level as u8,
        content: rest.trim().to_string(),
    })
}
