//! HudHudScript CLI library
//!
//! Re-exports the CLI framework modules for use by .hud scripts and tooling.

pub mod argparse;
pub mod common;
pub mod repl;

// HOST-7: re-export host access config type for external integration tests.
pub use common::HostAccessConfig;
