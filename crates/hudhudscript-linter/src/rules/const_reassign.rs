//! Rule: const-reassign
//!
//! Reports an error when a `const` variable is the target of an assignment.
//! The type-checker / runtime should also catch this, but the linter can
//! flag it earlier during static analysis.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;

const CODE: &str = "const-reassign";
const DEFAULT_SEVERITY: Severity = Severity::Error;

/// Check whether an assignment target is a known `const` name.
pub fn check(name: &str, span: Span, config: &LintConfig, diagnostics: &mut Vec<LintDiagnostic>) {
    if !config.is_enabled(CODE) {
        return;
    }
    diagnostics.push(LintDiagnostic::new(
        CODE,
        format!("cannot reassign `const` variable `{name}`"),
        config.severity(CODE, DEFAULT_SEVERITY),
        span,
    ));
}
