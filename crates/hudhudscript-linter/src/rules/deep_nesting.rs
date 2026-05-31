//! Rule: deep-nesting
//!
//! Warns when block nesting exceeds the configured threshold (default 4).

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;

const CODE: &str = "deep-nesting";
const DEFAULT_SEVERITY: Severity = Severity::Warning;

/// Emit a diagnostic if the current depth exceeds the configured max.
pub fn check(depth: usize, span: Span, config: &LintConfig, diagnostics: &mut Vec<LintDiagnostic>) {
    if !config.is_enabled(CODE) {
        return;
    }
    if depth > config.max_nesting_depth {
        diagnostics.push(LintDiagnostic::new(
            CODE,
            format!(
                "block is nested {} levels deep (max {})",
                depth, config.max_nesting_depth
            ),
            config.severity(CODE, DEFAULT_SEVERITY),
            span,
        ));
    }
}
