//! Council Execution — strategy-based agent parallel/roundRobin execution (Issue #14)
//!
//! Takes the members list from a council, dispatches them according to execution
//! strategy, applies voting algorithm, and calls session hooks.

pub mod error;
pub mod executor;
pub mod session;
pub mod types;

pub use error::*;
pub use executor::*;
pub use session::*;
pub use types::*;
