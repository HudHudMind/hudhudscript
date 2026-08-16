//! Canonical merge of a compiled module into the active bytecode.

use super::*;
use hudhudscript_bytecode::Bytecode;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

mod indices;

use indices::{len_u32, merge_module_payload_pools_once, remap_chunk_indices};

/// Merge selected module registries and all shared payload pools exactly once.
pub(crate) fn merge_module_bytecode(source: &Bytecode, target: &Bytecode) -> CompileResult<()> {
    let source_functions = source.function_entries_by_index()?;
    let selected_functions: Vec<_> = source_functions
        .into_iter()
        .filter(|(name, _)| !target.has_function(name))
        .collect();

    let target_actions = target.action_registry.borrow();
    let mut selected_actions: Vec<_> = source
        .action_registry
        .borrow()
        .iter()
        .filter(|(name, _)| !target_actions.contains_key(*name))
        .map(|(name, chunk)| (name.clone(), Arc::clone(chunk)))
        .collect();
    drop(target_actions);
    selected_actions.sort_by(|left, right| left.0.cmp(&right.0));

    with_target_bytecode_mut(target, |target| {
        let (bases, call_range) = merge_module_payload_pools_once(source, target)?;

        for (name, chunk) in selected_functions {
            let mut remapped = (*chunk).clone();
            remap_chunk_indices(&mut remapped, bases)?;
            target.add_function(name, Arc::new(remapped));
        }

        for (name, chunk) in selected_actions {
            let mut remapped = (*chunk).clone();
            remap_chunk_indices(&mut remapped, bases)?;
            target
                .action_registry
                .borrow_mut()
                .entry(name)
                .or_insert_with(|| Arc::new(remapped));
        }

        resolve_merged_call_range(target, call_range)
    })
}

/// The active bytecode is shared as `&Bytecode` by the instruction context.
/// Module loading is single-threaded; all RefCell borrows are released before
/// this helper; and no other mutable access exists while the closure runs.
fn with_target_bytecode_mut<T>(
    target: &Bytecode,
    operation: impl FnOnce(&mut Bytecode) -> CompileResult<T>,
) -> CompileResult<T> {
    #[allow(invalid_reference_casting)]
    let target = unsafe { &mut *(target as *const Bytecode as *mut Bytecode) };
    operation(target)
}

fn resolve_merged_call_range(target: &mut Bytecode, call_range: Range<usize>) -> CompileResult<()> {
    let entries = target.function_entries_by_index()?;
    let mut indices = HashMap::with_capacity(entries.len());
    for (index, (name, _)) in entries.iter().enumerate() {
        indices.insert(name.as_str(), len_u32("function index", index)?);
    }

    for payload_index in call_range.clone() {
        let payload = target.call_payloads.get(payload_index).ok_or_else(|| {
            merge_error(format!(
                "new call payload index {} is out of range",
                payload_index
            ))
        })?;
        let name = hudhudscript_bytecode::interner::resolve(
            hudhudscript_bytecode::interner::SymbolId(payload.sym.0),
        );
        if let Some(&function_index) = indices.get(name.as_str()) {
            target.call_payloads[payload_index].function_idx = function_index;
        }
    }

    for payload_index in call_range {
        let payload = &target.call_payloads[payload_index];
        if payload.function_idx == u32::MAX {
            continue;
        }
        let expected = hudhudscript_bytecode::interner::resolve(
            hudhudscript_bytecode::interner::SymbolId(payload.sym.0),
        );
        let actual = target.function_name_at(payload.function_idx)?;
        if actual != expected {
            return Err(merge_error(format!(
                "call payload {} resolves symbol '{}' to function '{}' at index {}",
                payload_index, expected, actual, payload.function_idx
            )));
        }
    }
    Ok(())
}

#[cold]
#[inline(never)]
fn merge_error(message: String) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(format!("Module merge invariant: {}", message))
}

#[cfg(test)]
mod tests;

pub(crate) fn collect_module_export_names(ast: &[hudhudscript_ast::Stmt]) -> Vec<String> {
    let mut names = Vec::new();
    for statement in ast {
        match statement {
            hudhudscript_ast::Stmt::Decl(declaration) => match declaration {
                hudhudscript_ast::Decl::Agent { name, .. }
                | hudhudscript_ast::Decl::Provider { name, .. }
                | hudhudscript_ast::Decl::Action { name, .. }
                | hudhudscript_ast::Decl::Tool { name, .. }
                | hudhudscript_ast::Decl::Resource { name, .. }
                | hudhudscript_ast::Decl::Subject { name, .. }
                | hudhudscript_ast::Decl::Role { name, .. }
                | hudhudscript_ast::Decl::Entity { name, .. } => names.push(name.clone()),
                _ => {}
            },
            hudhudscript_ast::Stmt::VarDecl(declaration) => names.push(declaration.name.clone()),
            hudhudscript_ast::Stmt::Let { name, .. }
            | hudhudscript_ast::Stmt::Const { name, .. }
            | hudhudscript_ast::Stmt::Function { name, .. }
            | hudhudscript_ast::Stmt::Trait { name, .. }
            | hudhudscript_ast::Stmt::EnumDecl { name, .. } => names.push(name.clone()),
            hudhudscript_ast::Stmt::Class(declaration) => names.push(declaration.name.clone()),
            hudhudscript_ast::Stmt::Export { item, .. } => {
                names.extend(collect_module_export_names(std::slice::from_ref(
                    item.as_ref(),
                )));
            }
            _ => {}
        }
    }
    names
}
