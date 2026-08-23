//! DatabaseTool — Built-in database tool for HudHudScript agents (Issue #21)
//!
//! Provides a tool interface for executing SQL queries and listing tables.
//! Full sqlx integration is behind the `db` feature flag to keep the
//! default build lightweight.

#[cfg(feature = "db")]
mod codec;
pub mod config;
pub mod error;
#[cfg(feature = "db")]
mod metadata;
#[cfg(feature = "db")]
mod migrations;
#[cfg(feature = "db")]
mod mysql;
#[cfg(feature = "db")]
mod params;
#[cfg(feature = "db")]
mod pool;
#[cfg(feature = "db")]
mod postgres;
pub mod registry;
#[cfg(feature = "db")]
pub mod runtime;
#[cfg(feature = "db")]
mod service;
#[cfg(feature = "db")]
mod service_support;
#[cfg(feature = "db")]
mod sqlite;
pub mod tool;
#[cfg(feature = "db")]
mod transactions;
pub mod types;

pub use config::*;
pub use error::*;
pub use registry::*;
#[cfg(feature = "db")]
pub use service::*;
pub use tool::*;
pub use types::*;
