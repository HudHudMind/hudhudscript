//! Evaluation context for rule condition evaluation.
//!
//! Provides a type-safe wrapper around a HashMap with helper functions
//! for context construction, type validation, and field access.

pub mod access;
pub mod core;
pub mod iter;

pub use core::EvaluationContext;
