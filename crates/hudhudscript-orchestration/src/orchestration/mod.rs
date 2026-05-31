//! Orchestration Engine — coordinates workflows, networks, and layers.
//!
//! The engine provides a high-level registration and execution API. Workflows
//! contain steps that delegate to layers, networks, or councils. Each step
//! type maps to a dedicated sub-executor.

pub mod engine;
pub mod error;
pub mod types;

pub use engine::*;
pub use error::*;
pub use types::*;
