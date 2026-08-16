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
                errors.push(format!("chain '{}' must have at least one link", name));
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// A1: Tek canonical semantic validation pipeline.
///
/// Structural checks (duplicate, empty) run before compilation.
/// Cross-reference validation (unknown targets, missing links) is handled
/// by the compiler during compilation, when all symbols are registered.
pub fn validate_loop_semantics(stmts: &[Stmt]) -> Result<(), Vec<String>> {
    let mut all_errors = Vec::new();

    if let Err(mut struct_errors) = validate_loop_structure(stmts) {
        all_errors.append(&mut struct_errors);
    }

    // Collect symbols for duplicate detection only — cross-reference
    // validation is deferred to the compiler which has full context.
    let symbols = super::loop_symbols::collect_loop_symbols(stmts);
    if !symbols.errors.is_empty() {
        all_errors.extend(symbols.errors.clone());
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        // Remove duplicates and keep order
        let mut unique_errors = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for err in all_errors {
            if seen.insert(err.clone()) {
                unique_errors.push(err);
            }
        }
        Err(unique_errors)
    }
}
