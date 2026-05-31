//! Rule: missing-return
//!
//! Warns when a function has inconsistent return paths — i.e. some branches
//! contain an explicit `return <value>` while other branches fall through
//! without returning a value.
//!
//! This is a conservative, syntax-only check. It does not perform data-flow
//! analysis; it simply looks for the presence of `Return { value: Some(_) }`
//! anywhere in the body and, if found, checks whether the last statement is
//! also a return.

use crate::{LintConfig, LintDiagnostic, Severity};
use hudhudscript_ast::{Span, Stmt};

const CODE: &str = "missing-return";
const DEFAULT_SEVERITY: Severity = Severity::Warning;

/// Returns `true` if *any* statement in the tree is `Return { value: Some(_) }`.
fn has_value_return(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Return { value: Some(_), .. } => return true,
            Stmt::Block { statements, .. } => {
                if has_value_return(statements) {
                    return true;
                }
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                if has_value_return(std::slice::from_ref(then_branch.as_ref())) {
                    return true;
                }
                if let Some(eb) = else_branch {
                    if has_value_return(std::slice::from_ref(eb.as_ref())) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Returns `true` if the statement list *always* ends with a return (or every
/// branch of a trailing if/else does).
fn ends_with_return(stmts: &[Stmt]) -> bool {
    match stmts.last() {
        Some(Stmt::Return { .. }) => true,
        Some(Stmt::Block { statements, .. }) => ends_with_return(statements),
        Some(Stmt::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        }) => {
            ends_with_return(std::slice::from_ref(then_branch.as_ref()))
                && ends_with_return(std::slice::from_ref(else_branch.as_ref()))
        }
        _ => false,
    }
}

/// Check a function body for inconsistent returns.
pub fn check(
    body: &[Stmt],
    name: &str,
    span: Span,
    config: &LintConfig,
    diagnostics: &mut Vec<LintDiagnostic>,
) {
    if !config.is_enabled(CODE) {
        return;
    }
    // Only warn if the function contains at least one value-returning return
    // but does *not* end with a return on all paths.
    if has_value_return(body) && !ends_with_return(body) {
        diagnostics.push(LintDiagnostic::new(
            CODE,
            format!("function `{name}` has inconsistent return paths"),
            config.severity(CODE, DEFAULT_SEVERITY),
            span,
        ));
    }
}
