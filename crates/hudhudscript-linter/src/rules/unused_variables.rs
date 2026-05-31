//! Rule: unused-variable
//!
//! Detects variables that are declared but never referenced.
//! This is a basic heuristic: it tracks declarations and identifier references
//! collected during the AST walk, then reports any declaration whose name
//! was never seen in a reference position.
//!
//! The actual check logic lives in `LintContext::check_unused_variables`
//! because it needs mutable access to the diagnostics vec after the walk.

use crate::walker::LintContext;
use hudhudscript_ast::Span;

/// Record a variable declaration.
pub fn record_decl(ctx: &mut LintContext, name: String, span: Span) {
    ctx.var_declarations.push((name, span));
}

/// Record a variable reference (identifier usage in expression position).
pub fn record_ref(ctx: &mut LintContext, name: &str) {
    ctx.var_references.insert(name.to_string());
}
