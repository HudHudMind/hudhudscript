//! Loop engineering compile-time lowering (no AST evaluator, no closure execution)
//! Script loops use the compiler pipeline: Decl::Loop → FunctionChunk → VM.

use hudhudscript_ast::*;

/// Compile-time validation. No runtime execution, no native closures.
pub fn validate_loop_structure(stmts: &[Stmt]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut loop_names: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for stmt in stmts {
        if let Stmt::Decl(Decl::Loop { name, items, .. }) = stmt {
            if !loop_names.insert(name) {
                errors.push(format!("duplicate loop: '{}'", name));
                continue;
            }
            if items.is_empty() {
                errors.push(format!("loop '{}' has no items", name));
            }
        }
        if let Stmt::Decl(Decl::Chain { name, links, .. }) = stmt {
            if links.is_empty() {
                errors.push(format!("chain '{}' has no links", name));
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
