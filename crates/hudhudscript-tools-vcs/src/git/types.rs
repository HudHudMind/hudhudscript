use serde::{Deserialize, Serialize};
use std::process::Output;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Unmerged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEntry {
    pub path: String,
    pub index_status: Option<FileStatus>,
    pub worktree_status: Option<FileStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeResult {
    Success,
    Conflict(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: i32,
}

impl GitOutput {
    pub fn from_output(output: Output) -> Self {
        let exit_code = output.status.code().unwrap_or(-1);
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            success: output.status.success(),
            exit_code,
        }
    }
}
