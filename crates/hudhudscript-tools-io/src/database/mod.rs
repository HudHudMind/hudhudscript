//! DatabaseTool — Built-in database tool for HudHudScript agents (Issue #21)
//!
//! Provides a tool interface for executing SQL queries and listing tables.
//! Full sqlx integration is behind the `db` feature flag to keep the
//! default build lightweight.

pub mod config;
pub mod error;
pub mod registry;
pub mod tool;
pub mod types;

pub use config::*;
pub use error::*;
pub use registry::*;
pub use tool::*;
pub use types::*;
