//! Layer Execution — groups of agents executed in parallel or sequential mode.
//!
//! A layer is the smallest schedulable unit. It holds agent references and
//! configuration (timeout, failure strategy, dependencies). The executor
//! dispatches agents and applies the configured failure strategy.

pub mod error;
pub mod executor;
pub mod types;

pub use error::*;
pub use executor::*;
pub use types::*;
