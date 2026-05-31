//! Real git integration via `std::process::Command`
//!
//! Provides a `GitRepo` handle bound to a working directory. Every method
//! shells out to the system `git` binary — no libgit2, no in-process fakes.

use crate::state_tree::VcsError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A handle to a git repository rooted at a specific directory.
#[derive(Debug, Clone)]
pub struct GitRepo {
    /// Working directory of the repository.
    workdir: PathBuf,
}

/// Output of `git status --porcelain`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusEntry {
    /// Two-character status code (e.g. "M ", "??", "A ", " M").
    pub code: String,
    /// Relative path of the file.
    pub path: String,
}

/// A single log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Full commit hash.
    pub hash: String,
    /// Author name.
    pub author: String,
    /// Commit date (ISO-ish string from git).
    pub date: String,
    /// First line of the commit message.
    pub subject: String,
}

impl GitRepo {
    // -----------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------

    /// Open an existing git repository at `path`.
    ///
    /// Returns an error if `path` is not inside a git work-tree.
    pub fn open(path: &Path) -> Result<Self, VcsError> {
        let workdir = path.to_path_buf();
        let repo = Self { workdir };
        // Verify it really is a git repo.
        repo.run(&["rev-parse", "--git-dir"])?;
        Ok(repo)
    }

    /// Initialise a new git repository at `path` (runs `git init`).
    pub fn init(path: &Path) -> Result<Self, VcsError> {
        std::fs::create_dir_all(path)
            .map_err(|e| VcsError::InvalidOperation(format!("Cannot create directory: {}", e)))?;
        let repo = Self {
            workdir: path.to_path_buf(),
        };
        repo.run(&["init"])?;
        Ok(repo)
    }

    /// Initialise if not already a repo, otherwise just open.
    ///
    /// Only falls back to `init()` if `open()` fails because the directory is
    /// not a git repository. Other failures (permission denied, corrupted
    /// `.git` directory, missing parent) are propagated. Previously this
    /// silently called `init()` on ANY error, which could destroy data —
    /// fixed in v0.4.47.9.
    pub fn init_or_open(path: &Path) -> Result<Self, VcsError> {
        match Self::open(path) {
            Ok(repo) => Ok(repo),
            Err(VcsError::InvalidOperation(msg))
                if msg.contains("not a git repository")
                    || msg.contains("Not a git repository")
                    || msg.contains("fatal: not a git repository") =>
            {
                // Genuinely not a repo — safe to init.
                Self::init(path)
            }
            Err(e) => {
                // Other error (permission denied, corrupted .git, etc.) — propagate.
                // Caller can decide what to do; we will not silently mask data loss.
                Err(e)
            }
        }
    }

    // -----------------------------------------------------------------
    // Core operations
    // -----------------------------------------------------------------

    /// `git status --porcelain` — returns parsed status entries.
    pub fn status(&self) -> Result<Vec<StatusEntry>, VcsError> {
        let output = self.run(&["status", "--porcelain"])?;
        let mut entries = Vec::new();
        for line in output.lines() {
            if line.len() < 4 {
                continue;
            }
            let code = line[..2].to_string();
            let path = line[3..].to_string();
            entries.push(StatusEntry { code, path });
        }
        Ok(entries)
    }

    /// `git add <paths>` — stage files. Pass `&["."]` to add everything.
    pub fn add(&self, paths: &[&str]) -> Result<(), VcsError> {
        let mut args = vec!["add"];
        args.extend(paths);
        self.run(&args)?;
        Ok(())
    }

    /// `git commit -m <message>`.
    pub fn commit(&self, message: &str) -> Result<String, VcsError> {
        self.run(&["commit", "-m", message])
    }

    /// `git diff` (unstaged changes) or `git diff --cached` (staged).
    pub fn diff(&self, staged: bool) -> Result<String, VcsError> {
        if staged {
            self.run(&["diff", "--cached"])
        } else {
            self.run(&["diff"])
        }
    }

    /// `git diff <commit_a> <commit_b>`.
    pub fn diff_commits(&self, a: &str, b: &str) -> Result<String, VcsError> {
        self.run(&["diff", a, b])
    }

    /// `git log` with a limited number of entries.
    pub fn log(&self, max_count: usize) -> Result<Vec<LogEntry>, VcsError> {
        let count_arg = format!("-{}", max_count);
        let output = self.run(&["log", &count_arg, "--format=%H%n%an%n%ai%n%s%n---"])?;

        let mut entries = Vec::new();
        let mut lines = output.lines().peekable();
        while lines.peek().is_some() {
            let hash = match lines.next() {
                Some(h) if !h.is_empty() => h.to_string(),
                _ => break,
            };
            // Each commit has 4 lines: hash, author, date, subject + separator.
            // If any line is missing, the git output is malformed — return error
            // instead of silently filling with empty strings (v0.4.47.9).
            let author = lines.next().ok_or_else(|| {
                VcsError::ParseError(format!(
                    "git log: missing 'author' line for commit {}",
                    hash
                ))
            })?;
            let date = lines.next().ok_or_else(|| {
                VcsError::ParseError(format!("git log: missing 'date' line for commit {}", hash))
            })?;
            let subject = lines.next().ok_or_else(|| {
                VcsError::ParseError(format!(
                    "git log: missing 'subject' line for commit {}",
                    hash
                ))
            })?;
            // consume separator
            let _ = lines.next();
            entries.push(LogEntry {
                hash,
                author: author.to_string(),
                date: date.to_string(),
                subject: subject.to_string(),
            });
        }
        Ok(entries)
    }

    /// `git branch` — list local branch names.
    pub fn branches(&self) -> Result<Vec<String>, VcsError> {
        let output = self.run(&["branch", "--format=%(refname:short)"])?;
        Ok(output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect())
    }

    /// `git checkout <branch>` — switch branches.
    pub fn checkout(&self, branch: &str) -> Result<(), VcsError> {
        self.run(&["checkout", branch])?;
        Ok(())
    }

    /// `git checkout -b <branch>` — create and switch to a new branch.
    pub fn checkout_new_branch(&self, branch: &str) -> Result<(), VcsError> {
        self.run(&["checkout", "-b", branch])?;
        Ok(())
    }

    /// Return the current branch name (`git rev-parse --abbrev-ref HEAD`).
    pub fn current_branch(&self) -> Result<String, VcsError> {
        let output = self.run(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(output.trim().to_string())
    }

    /// Return the working directory path.
    pub fn workdir(&self) -> &Path {
        &self.workdir
    }

    // -----------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------

    /// Run a git command and return its stdout.
    fn run(&self, args: &[&str]) -> Result<String, VcsError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.workdir)
            .output()
            .map_err(|e| {
                VcsError::InvalidOperation(format!(
                    "Failed to execute git {}: {}",
                    args.first().unwrap_or(&""),
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VcsError::InvalidOperation(format!(
                "git {} failed: {}",
                args.first().unwrap_or(&""),
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
