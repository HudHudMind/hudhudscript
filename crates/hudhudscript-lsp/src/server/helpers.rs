//! Standalone helper functions for the LSP server.

use hudhudscript_parser::parse;
use tower_lsp::lsp_types::*;

use crate::completion::{CompletionItem as InternalCompletionItem, CompletionKind};

/// L3: Tek panic-isolation yardımcısı — Kural 7 (tek noktada catch_unwind).
/// Panik olursa stderr'e yazılır, None döner; process YAŞAR.
pub fn isolate<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> Option<T> {
    std::panic::catch_unwind(f)
        .map_err(|_| eprintln!("[hudhudscript-lsp] PANIC yakalandı ve isolate edildi"))
        .ok()
}

/// Convert parse errors from the HudHudScript parser into LSP `Diagnostic`s.
/// FIX-1: parse() is wrapped in isolate() to catch panics on invalid/incomplete
/// source — server stays alive even if the parser crashes.
pub fn parse_diagnostics(source: &str) -> Vec<Diagnostic> {
    match isolate(|| parse(source)) {
        Some(Ok(_)) => vec![],
        Some(Err(err)) => {
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
        None => {
            // Parser panicked — return a single diagnostic so the user knows.
            vec![Diagnostic {
                range: Range::new(Position::new(0, 0), Position::new(0, 1)),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("hudhudscript".to_string()),
                message: "Internal parser error (source may be incomplete/invalid)".to_string(),
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

/// Convert an internal completion item into an LSP `CompletionItem`.
pub fn completion_item_to_lsp(item: InternalCompletionItem) -> CompletionItem {
    CompletionItem {
        label: item.label,
        kind: Some(completion_kind_to_lsp(item.kind)),
        detail: item.detail,
        insert_text: item.insert_text.clone(),
        insert_text_format: item.insert_text.as_ref().map(|_| InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

/// Compute the byte offset into `text` for a given LSP Position (0-indexed line/col).
/// FIX-2: `pos.character` is a UTF-16 code-unit index (LSP spec). We convert it
/// to a byte offset using `char_indices`, ensuring the result is always a valid
/// char boundary — preventing panics on multi-byte sources (Turkish/Arabic/etc.).
pub fn position_to_offset(text: &str, pos: Position) -> usize {
    let mut offset = 0usize;
    for (i, line) in text.lines().enumerate() {
        if i == pos.line as usize {
            // Convert UTF-16 code-unit index to byte offset via char_indices.
            let mut char_offset = 0usize;
            for (byte_idx, ch) in line.char_indices() {
                if char_offset >= pos.character as usize {
                    return offset + byte_idx;
                }
                // Each UTF-16 code unit counts as 1; supplementary chars (4-byte
                // UTF-8) count as 2 in UTF-16.
                char_offset += ch.len_utf16();
            }
            // Position at or past end of line — clamp to line end (char boundary).
            return offset + line.len();
        }
        offset += line.len() + 1; // +1 for newline
    }
    text.len()
}
