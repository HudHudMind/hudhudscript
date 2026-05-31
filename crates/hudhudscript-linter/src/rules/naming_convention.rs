//! Rule: naming-convention
//!
//! Agent, subject, class, and enum names should be PascalCase.
//! Variable names should be camelCase or snake_case.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::Span;

const CODE: &str = "naming-convention";
const DEFAULT_SEVERITY: Severity = Severity::Warning;

pub fn is_pascal_case(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    if !first.is_ascii_uppercase() {
        return false;
    }
    // Must not contain underscores (that would be SCREAMING_SNAKE)
    !s.contains('_')
}

pub fn is_camel_or_snake(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    // camelCase starts lowercase, no underscores
    if first.is_ascii_lowercase() && !s.contains('_') {
        return true;
    }
    // snake_case: all lowercase/digits/underscores
    if s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return true;
    }
    false
}

/// Check that a type-like name (agent, subject, class, enum) is PascalCase.
pub fn check_type_name(
    name: &str,
    kind: &str,
    span: Span,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    if !config.is_enabled(CODE) {
        return;
    }
    if !is_pascal_case(name) {
        diagnostics.push(LintDiagnostic::new(
            CODE,
            format!("{kind} name `{name}` should be PascalCase"),
            config.severity(CODE, DEFAULT_SEVERITY),
            span,
        ));
    }
}

/// Check that a variable name is camelCase or snake_case.
pub fn check_variable_name(
    name: &str,
    span: Span,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    if !config.is_enabled(CODE) {
        return;
    }
    // Skip single-character names and conventional names like `_`
    if name.len() <= 1 || name.starts_with('_') {
        return;
    }
    if !is_camel_or_snake(name) {
        diagnostics.push(LintDiagnostic::new(
            CODE,
            format!("variable `{name}` should be camelCase or snake_case"),
            config.severity(CODE, DEFAULT_SEVERITY),
            span,
        ));
    }
}
