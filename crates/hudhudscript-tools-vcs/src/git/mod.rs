//! GitTool — Built-in git operations tool for HudHudScript agents (Issue #22, #621)
//!
//! Runs git subcommands via `std::process::Command`.
//! Provides both low-level `GitTool` (raw output) and high-level `GitRepo`
//! (structured types) APIs.

pub mod error;
pub mod registry;
pub mod repo;
pub mod tool;
pub mod types;

pub use error::GitError;
pub use registry::register_git_tools;
pub use repo::{parse_status_char, GitRepo};
pub use tool::GitTool;
pub use types::{BranchInfo, FileStatus, GitOutput, LogEntry, MergeResult, StatusEntry};
