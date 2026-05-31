//! Rule: empty-catch
//!
//! Warns when a `catch` block has an empty body. Swallowing errors silently
//! is almost always a mistake.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;

const CODE: &str = "empty-catch";
const DEFAULT_SEVERITY: Severity = Severity::Warning;

/// Check that a catch clause body is not empty.
///
/// `body_stmts` should be the statements inside the catch block.
pub fn check(
    body_empty: bool,
    param: &str,
    span: Span,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    if !config.is_enabled(CODE) {
        return;
    }
    if body_empty {
        diagnostics.push(LintDiagnostic::new(
            CODE,
            format!("catch block for `{param}` is empty — errors should not be silently swallowed"),
            config.severity(CODE, DEFAULT_SEVERITY),
            span,
        ));
    }
}
