//! Validation error types for general-purpose validation.
//!
//! Note: there is a separate `ValidationError` in `hudhudscript-tools-schema`
//! that covers tool schema validation specifically (JSON Schema compliance).
//! This type covers general validation primitives (range, pattern, length).
//! They are intentionally separate — see Issue #825 / #849.

use thiserror::Error;

/// Validation error type
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Type mismatch
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// Value out of range
    #[error("Value out of range: {value} not in [{min}, {max}]")]
    OutOfRange {
        value: String,
        min: String,
        max: String,
    },

    /// Pattern mismatch
    #[error("Pattern mismatch: value does not match pattern {pattern}")]
    PatternMismatch { pattern: String },

    /// Required field missing
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    /// Invalid length
    #[error("Invalid length: expected {expected}, found {found}")]
    InvalidLength { expected: String, found: usize },

    /// Invalid format
    #[error("Invalid format: {message}")]
    InvalidFormat { message: String },

    /// Custom validation error
    #[error("Validation failed: {message}")]
    Custom { message: String },
}

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;
