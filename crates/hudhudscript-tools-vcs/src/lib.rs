//! HudHudScript Tools — Version Control
//!
//! Git operations tool (Issue #22, #621).
#![allow(clippy::type_complexity)]

pub mod git;
pub mod git_config;

pub use git::{
    parse_status_char, register_git_tools, BranchInfo, FileStatus, GitError, GitOutput, GitRepo,
    GitTool, LogEntry, MergeResult, StatusEntry,
};
pub use git_config::GitConfig;
pub mod github;
