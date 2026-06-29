//! HudHud Web Markdown — Markdown to HTML converter.
//!
//! Reuses `hudhudscript-markdown::parse_blocks` (Kural 7).
//! Converts parsed blocks to HTML.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use hudhudscript_markdown::markdown::Block;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

/// `Web.markdown(md_text)` → HTML string.
pub fn to_html(args: &[Value16]) -> HudHudResult<Value16> {
    let md = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            Error::new(
                ErrorCode::RuntimeTypeError,
                "Web.markdown: expected string argument".to_string(),
            )
        })?;
    let blocks = hudhudscript_markdown::markdown::parse_blocks(md);
    let mut html = String::new();
    for block in blocks {
        block_to_html(&block, &mut html);
    }
    Ok(Value16::string(html))
}

fn block_to_html(block: &Block, out: &mut String) {
    match block {
        Block::Heading { level, content } => {
            let tag = format!("h{}", (*level).min(6).max(1));
            out.push_str(&format!(
                "<{}>{}</{}>\n",
                tag,
                render_inline(content),
                tag
            ));
        }
        Block::Paragraph(text) => {
            out.push_str(&format!("<p>{}</p>\n", render_inline(text)));
        }
        Block::CodeBlock { lang: _, code } => {
            let escaped = html_escape(code);
            out.push_str(&format!("<pre><code>{}</code></pre>\n", escaped));
        }
        Block::Blockquote(text) => {
            out.push_str(&format!(
                "<blockquote>{}</blockquote>\n",
                render_inline(text)
            ));
        }
        Block::UnorderedList(items) => {
            out.push_str("<ul>\n");
            for item in items {
                out.push_str(&format!(
                    "<li>{}</li>\n",
                    render_inline(item)
                ));
            }
            out.push_str("</ul>\n");
        }
        Block::OrderedList(items) => {
            out.push_str("<ol>\n");
            for item in items {
                out.push_str(&format!(
                    "<li>{}</li>\n",
                    render_inline(item)
                ));
            }
            out.push_str("</ol>\n");
        }
        Block::Table { headers, rows } => {
            out.push_str("<table>\n<thead>\n<tr>\n");
            for h in headers {
                out.push_str(&format!(
                    "<th>{}</th>\n",
                    render_inline(h)
                ));
            }
            out.push_str("</tr>\n</thead>\n<tbody>\n");
            for row in rows {
                out.push_str("<tr>\n");
                for cell in row {
                    out.push_str(&format!(
                        "<td>{}</td>\n",
                        render_inline(cell)
                    ));
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</tbody>\n</table>\n");
        }
        Block::HorizontalRule => {
            out.push_str("<hr>\n");
        }
    }
}

/// Render inline markdown (bold, italic, code, links, images).
fn render_inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Bold: **text** or __text__
        if i + 1 < len && (chars[i] == '*' && chars[i + 1] == '*')
            || (chars[i] == '_' && chars[i + 1] == '_')
        {
            let marker = if chars[i] == '*' { "**" } else { "__" };
            i += 2;
            let mut content = String::new();
            while i + 1 < len {
                if (marker == "**" && chars[i] == '*' && chars[i + 1] == '*')
                    || (marker == "__" && chars[i] == '_' && chars[i + 1] == '_')
                {
                    i += 2;
                    break;
                }
                content.push(chars[i]);
                i += 1;
            }
            out.push_str(&format!("<strong>{}</strong>", render_inline(&content)));
            continue;
        }
        // Italic: *text* or _text_
        if (chars[i] == '*' || chars[i] == '_')
            && !(i + 1 < len
                && chars[i + 1] == chars[i])
        {
            let marker = chars[i];
            i += 1;
            let mut content = String::new();
            while i < len && chars[i] != marker {
                content.push(chars[i]);
                i += 1;
            }
            if i < len {
                i += 1; // skip closing marker
            }
            out.push_str(&format!("<em>{}</em>", render_inline(&content)));
            continue;
        }
        // Inline code: `text`
        if chars[i] == '`' {
            i += 1;
            let mut code = String::new();
            while i < len && chars[i] != '`' {
                code.push(chars[i]);
                i += 1;
            }
            if i < len {
                i += 1;
            }
            out.push_str(&format!("<code>{}</code>", html_escape(&code)));
            continue;
        }
        // Link: [text](url)
        if chars[i] == '[' {
            let start = i;
            i += 1;
            let mut link_text = String::new();
            while i < len && chars[i] != ']' {
                link_text.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == ']' {
                i += 1;
                if i < len && chars[i] == '(' {
                    i += 1;
                    let mut url = String::new();
                    while i < len && chars[i] != ')' {
                        url.push(chars[i]);
                        i += 1;
                    }
                    if i < len {
                        i += 1;
                    }
                    out.push_str(&format!(
                        "<a href=\"{}\">{}</a>",
                        html_escape(&url),
                        render_inline(&link_text)
                    ));
                    continue;
                }
            }
            // Not a valid link — output [
            out.push('[');
            i = start + 1;
            continue;
        }
        // Image: ![alt](url)
        if i + 1 < len && chars[i] == '!' && chars[i + 1] == '[' {
            i += 2;
            let mut alt = String::new();
            while i < len && chars[i] != ']' {
                alt.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == ']' {
                i += 1;
                if i < len && chars[i] == '(' {
                    i += 1;
                    let mut url = String::new();
                    while i < len && chars[i] != ')' {
                        url.push(chars[i]);
                        i += 1;
                    }
                    if i < len {
                        i += 1;
                    }
                    out.push_str(&format!(
                        "<img src=\"{}\" alt=\"{}\">",
                        html_escape(&url),
                        html_escape(&alt)
                    ));
                    continue;
                }
            }
            out.push_str("![");
            i -= 1; // re-process
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

