//! Git tool — re-exported from `hudhudscript-tools-vcs`
//!
//! The original `tools/git.rs` was a 440-line duplicate of code that also
//! lived in `tools-vcs/git.rs`. To eliminate the duplication, this module
//! is now a thin re-export. The single source of truth is `tools-vcs::git`.

pub use hudhudscript_tools_vcs::git::{register_git_tools, GitError, GitOutput, GitTool};
