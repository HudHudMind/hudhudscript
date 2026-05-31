//! Shared archive/compression builtins — tar.gz, zip, gzip, deflate (Kural 7).
//!
//! Single source of truth for the VM and interpreter runtimes.
//! Uses system binaries (`tar`, `gzip`, `zip`, `unzip`, `python3` for deflate)
//! under the hood — same commands both runtimes have always shelled out to,
//! now factored into one module.

pub mod compress;
pub mod list;
pub mod tar;
pub mod types;
pub mod utils;
pub mod zip;

pub use types::{dispatch, ScriptMethodId};
