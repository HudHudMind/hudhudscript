//! Find-references provider
//!
//! Walks the AST to collect every location where a given identifier appears,
//! including declarations, variable references, function calls, agent/task/tool/
//! resource/subject references, type references, and member accesses.

use crate::references::stmt::collect_stmt;
use hudhudscript_ast::*;
use tower_lsp::lsp_types::{Location, Position as LspPosition, Range as LspRange, Url};

/// Collect all locations where `name` appears in the given source text.
///
/// Returns an empty `Vec` when the source fails to parse.
pub fn find_references(source: &str, name: &str, uri: &Url) -> Vec<Location> {
    let stmts = match crate::server::helpers::isolate(|| hudhudscript_parser::parse(source)) {
        Some(Ok(stmts)) => stmts,
        _ => return vec![],
    };

    let mut locations = Vec::new();
    for stmt in &stmts {
        collect_stmt(stmt, name, uri, &mut locations);
    }
    locations
}

// ── Statement walker ────────────────────────────────────────────────────────

pub mod decl;
pub mod expr;
pub mod stmt;

pub(crate) fn push_if_match(
    candidate: &str,
    name: &str,
    span: Span,
    uri: &Url,
    out: &mut Vec<Location>,
) {
    if candidate == name {
        out.push(Location {
            uri: uri.clone(),
            range: span_to_range(span),
        });
    }
}

/// Convert a 1-indexed AST `Span` to a 0-indexed LSP `Range`.
pub fn span_to_range(span: Span) -> LspRange {
    LspRange {
        start: LspPosition {
            line: span.start.line.saturating_sub(1) as u32,
            character: span.start.column.saturating_sub(1) as u32,
        },
        end: LspPosition {
            line: span.end.line.saturating_sub(1) as u32,
            character: span.end.column.saturating_sub(1) as u32,
        },
    }
}

/// Extract the identifier at the given 0-indexed LSP position from source text.
///
/// Walks outward from the byte offset to find the largest contiguous run of
/// identifier characters (ASCII alphanumeric + `_`).
pub fn identifier_at_position(source: &str, position: &LspPosition) -> Option<String> {
    let line_idx = position.line as usize;
    let col_idx = position.character as usize;

    let line = source.lines().nth(line_idx)?;
    if col_idx >= line.len() {
        return None;
    }

    let bytes = line.as_bytes();
    if !is_ident_char(bytes[col_idx]) {
        return None;
    }

    let mut start = col_idx;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = col_idx;
    while end + 1 < bytes.len() && is_ident_char(bytes[end + 1]) {
        end += 1;
    }

    Some(line[start..=end].to_string())
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
