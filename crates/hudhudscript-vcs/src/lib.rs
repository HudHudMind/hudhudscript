//! HudHudScript Version Control System
//!
//! Built-in VCS for state management and versioning

pub mod branch;
pub mod conflict;
pub mod diff;
pub mod edit;
pub mod git;
pub mod merge;
pub mod patch;
pub mod persist;
pub mod state_tree;

pub use branch::{Branch, BranchId, BranchMetadata, BranchState, StateChange};
pub use conflict::{Conflict, ConflictType};
pub use diff::{create_diff, format_colored, format_unified, DiffHunk, DiffLine, DiffResult};
pub use edit::{delete_lines, indent_lines, insert_lines, replace_lines};
pub use git::{GitRepo, LogEntry, StatusEntry};
pub use merge::{MergeResult, MergeStrategy, QuorumType};
pub use patch::{apply_fuzzy, apply_patch, parse_patch, Patch};
pub use state_tree::{StateTree, VcsError};
