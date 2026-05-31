//! Rule: variable-shadowing
//!
//! Warns when a variable declaration shadows an existing variable from an
//! outer scope.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;
use std::collections::HashSet;

const CODE: &str = "variable-shadowing";
const DEFAULT_SEVERITY: Severity = Severity::Info;

/// Check whether `name` already exists in the scope stack and emit a diagnostic if so.
pub fn check(
    name: &str,
    span: Span,
    scope_stack: &[HashSet<String>],
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    if !config.is_enabled(CODE) {
        return;
    }
    // Walk outer scopes (skip the innermost — that is the current scope being built).
    for scope in scope_stack.iter().rev().skip(1) {
        if scope.contains(name) {
            diagnostics.push(LintDiagnostic::new(
                CODE,
                format!("variable `{name}` shadows a variable from an outer scope"),
                config.severity(CODE, DEFAULT_SEVERITY),
                span,
            ));
            return;
        }
    }
}
