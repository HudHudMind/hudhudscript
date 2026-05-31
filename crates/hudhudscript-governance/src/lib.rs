//! # HudHudScript Governance
//!
//! Core governance structures for the Council & Constitution System.
//! This crate provides the fundamental types for constitutions, laws, councils,
//! rules, swarms, and communities.

pub mod access_control;
pub mod agent_integration;
pub mod audit;
pub mod community;
pub mod constitution;
pub mod council;
pub mod dependency;
pub mod enforcement;
pub mod error;
pub mod id_generator;
pub mod id_validator;
pub mod resources;
pub mod role;
pub mod swarm;
pub mod types;

pub use types::*;

/// Re-export chrono::Utc so dependents (VM, interpreter) can create
/// `DateTime<Utc>` values without adding a direct chrono dependency.
pub use chrono::Utc;
