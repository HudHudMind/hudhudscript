use crate::git::error::GitError;
use crate::git::types::GitOutput;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitTool {
    workdir: PathBuf,
}

impl GitTool {
    pub fn new(workdir: impl Into<PathBuf>) -> Self {
        Self {
            workdir: workdir.into(),
        }
    }

    pub fn current_dir() -> Self {
        Self {
            workdir: PathBuf::from("."),
        }
    }

    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    fn run(&self, args: &[&str]) -> Result<GitOutput, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    GitError::GitNotFound
                } else {
                    GitError::SpawnFailed(e.to_string())
                }
            })?;
        let result = GitOutput::from_output(output);
        if result.success {
            Ok(result)
        } else {
            Err(GitError::CommandFailed {
                code: result.exit_code,
                stderr: result.stderr.clone(),
            })
        }
    }

    fn run_allow_failure(&self, args: &[&str]) -> Result<GitOutput, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    GitError::GitNotFound
                } else {
                    GitError::SpawnFailed(e.to_string())
                }
            })?;
        Ok(GitOutput::from_output(output))
    }

    pub fn status(&self) -> Result<GitOutput, GitError> {
        self.run_allow_failure(&["status"])
    }

    pub fn status_porcelain(&self) -> Result<GitOutput, GitError> {
        self.run_allow_failure(&["status", "--porcelain"])
    }

    pub fn commit(&self, message: &str, stage_all: bool) -> Result<GitOutput, GitError> {
        if message.is_empty() {
            return Err(GitError::InvalidArguments(
                "commit message must not be empty".into(),
            ));
        }
        if stage_all {
            self.run(&["add", "-A"])?;
        }
        self.run(&["commit", "-m", message])
    }

    pub fn push(&self, remote: Option<&str>, branch: Option<&str>) -> Result<GitOutput, GitError> {
        let mut args = vec!["push"];
        if let Some(r) = remote {
            args.push(r);
        }
        if let Some(b) = branch {
            args.push(b);
        }
        self.run(&args)
    }

    pub fn branch_list(&self) -> Result<GitOutput, GitError> {
        self.run(&["branch"])
    }

    pub fn branch_create(&self, name: &str) -> Result<GitOutput, GitError> {
        if name.is_empty() {
            return Err(GitError::InvalidArguments(
                "branch name must not be empty".into(),
            ));
        }
        self.run(&["branch", name])
    }

    pub fn branch_delete(&self, name: &str) -> Result<GitOutput, GitError> {
        if name.is_empty() {
            return Err(GitError::InvalidArguments(
                "branch name must not be empty".into(),
            ));
        }
        self.run(&["branch", "-d", name])
    }

    pub fn checkout(&self, target: &str) -> Result<GitOutput, GitError> {
        if target.is_empty() {
            return Err(GitError::InvalidArguments(
                "checkout target must not be empty".into(),
            ));
        }
        self.run(&["checkout", target])
    }

    pub fn checkout_new_branch(&self, name: &str) -> Result<GitOutput, GitError> {
        if name.is_empty() {
            return Err(GitError::InvalidArguments(
                "branch name must not be empty".into(),
            ));
        }
        self.run(&["checkout", "-b", name])
    }

    pub fn log(&self, n: Option<usize>) -> Result<GitOutput, GitError> {
        let count = format!("-{}", n.unwrap_or(10));
        self.run(&["log", "--oneline", &count])
    }

    pub fn log_format(
        &self,
        n: Option<usize>,
        format: Option<&str>,
    ) -> Result<GitOutput, GitError> {
        let count = format!("-{}", n.unwrap_or(10));
        let fmt = format.unwrap_or("%h %an %s");
        let fmt_arg = format!("--pretty=format:{}", fmt);
        self.run(&["log", &count, &fmt_arg])
    }

    pub fn add(&self, paths: &[&str]) -> Result<GitOutput, GitError> {
        if paths.is_empty() {
            return Err(GitError::InvalidArguments(
                "at least one path required for git add".into(),
            ));
        }
        let mut args = vec!["add", "--"];
        args.extend_from_slice(paths);
        self.run(&args)
    }

    pub fn diff(&self, staged: bool) -> Result<GitOutput, GitError> {
        if staged {
            self.run(&["diff", "--staged"])
        } else {
            self.run(&["diff"])
        }
    }

    pub fn pull(&self, remote: Option<&str>, branch: Option<&str>) -> Result<GitOutput, GitError> {
        let mut args = vec!["pull"];
        if let Some(r) = remote {
            args.push(r);
        }
        if let Some(b) = branch {
            args.push(b);
        }
        self.run(&args)
    }
}

impl Default for GitTool {
    fn default() -> Self {
        Self::current_dir()
    }
}
