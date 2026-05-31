//! # HudHudScript IDs
//!
//! ID generation and validation for governance structures.
//! This crate provides thread-safe ID generation and regex-based validation
//! for constitutions, laws, rules, councils, swarms, and communities.

pub mod generator;
pub mod validator;

// Re-export primary types for convenience
pub use generator::IdGenerator;
pub use validator::{
    sanitize_and_validate_constitution_id, sanitize_and_validate_law_id,
    sanitize_and_validate_rule_id, sanitize_id, validate_constitution_id, validate_law_id,
    validate_rule_id,
};
