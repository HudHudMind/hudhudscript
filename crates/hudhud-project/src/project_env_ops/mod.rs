//! Shared project environment detection — toolchain markers, venv, .env parsing.
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).

mod config_ops;
mod dependencies;
mod dispatch;
mod env_core;
mod helpers;
mod toolchain;

pub use dispatch::{dispatch, ScriptMethodId};
