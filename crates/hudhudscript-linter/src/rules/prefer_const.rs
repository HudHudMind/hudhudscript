//! Rule: prefer-const
//!
//! Suggests using `const` instead of `let` when a variable is never
//! reassigned after its initial declaration. This is checked in a
//! post-walk pass similar to unused-variable.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;

const CODE: &str = "prefer-const";
const DEFAULT_SEVERITY: Severity = Severity::Info;

/// Emit a diagnostic for a `let` variable that is never reassigned.
pub fn report(name: &str, span: Span, config: &LintConfig, diagnostics: &mut Vec<LintDiagnostic>) {
    if !config.is_enabled(CODE) {
        return;
    }
    diagnostics.push(LintDiagnostic::new(
        CODE,
        format!("`{name}` is never reassigned — consider using `const` instead of `let`"),
        config.severity(CODE, DEFAULT_SEVERITY),
        span,
    ));
}
