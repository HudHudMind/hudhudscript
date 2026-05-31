//! Network Execution — DAG-based layer orchestration.
//!
//! A network is a directed acyclic graph of layers executed in topological order.
//!
//! ## Design notes
//!
//! *Network topology is stored on the `Network` object itself, not in a global
//! registry. The executor only resolves layer IDs through the `LayerExecutor`.*
//!
//! *Cycle detection uses a standard DFS-based approach. `topological_sort` is
//! deterministic (queue → FIFO) and uses Kahn’s algorithm.*

pub mod error;
pub mod executor;
pub mod types;

pub use error::*;
pub use executor::*;
pub use types::*;
