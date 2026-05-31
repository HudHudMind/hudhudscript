use crate::git::error::GitError;
use crate::git::types::{BranchInfo, FileStatus, LogEntry, MergeResult, StatusEntry};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitRepo {
    path: PathBuf,
}

impl GitRepo {
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn discover(path: impl AsRef<Path>) -> Result<Self, GitError> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(path.as_ref())
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    GitError::GitNotFound
                } else {
                    GitError::SpawnFailed(e.to_string())
                }
            })?;
        if !output.status.success() {
            return Err(GitError::RepositoryNotFound(
                path.as_ref().display().to_string(),
            ));
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Self {
            path: PathBuf::from(root),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn run(&self, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    GitError::GitNotFound
                } else {
                    GitError::SpawnFailed(e.to_string())
                }
            })?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(GitError::CommandFailed {
                code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            })
        }
    }

    fn run_raw(&self, args: &[&str]) -> Result<(String, String, bool), GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    GitError::GitNotFound
                } else {
                    GitError::SpawnFailed(e.to_string())
                }
            })?;
        Ok((
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
            output.status.success(),
        ))
    }

    pub fn status(&self) -> Result<Vec<StatusEntry>, GitError> {
        let out = self.run(&["status", "--porcelain=v1", "-uall"])?;
        if out.is_empty() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for line in out.lines() {
            if line.len() < 4 {
                continue;
            }
            let index_char = line.as_bytes()[0];
            let wt_char = line.as_bytes()[1];
            let path = line[3..].to_string();
            entries.push(StatusEntry {
                path,
                index_status: parse_status_char(index_char),
                worktree_status: parse_status_char(wt_char),
            });
        }
        Ok(entries)
    }

    pub fn diff(&self, path: Option<&str>) -> Result<String, GitError> {
        let mut args = vec!["diff"];
        if let Some(p) = path {
            args.push("--");
            args.push(p);
        }
        self.run(&args)
    }

    pub fn diff_staged(&self) -> Result<String, GitError> {
        self.run(&["diff", "--staged"])
    }

    pub fn log(&self, count: usize) -> Result<Vec<LogEntry>, GitError> {
        let n_arg = format!("-{count}");
        let fmt = "--pretty=format:%h%x00%an%x00%ai%x00%s";
        let out = self.run(&["log", &n_arg, fmt])?;
        if out.is_empty() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for line in out.lines() {
            let parts: Vec<&str> = line.splitn(4, '\0').collect();
            if parts.len() < 4 {
                continue;
            }
            entries.push(LogEntry {
                hash: parts[0].to_string(),
                author: parts[1].to_string(),
                date: parts[2].to_string(),
                message: parts[3].to_string(),
            });
        }
        Ok(entries)
    }

    pub fn add(&self, paths: &[&str]) -> Result<(), GitError> {
        if paths.is_empty() {
            return Err(GitError::InvalidArguments(
                "at least one path required for git add".into(),
            ));
        }
        let mut args = vec!["add", "--"];
        args.extend_from_slice(paths);
        self.run(&args)?;
        Ok(())
    }

    pub fn add_all(&self) -> Result<(), GitError> {
        self.run(&["add", "-A"])?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<String, GitError> {
        if message.is_empty() {
            return Err(GitError::InvalidArguments(
                "commit message must not be empty".into(),
            ));
        }
        self.run(&["commit", "-m", message])?;
        self.run(&["rev-parse", "--short", "HEAD"])
    }

    pub fn push(&self, remote: &str, branch: &str) -> Result<(), GitError> {
        self.run(&["push", remote, branch])?;
        Ok(())
    }

    pub fn pull(&self, remote: &str, branch: &str) -> Result<(), GitError> {
        self.run(&["pull", remote, branch])?;
        Ok(())
    }

    pub fn branches(&self) -> Result<Vec<BranchInfo>, GitError> {
        let out = self.run(&[
            "branch",
            "--format=%(HEAD)%(refname:short)%00%(upstream:short)",
        ])?;
        if out.is_empty() {
            return Ok(Vec::new());
        }
        let mut branches = Vec::new();
        for line in out.lines() {
            if line.is_empty() {
                continue;
            }
            let is_current = line.starts_with('*');
            let rest = if is_current { &line[1..] } else { line };
            let parts: Vec<&str> = rest.splitn(2, '\0').collect();
            let name = parts[0].to_string();
            let upstream = parts
                .get(1)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            branches.push(BranchInfo {
                name,
                is_current,
                upstream,
            });
        }
        Ok(branches)
    }

    pub fn current_branch(&self) -> Result<String, GitError> {
        self.run(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    pub fn create_branch(&self, name: &str) -> Result<(), GitError> {
        if name.is_empty() {
            return Err(GitError::InvalidArguments(
                "branch name must not be empty".into(),
            ));
        }
        self.run(&["branch", name])?;
        Ok(())
    }

    pub fn delete_branch(&self, name: &str) -> Result<(), GitError> {
        if name.is_empty() {
            return Err(GitError::InvalidArguments(
                "branch name must not be empty".into(),
            ));
        }
        self.run(&["branch", "-d", name])?;
        Ok(())
    }

    pub fn switch_branch(&self, name: &str) -> Result<(), GitError> {
        if name.is_empty() {
            return Err(GitError::InvalidArguments(
                "branch name must not be empty".into(),
            ));
        }
        self.run(&["switch", name])?;
        Ok(())
    }

    pub fn merge(&self, branch: &str) -> Result<MergeResult, GitError> {
        if branch.is_empty() {
            return Err(GitError::InvalidArguments(
                "branch name must not be empty".into(),
            ));
        }
        let (stdout, stderr, success) = self.run_raw(&["merge", branch])?;
        if success {
            return Ok(MergeResult::Success);
        }
        if stderr.contains("CONFLICT") || stdout.contains("CONFLICT") {
            let conflicts = self.conflicted_files()?;
            return Ok(MergeResult::Conflict(conflicts));
        }
        Err(GitError::CommandFailed { code: 1, stderr })
    }

    pub fn rebase(&self, onto: &str) -> Result<(), GitError> {
        if onto.is_empty() {
            return Err(GitError::InvalidArguments(
                "rebase target must not be empty".into(),
            ));
        }
        self.run(&["rebase", onto])?;
        Ok(())
    }

    pub fn merge_abort(&self) -> Result<(), GitError> {
        self.run(&["merge", "--abort"])?;
        Ok(())
    }

    pub fn rebase_abort(&self) -> Result<(), GitError> {
        self.run(&["rebase", "--abort"])?;
        Ok(())
    }

    pub fn remotes(&self) -> Result<Vec<String>, GitError> {
        let out = self.run(&["remote"])?;
        if out.is_empty() {
            return Ok(Vec::new());
        }
        Ok(out.lines().map(|l| l.to_string()).collect())
    }

    pub fn remote_url(&self, name: &str) -> Result<String, GitError> {
        self.run(&["remote", "get-url", name])
    }

    fn conflicted_files(&self) -> Result<Vec<String>, GitError> {
        let out = self.run(&["diff", "--name-only", "--diff-filter=U"])?;
        if out.is_empty() {
            return Ok(Vec::new());
        }
        Ok(out.lines().map(|l| l.to_string()).collect())
    }
}

pub fn parse_status_char(c: u8) -> Option<FileStatus> {
    match c {
        b'M' => Some(FileStatus::Modified),
        b'A' => Some(FileStatus::Added),
        b'D' => Some(FileStatus::Deleted),
        b'R' => Some(FileStatus::Renamed),
        b'C' => Some(FileStatus::Copied),
        b'?' => Some(FileStatus::Untracked),
        b'U' => Some(FileStatus::Unmerged),
        b' ' | b'!' => None,
        _ => None,
    }
}
