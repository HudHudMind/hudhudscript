//! Council management module.
//!
//! Provides functions for creating and managing councils,
//! which are groups of agents organized under a constitution with specific roles.

pub mod builder;
pub mod error;
pub mod manager;

pub use builder::*;
pub use error::*;
pub use manager::*;
