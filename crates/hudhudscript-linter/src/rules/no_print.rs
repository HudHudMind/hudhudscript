//! Rule: no-print
//!
//! Warns on calls to `print()` / `println()` which are typically left over
//! from debugging and should not appear in production code. This rule is
//! **disabled by default** — enable it in `.hudlint` configuration.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;

const CODE: &str = "no-print";
const DEFAULT_SEVERITY: Severity = Severity::Warning;

/// Names considered "print" functions.
const PRINT_NAMES: &[&str] = &["print", "println", "console_log"];

/// Check whether a function call is to a print function.
pub fn check(
    callee_name: &str,
    span: Span,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    if !config.is_enabled(CODE) {
        return;
    }
    if PRINT_NAMES.contains(&callee_name) {
        diagnostics.push(LintDiagnostic::new(
            CODE,
            format!("`{callee_name}()` call found — consider removing print statements in production code"),
            config.severity(CODE, DEFAULT_SEVERITY),
            span,
        ));
    }
}
