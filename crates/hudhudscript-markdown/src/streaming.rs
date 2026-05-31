//! Streaming / progressive Markdown renderer.
//!
//! Handles partial content that arrives chunk-by-chunk (e.g. from an LLM
//! streaming response). Tracks state such as whether we are inside a code
//! block and renders complete blocks immediately while buffering incomplete
//! ones.

use crate::markdown;
use crate::syntax::{self, Language};
use crate::theme::{Theme, DIM, RESET};

/// State of the streaming renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamState {
    /// Normal text mode.
    Normal,
    /// Inside a fenced code block with an optional language tag.
    InCodeBlock { lang: Option<String> },
}

/// A progressive Markdown renderer that accepts text incrementally.
pub struct StreamingRenderer {
    /// Accumulated raw text so far.
    buffer: String,
    /// Theme to use for rendering.
    theme: Theme,
    /// Number of characters already rendered from the buffer.
    rendered_offset: usize,
}

impl StreamingRenderer {
    /// Create a new streaming renderer with the given theme.
    pub fn new(theme: Theme) -> Self {
        Self {
            buffer: String::new(),
            theme,
            rendered_offset: 0,
        }
    }

    /// Push a new chunk of text and return the rendered output for any
    /// newly completed blocks. Partial / incomplete blocks are buffered
    /// until more data arrives or [`finish`] is called.
    pub fn push(&mut self, chunk: &str) -> String {
        self.buffer.push_str(chunk);
        self.render_available()
    }

    /// Signal that no more input will arrive. Renders any remaining
    /// buffered content (including incomplete code blocks).
    pub fn finish(&mut self) -> String {
        let remaining = &self.buffer[self.rendered_offset..];
        if remaining.trim().is_empty() {
            return String::new();
        }

        // Force-render whatever is left, even if incomplete
        let output = self.render_remaining(remaining);
        self.rendered_offset = self.buffer.len();
        output
    }

    /// Return all text accumulated so far (rendered + buffered).
    pub fn full_text(&self) -> &str {
        &self.buffer
    }

    /// Re-render the entire buffer from scratch. Useful when the terminal
    /// is resized or the user wants a clean redraw.
    pub fn rerender(&self) -> String {
        markdown::render(&self.buffer, &self.theme)
    }

    // -- internal -----------------------------------------------------------

    /// Scan the un-rendered portion of the buffer for complete blocks and
    /// render them. Returns the rendered string for those blocks.
    fn render_available(&mut self) -> String {
        let unrendered = &self.buffer[self.rendered_offset..];

        // Find the last position where we can safely cut: end of a complete
        // block. We look for the last blank-line boundary or code fence
        // closure that is NOT inside an open code fence.
        let safe_end = find_safe_cut(unrendered);
        if safe_end == 0 {
            return String::new();
        }

        let to_render = &self.buffer[self.rendered_offset..self.rendered_offset + safe_end];
        let blocks = markdown::parse_blocks(to_render);
        let output = markdown::render_blocks(&blocks, &self.theme);
        self.rendered_offset += safe_end;
        output
    }

    /// Render remaining text that may be incomplete (e.g. an unclosed
    /// code block at end-of-stream).
    fn render_remaining(&self, text: &str) -> String {
        // Detect if we are in an unclosed code block
        let state = detect_state(text);
        match state {
            StreamState::Normal => {
                let blocks = markdown::parse_blocks(text);
                markdown::render_blocks(&blocks, &self.theme)
            }
            StreamState::InCodeBlock { lang } => {
                // Render everything before the code fence normally, then
                // render the partial code block.
                let mut output = String::new();
                if let Some(fence_pos) = text.rfind("```") {
                    let before = &text[..fence_pos];
                    if !before.trim().is_empty() {
                        let blocks = markdown::parse_blocks(before);
                        output.push_str(&markdown::render_blocks(&blocks, &self.theme));
                    }

                    // Partial code block
                    let after_fence = &text[fence_pos + 3..];
                    // Skip the language tag line
                    let code_start = after_fence.find('\n').map(|p| p + 1).unwrap_or(0);
                    let code = &after_fence[code_start..];

                    let language = lang
                        .as_deref()
                        .map(Language::from_tag)
                        .unwrap_or(Language::Generic);
                    let lang_label = lang.as_deref().unwrap_or("");

                    output.push_str(&format!(
                        "{}{}  {} {}\n",
                        self.theme.code_block_border.fg, DIM, lang_label, RESET
                    ));
                    let highlighted = syntax::highlight_block(code, language, &self.theme.syntax);
                    for line in highlighted.lines() {
                        output.push_str(&format!(
                            "{}{}  {}{}  {}\n",
                            self.theme.code_block_border.fg, DIM, RESET, line, RESET
                        ));
                    }
                    // No bottom border yet (block is incomplete)
                } else {
                    let blocks = markdown::parse_blocks(text);
                    output.push_str(&markdown::render_blocks(&blocks, &self.theme));
                }
                output
            }
        }
    }
}

/// Find the byte offset in `text` up to which we can safely render
/// complete blocks. Returns 0 if nothing can be rendered yet.
pub fn find_safe_cut(text: &str) -> usize {
    // We need to ensure we don't cut in the middle of a code block.
    let mut in_code_block = false;
    let mut last_safe = 0;
    for (offset, line) in LineOffsets::new(text) {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if !in_code_block {
                // Just closed a code fence - safe to cut after this line
                last_safe = offset + line.len();
                // Account for the newline character if present
                if text[last_safe..].starts_with('\n') {
                    last_safe += 1;
                }
                continue;
            }
        }

        if !in_code_block && trimmed.is_empty() {
            // Blank line outside code block -> safe cut point
            last_safe = offset + line.len();
            if text[last_safe..].starts_with('\n') {
                last_safe += 1;
            }
        }
    }

    last_safe
}

/// Detect whether `text` ends inside an open code block.
pub fn detect_state(text: &str) -> StreamState {
    let mut in_code = false;
    let mut lang: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code {
                in_code = false;
                lang = None;
            } else {
                in_code = true;
                let tag = trimmed.strip_prefix("```").unwrap().trim();
                lang = if tag.is_empty() {
                    None
                } else {
                    Some(tag.to_string())
                };
            }
        }
    }

    if in_code {
        StreamState::InCodeBlock { lang }
    } else {
        StreamState::Normal
    }
}

/// Iterator over lines in a string, yielding `(byte_offset, line_str)`.
/// The line does NOT include the trailing newline.
/// A string that ends with `\n` produces a final empty-string entry, matching
/// the behaviour of most line-oriented tools (e.g. Python's `str.splitlines`
/// with `keepends=False` does not, but POSIX text files — and this iterator —
/// treat a trailing newline as terminating a final, possibly empty, line).
pub struct LineOffsets<'a> {
    text: &'a str,
    pos: usize,
    /// Whether we still need to emit an empty entry after a trailing newline.
    pending_empty: bool,
}

impl<'a> LineOffsets<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            pos: 0,
            pending_empty: false,
        }
    }
}

impl<'a> Iterator for LineOffsets<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pending_empty {
            self.pending_empty = false;
            return Some((self.pos, ""));
        }
        if self.pos >= self.text.len() {
            return None;
        }
        let start = self.pos;
        match self.text[start..].find('\n') {
            Some(nl) => {
                let line = &self.text[start..start + nl];
                self.pos = start + nl + 1;
                // If the newline was the very last byte, schedule an empty
                // trailing entry on the next call.
                if self.pos == self.text.len() {
                    self.pending_empty = true;
                }
                Some((start, line))
            }
            None => {
                let line = &self.text[start..];
                self.pos = self.text.len();
                Some((start, line))
            }
        }
    }
}
