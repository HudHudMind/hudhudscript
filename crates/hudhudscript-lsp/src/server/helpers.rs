//! Standalone helper functions for the LSP server.

use hudhudscript_parser::parse;
use tower_lsp::lsp_types::*;

use crate::completion::{CompletionKind, CompletionProvider};

/// Convert parse errors from the HudHudScript parser into LSP `Diagnostic`s.
pub fn parse_diagnostics(source: &str) -> Vec<Diagnostic> {
    match parse(source) {
        Ok(_) => vec![],
        Err(err) => {
            // Map the error position (1-indexed) to LSP (0-indexed).
            let (line, col) = err
                .position
                .as_ref()
                .map(|p| {
                    let l = p.line.saturating_sub(1) as u32;
                    let c = p.column.saturating_sub(1) as u32;
                    (l, c)
                })
                .unwrap_or((0, 0));

            let range = Range {
                start: Position::new(line, col),
                end: Position::new(line, col + 1),
            };

            vec![Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("hudhudscript".to_string()),
                message: err.to_string(),
                ..Default::default()
            }]
        }
    }
}

/// Map our internal CompletionKind to the LSP CompletionItemKind.
pub fn completion_kind_to_lsp(kind: CompletionKind) -> CompletionItemKind {
    match kind {
        CompletionKind::Keyword => CompletionItemKind::KEYWORD,
        CompletionKind::Function => CompletionItemKind::FUNCTION,
        CompletionKind::Variable => CompletionItemKind::VARIABLE,
        CompletionKind::Module => CompletionItemKind::MODULE,
        CompletionKind::Snippet => CompletionItemKind::SNIPPET,
        CompletionKind::Field => CompletionItemKind::FIELD,
    }
}

/// Compute the byte offset into `text` for a given LSP Position (0-indexed line/col).
pub fn position_to_offset(text: &str, pos: Position) -> usize {
    let mut offset = 0usize;
    for (i, line) in text.lines().enumerate() {
        if i == pos.line as usize {
            return offset + (pos.character as usize).min(line.len());
        }
        offset += line.len() + 1; // +1 for newline
    }
    text.len()
}
