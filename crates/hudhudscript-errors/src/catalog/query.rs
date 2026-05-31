use crate::catalog::{ErrorCategory, ErrorCode, ERROR_GROUPS, ERROR_TABLE};

/// Static metadata for one error code.
///
/// All fields are `&'static str` (or static slice) so the entire catalog lives in
/// `.rodata` and is zero-allocation to look up.
#[derive(Debug, Clone, Copy)]
pub struct ErrorEntry {
    /// Stable code identifier.
    pub code: ErrorCode,
    /// Long, namespaced code (e.g. `"HHS_E_LEX_UNEXPECTED_CHAR"`).
    pub long_code: &'static str,
    /// Short numeric code (e.g. `"E0001"`).
    pub short_code: &'static str,
    /// Human-readable title (e.g. `"Unexpected character in source"`).
    pub title: &'static str,
    /// One-sentence description suitable for inline display.
    pub short_description: &'static str,
    /// Multi-paragraph explanation: cause, effect, how to fix.
    pub long_description: &'static str,
    /// Optional hints / recovery suggestions for the user.
    pub hints: &'static [&'static str],
    /// Optional snippet of code that triggers this issue.
    pub example_bad: Option<&'static str>,
    /// Optional snippet of corrected code.
    pub example_good: Option<&'static str>,
    /// Related codes the user may want to consult.
    pub see_also: &'static [&'static str],
    /// Version this code was introduced in.
    pub since_version: &'static str,
    /// High-level category.
    pub category: ErrorCategory,
}
