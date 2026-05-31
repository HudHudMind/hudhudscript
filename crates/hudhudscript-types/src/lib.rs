//! HudHudScript Type System
//!
//! This crate provides static type checking and type inference.

mod checker;
pub mod contracts;
pub mod inference;
pub mod semantics;
pub mod types;

pub use checker::{SymbolInfo, SymbolTable, TypeChecker};
pub use contracts::{
    ContractSignature, ContractViolation, Postcondition, Precondition, TypeConstraint,
};
pub use inference::TypeInference;
pub use semantics::{default_ownership, OwnedType, Ownership};
pub use types::*;
pub use types::error_codes as type_codes;

/// Type error — type alias for the unified [`hudhudscript_errors::Error`].
///
/// (v0.4.48 — TAM CONSOLIDATION; Anayasa Kural 1 İstisna authorized.)
/// The eleven former variants are constructed via the [`types::error_codes`] module.
/// Downstream code that used to match on enum variants now matches on
/// `error.code` against `ErrorCode::Type*`, and reads variant fields via
/// `error.context_get`.
pub type TypeError = hudhudscript_errors::Error;
