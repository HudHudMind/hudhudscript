//! Rename provider (Issue #976)
//!
//! Implements `textDocument/rename` and `textDocument/prepareRename` by reusing
//! the existing `references::find_references` and `references::identifier_at_position`
//! helpers.  All occurrences of the identifier within the single file are replaced.

use std::collections::HashMap;

use tower_lsp::lsp_types::{
    Position as LspPosition, PrepareRenameResponse, TextEdit, Url, WorkspaceEdit,
};

use crate::references;

/// Prepare rename: validate that there is a renameable identifier at the cursor
/// and return the range + placeholder text for the client UI.
///
/// Returns `None` when the cursor is not on an identifier.
pub fn prepare_rename(source: &str, position: &LspPosition) -> Option<PrepareRenameResponse> {
    let name = references::identifier_at_position(source, position)?;

    // Compute the range of the identifier under the cursor so the client can
    // highlight it.  We walk the line to find exact start/end columns.
    let line_idx = position.line as usize;
    let col_idx = position.character as usize;
    let line = source.lines().nth(line_idx)?;

    let bytes = line.as_bytes();
    let mut start = col_idx;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = col_idx;
    while end + 1 < bytes.len() && is_ident_char(bytes[end + 1]) {
        end += 1;
    }

    let range = tower_lsp::lsp_types::Range {
        start: LspPosition::new(position.line, start as u32),
        end: LspPosition::new(position.line, (end + 1) as u32),
    };

    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range,
        placeholder: name,
    })
}

/// Perform rename: find every occurrence of the identifier at the cursor
/// position and return a `WorkspaceEdit` that replaces each with `new_name`.
///
/// Returns `None` when the cursor is not on an identifier or the source fails
/// to parse.
pub fn rename(
    source: &str,
    position: &LspPosition,
    new_name: &str,
    uri: &Url,
) -> Option<WorkspaceEdit> {
    let old_name = references::identifier_at_position(source, position)?;

    let locations = references::find_references(source, &old_name, uri);
    if locations.is_empty() {
        return None;
    }

    // Group edits by document URI (currently single-file, but future-proof).
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    for loc in locations {
        changes.entry(loc.uri).or_default().push(TextEdit {
            range: loc.range,
            new_text: new_name.to_string(),
        });
    }

    Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
