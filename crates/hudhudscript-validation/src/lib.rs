//! Input validation framework for HudHudScript
//!
//! Provides schema validation, type checking, and sanitization for user inputs.

pub mod error;
pub mod sanitizer;
pub mod schema;
pub mod validator;

pub use error::{ValidationError, ValidationResult};
pub use sanitizer::Sanitizer;
pub use schema::{Schema, SchemaType, ValidationRule};
pub use validator::Validator;
