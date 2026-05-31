//! Helper utilities for parsing

use hudhudscript_ast::{Position, Span};
use pest::iterators::Pair;

use crate::pest_parser::Rule;

// ── O(1) line/column lookup via pre-built line-offset table ──────────
//
// The old `pair_to_span` called pest's `line_col()` on every AST node,
// which rescans from byte 0 each time → O(N) per call × M nodes = O(N²).
//
// Fix: build the line-start-offset table ONCE (O(N)) at parse entry,
// then resolve byte offset → (line, col) via binary search in O(log L)
// where L = number of source lines.  For a 1500-line file this drops
// parse time from ~8s to <200ms.

use std::cell::RefCell;

thread_local! {
    /// Pre-computed line-start byte offsets for the current parse session.
    /// Index i holds the byte offset of the first character on line (i+1).
    /// Built once per `init_line_index(source)` call at the top of `parse()`.
    static LINE_OFFSETS: RefCell<Vec<usize>> = RefCell::new(Vec::new());
}

/// Build the line-start-offset table from `source`.  Must be called once
/// at the beginning of each parse session (before any `pair_to_span`).
pub fn init_line_index(source: &str) {
    LINE_OFFSETS.with(|cell| {
        let mut offsets = cell.borrow_mut();
        offsets.clear();
        offsets.push(0); // line 1 starts at byte 0
        for (i, byte) in source.as_bytes().iter().enumerate() {
            if *byte == b'\n' {
                offsets.push(i + 1); // next line starts after the newline
            }
        }
    });
}

/// Resolve a byte offset to (line, column), both 1-indexed.
/// Uses binary search over the pre-built line-offset table → O(log L).
#[inline]
fn offset_to_line_col(offset: usize) -> (usize, usize) {
    LINE_OFFSETS.with(|cell| {
        let offsets = cell.borrow();
        if offsets.is_empty() {
            // Fallback: table not initialized (shouldn't happen in normal flow)
            return (1, offset + 1);
        }
        // Binary search: find the last line whose start offset <= offset
        let line_idx = match offsets.binary_search(&offset) {
            Ok(exact) => exact,                      // offset is exactly a line start
            Err(insert) => insert.saturating_sub(1), // offset is within this line
        };
        let line = line_idx + 1; // 1-indexed
        let col = offset - offsets[line_idx] + 1; // 1-indexed
        (line, col)
    })
}

/// Convert a Pest pair to a Span.
///
/// Uses the pre-built line-offset table for O(log L) line/column lookup
/// instead of pest's O(N) `line_col()` rescan.
pub fn pair_to_span(pair: &Pair<Rule>) -> Span {
    let span = pair.as_span();
    let start_offset = span.start();
    let end_offset = span.end();
    let (start_line, start_col) = offset_to_line_col(start_offset);
    let (end_line, end_col) = offset_to_line_col(end_offset);

    Span {
        start: Position::new(start_line, start_col, start_offset),
        end: Position::new(end_line, end_col, end_offset),
    }
}
