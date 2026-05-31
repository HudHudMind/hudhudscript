//! Unified exception system for HudHudScript.
//!
//! This crate provides the runtime [`Exception`] type that complements the
//! static error catalog in [`hudhudscript-errors`]. Every exception carries an
//! error catalog: same discriminant numbers, same variant names, same
//! categories. Conversion between the two enums is zero-cost.
//!
//! ## Error vs Exception
//!
//! - An **error** ([`hudhudscript_errors::Error`]) is the static description of
//!   *what can go wrong*: a stable code with a title and prose explanation in
//!   the catalog.
//! - An **exception** ([`Exception`]) is the dynamic instance *raised at
//!   runtime*: it carries the code, a formatted message, a source position,
//!   the cause chain, the call stack, and any hints. When displayed it
//!   "shows" the underlying error metadata from the catalog.
//!
//! ## Parity
//!
//! by tests in this crate. Adding a new error code without adding the matching
//! exception code (or vice-versa) breaks the build, so the two stay in sync
//! by construction.
//!
//! [`hudhudscript-errors`]: https://docs.rs/hudhudscript-errors

// Auto-generated catalog. Edit `crates/hudhudscript-errors/tools/gen_catalog.py`
// and regenerate both `catalog.rs` files together.
pub mod catalog;
pub use catalog::{ExceptionCategory, ExceptionCode, ExceptionEntry};

pub mod exception;
pub mod frame;
pub mod outcome;

pub use exception::{Exception, Result};
pub use frame::StackFrame;
pub use outcome::Outcome;
