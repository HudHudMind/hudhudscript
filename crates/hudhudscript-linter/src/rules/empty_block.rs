//! Rule: empty-block
//!
//! Warns on empty function, agent, or subject bodies.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;

const CODE: &str = "empty-block";
const DEFAULT_SEVERITY: Severity = Severity::Warning;

/// Check that a body is not empty.
pub fn check(
    body: &[hudhudscript_ast::Stmt],
    kind: &str,
    name: &str,
    span: Span,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    if !config.is_enabled(CODE) {
        return;
    }
    if body.is_empty() {
        diagnostics.push(LintDiagnostic::new(
            CODE,
            format!("{kind} `{name}` has an empty body"),
            config.severity(CODE, DEFAULT_SEVERITY),
            span,
        ));
    }
}
