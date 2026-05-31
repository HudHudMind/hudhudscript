//! Package installer — download, verify, extract, and register packages.

pub mod core;
pub mod install;
pub mod local;
pub mod project;
pub mod utils;

pub use core::Installer;
pub use utils::InstallOptions;
