//! Shared GPU (nvidia-smi / rocm-smi / lspci) wrapper (Kural 7).

pub mod control;
pub mod query;
pub mod types;
pub mod utils;

pub use types::{dispatch, ScriptMethodId};
