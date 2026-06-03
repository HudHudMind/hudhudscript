//! Unified error catalog and types for HudHudScript.
//!
//! This crate is the **single source of truth** for every error code in the
//! HudHudScript ecosystem. Every error — regardless of which phase or subsystem
//! raised it — is identified by an [`ErrorCode`] from the [`ERROR_TABLE`]
//! catalog, with structured metadata: long code, short code, title, short and
//! long descriptions, and category.
//!
//! ## Core types
//!
//! - [`ErrorCode`]   — stable enum identifying every distinct error in the system.
//! - [`ErrorEntry`]  — static metadata for one error code.
//! - [`ERROR_TABLE`] — the complete catalog (324 entries across 23 categories).
//! - [`Error`]       — the runtime error value: a code + message + position + context.
//! - [`SourcePosition`] — file-aware source location.
//! - [`Severity`]    — Error / Warning / Info classification (for diagnostics).
//! - [`Diagnostic`]  — structured renderable diagnostic for IDE/LSP.
//!
//! ## Error / Exception parity
//!
//! Every entry in this crate's [`ERROR_TABLE`] has a 1:1 sibling in the
//! [`hudhudscript-exception`] crate's `EXCEPTION_TABLE`. The two enums
//! ([`ErrorCode`] here, `ExceptionCode` there) use identical discriminants and
//! variant names so they can be transmuted at zero cost. Parity is enforced by
//! a compile-time test in the exception crate.
//!
//! ## Migrating phase-specific errors
//!
//! Phase crates (lexer, parser, compiler, ...) used to define their own enums
//! (`LexError`, `ParseError`, ...). They now route every variant through this
//! catalog by carrying an [`ErrorCode`] field, so the entire pipeline speaks
//! the same vocabulary.
//!
//! [`hudhudscript-exception`]: https://docs.rs/hudhudscript-exception

// Auto-generated catalog (323 codes). Edit `tools/gen_catalog.py` and regenerate.
pub mod catalog;
pub use catalog::{ErrorCategory, ErrorCode, ErrorEntry, ERROR_GROUPS, ERROR_TABLE};

pub mod embedded_translations;
pub use embedded_translations::{
    active_embedded_error_catalog, available_embedded_error_locales, embedded_error_catalog,
    localized_error_entry, EmbeddedErrorTranslation, EmbeddedLocaleCatalog, LocalizedErrorEntry,
};

// Legacy Exception system REMOVED (v0.4.x).
// Use `hudhudscript-exception::Exception` instead.

// Shared runtime types (GeneratorState, PromiseState) — Issue #900, #901.
pub mod shared_types;
pub use shared_types::{GeneratorState, PromiseState};

// Runtime trait — common interface for Interpreter and VM — Issue #912, #910.
pub mod runtime_trait;
pub use runtime_trait::Runtime;

// Shared scope management trait — Issue #895, Phase 1.
pub mod scope_trait;
pub use scope_trait::{ScopeManager, SymbolInfo};

// Unified module resolution trait — Issue #921, Phase 1.
pub mod module_trait;
pub use module_trait::{ModuleContent, ModuleResolver};

// Refactored modules (≤400 lines each).
pub mod constants;
mod constructors;
mod diagnostics;
mod error_value;
mod extensions;
mod position;
mod top_level;

pub use constants::{MAILBOX_CAPACITY, MAX_CALL_DEPTH, MAX_STACK_SIZE};
pub use constructors::*;
pub use diagnostics::{Diagnostic, Severity};
pub use error_value::{render_with_source, render_with_source_in_locale, Error, ErrorPayload};
pub use extensions::*;
pub use position::SourcePosition;
pub use top_level::HudHudError;

/// Canonical result alias used across the entire HudHudScript pipeline.
pub type HudHudResult<T> = std::result::Result<T, Error>;
