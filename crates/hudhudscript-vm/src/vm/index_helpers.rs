//! V0.7.138-P0: Index hot-path helpers.
//!
//! Palindrome and matrix regressions are dominated by `Index` / `IndexAssign`
//! bytecode.  v0.7.121 used `as_number_fast()`; v0.7.137 switched to a raw
//! `split_tag()` match that the compiler cannot optimize as well on the hot
//! Int-index path.  This module restores the fast Int-first path while
//! keeping the Number-index fallback intact.

use hudhudscript_bytecode::Value16;

/// Extract a signed numeric index from a value, Int first.
///
/// Int indices are by far the common case for loops (`i`, `j`, `k`, etc.).
/// We try the fast Int path first; if that fails we fall back to Number so
/// code like `arr[1.0]` continues to work.
#[inline(always)]
pub(crate) fn numeric_index_i64(v: Value16) -> Option<i64> {
    if let Some(i) = v.as_int_fast() {
        Some(i)
    } else {
        v.as_number_fast().map(|n| n as i64)
    }
}

/// Convert a signed index to `usize`, rejecting negative values.
///
/// Negative indices are invalid in HudHudScript; casting them silently to
/// `usize` would wrap to huge values and either succeed unexpectedly or raise
/// an out-of-bounds error from the wrong direction.
#[inline(always)]
pub(crate) fn index_i64_to_usize(i: i64) -> Option<usize> {
    if i < 0 {
        None
    } else {
        Some(i as usize)
    }
}
