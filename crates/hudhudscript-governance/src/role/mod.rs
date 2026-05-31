//! Agent role management module.
//!
//! Provides functions for managing agent roles within councils,
//! including predefined roles (Prosecutor, Judge, Executor, Member) and
//! custom roles with user-defined names.

pub mod error;
pub mod manager;
pub mod parse;
pub mod permissions;
pub mod query;

pub use error::*;
pub use manager::RoleManager;
