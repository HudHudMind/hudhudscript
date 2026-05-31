//! HudHudError — Convenience re-export and consolidation (#821)
//!
//! This module provides a unified error type alias and conversion traits
//! that all crates in the workspace can use. Instead of each crate defining
//! its own error types, they should use `HudHudError` which wraps the
//! catalog-based `hudhudscript_errors::Error`.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use hudhudscript_utils::error::{HudHudError, HudHudResult};
//!
//! fn my_function() -> HudHudResult<String> {
//!     Ok("success".to_string())
//! }
//! ```

/// Unified error type alias for all HudHudScript operations.
///
/// This wraps a String-based error for crates that don't need the full
/// catalog-based error system. Crates that need structured errors should
/// use `hudhudscript_errors::Error` directly.
pub type HudHudError = String;

/// Unified result type alias.
pub type HudHudResult<T> = std::result::Result<T, HudHudError>;

/// Trait for converting domain-specific errors into HudHudError.
///
/// Implement this for your crate-specific error types to enable
/// seamless conversion to the unified error type.
pub trait IntoHudHudError {
    /// Convert this error into a HudHudError string.
    fn into_hudhud_error(self) -> HudHudError;
}

impl IntoHudHudError for std::io::Error {
    fn into_hudhud_error(self) -> HudHudError {
        format!("IO error: {}", self)
    }
}

impl IntoHudHudError for std::fmt::Error {
    fn into_hudhud_error(self) -> HudHudError {
        format!("Format error: {}", self)
    }
}

impl IntoHudHudError for std::num::ParseFloatError {
    fn into_hudhud_error(self) -> HudHudError {
        format!("Number parse error: {}", self)
    }
}

impl IntoHudHudError for std::num::ParseIntError {
    fn into_hudhud_error(self) -> HudHudError {
        format!("Integer parse error: {}", self)
    }
}
