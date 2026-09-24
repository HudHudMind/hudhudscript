//! HudHudScript Type System
//!
//! This crate provides static type checking and type inference.

pub mod checker;
pub mod contracts;
pub mod hir;
pub mod hir_array_combinators;
pub mod hir_class;
pub mod hir_closure;
pub mod hir_desugar;
pub mod hir_expr_lower;
pub mod hir_lower;
pub mod hir_ops;
#[cfg(test)]
mod hir_lower_tests;
pub mod inference;
pub mod semantics;
pub mod types;

pub use checker::{SymbolInfo, SymbolTable, TypeChecker};
pub use hir::{HirBinOp, HirExpr, HirFunction, HirModule, HirParam, HirStmt, HirUnOp};
pub use hir_lower::{lower_function as lower_function_ast, lower_module, lower_module_with_init, HirLowerError};
pub use contracts::{
    ContractSignature, ContractViolation, Postcondition, Precondition, TypeConstraint,
};
pub use inference::TypeInference;
pub use semantics::{default_ownership, OwnedType, Ownership};
pub use types::error_codes as type_codes;
pub use types::*;

/// Type error — type alias for the unified [`hudhudscript_errors::Error`].
///
/// (v0.4.48 — TAM CONSOLIDATION; Anayasa Kural 1 İstisna authorized.)
/// The eleven former variants are constructed via the [`types::error_codes`] module.
/// Downstream code that used to match on enum variants now matches on
/// `error.code` against `ErrorCode::Type*`, and reads variant fields via
/// `error.context_get`.
pub type TypeError = hudhudscript_errors::Error;
