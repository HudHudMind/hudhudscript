//! Hover information provider (Issue #296)
//!
//! Parses the document, finds the symbol at the cursor position, and returns
//! type/documentation info as a Markdown hover card.
//!
//! Uses `AstVisitor` from `hudhudscript_ast::visitor` for AST traversal
//! (migrated in #894 Phase 2).

use tower_lsp::lsp_types::*;

pub mod builtin;
pub mod finder;

/// A resolved hover result (before conversion to LSP types).
#[derive(Debug)]
pub struct HoverInfo {
    /// Short signature / type line
    pub signature: String,
    /// Optional documentation body
    pub docs: Option<String>,
}

/// Provide hover information for the word at the given LSP position.
pub fn hover_at(source: &str, position: Position) -> Option<Hover> {
    // Identify the word under the cursor
    let word = word_at_position(source, position)?;

    // Try to find symbol information from the AST
    let info = finder::symbol_info_for(&word, source).or_else(|| builtin::builtin_info(&word))?;

    let mut md = format!("```hudhudscript\n{}\n```", info.signature);
    if let Some(docs) = &info.docs {
        md.push_str("\n\n---\n\n");
        md.push_str(docs);
    }

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: md,
        }),
        range: None,
    })
}

/// Extract the word (identifier) at the given position.
pub fn word_at_position(source: &str, pos: Position) -> Option<String> {
    let line_idx = pos.line as usize;
    let col = pos.character as usize;

    let line = source.lines().nth(line_idx)?;

    if col >= line.len() {
        return None;
    }

    // Walk backwards to find word start
    let start = line.get(..col).unwrap_or(line)
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);

    // Walk forwards to find word end
    let end = line.get(col..).unwrap_or("")
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| col + i)
        .unwrap_or(line.len());

    if start >= end {
        return None;
    }

    Some(line.get(start..end).unwrap_or("").to_string())
}
