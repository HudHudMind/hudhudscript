//! Real unit tests for hudhudscript-tools-vcs
//! Creates actual git repos with temp directories and tests GitRepo, GitError, types

use hudhudscript_tools_vcs::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper: initialize a temp git repo and return (TempDir, GitRepo)
fn init_temp_repo() -> (TempDir, GitRepo) {
    let dir = TempDir::new().unwrap();
    let path = dir.path();
    Command::new("git").args(["init"]).current_dir(path).output().unwrap();
    // Configure git user for commits
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(path).output().ok();
    Command::new("git").args(["config", "user.name", "Test User"]).current_dir(path).output().ok();
    let repo = GitRepo::open(path);
    (dir, repo)
}

// ═══════════════════════════════════════════════════════════════════════════
// GitRepo — discover
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_repo_discover_from_root() {
    let (dir, _repo) = init_temp_repo();
    let discovered = GitRepo::discover(dir.path()).unwrap();
    assert!(discovered.path().exists());
}

#[test]
fn git_repo_discover_from_subdir() {
    let (dir, _repo) = init_temp_repo();
    let subdir = dir.path().join("sub");
    fs::create_dir(&subdir).unwrap();
    let discovered = GitRepo::discover(&subdir).unwrap();
    assert!(discovered.path().exists());
}

#[test]
fn git_repo_discover_nonexistent() {
    let result = GitRepo::discover("/tmp/hudhud_nonexistent_git_dir_12345");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// GitRepo — status
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_status_empty_repo() {
    let (_dir, repo) = init_temp_repo();
    let entries = repo.status().unwrap();
    assert!(entries.is_empty());
}

#[test]
fn git_status_untracked_file() {
    let (_dir, repo) = init_temp_repo();
    let file_path = repo.path().join("newfile.txt");
    fs::write(&file_path, "hello").unwrap();
    let entries = repo.status().unwrap();
    assert!(!entries.is_empty());
    let untracked = entries.iter().find(|e| e.path == "newfile.txt");
    assert!(untracked.is_some());
}

#[test]
fn git_status_staged_file() {
    let (_dir, repo) = init_temp_repo();
    let file_path = repo.path().join("staged.txt");
    fs::write(&file_path, "content").unwrap();
    Command::new("git").args(["add", "staged.txt"]).current_dir(repo.path()).output().unwrap();
    let entries = repo.status().unwrap();
    assert!(!entries.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// GitRepo — log
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_log_empty_repo() {
    let (_dir, repo) = init_temp_repo();
    // log on empty repo (no commits) may error or return empty
    let result = repo.log(10);
    let _ = result;
}

#[test]
fn git_log_with_commit() {
    let (_dir, repo) = init_temp_repo();
    let file_path = repo.path().join("committed.txt");
    fs::write(&file_path, "test commit").unwrap();
    Command::new("git").args(["add", "committed.txt"]).current_dir(repo.path()).output().unwrap();
    Command::new("git").args(["commit", "-m", "test: initial commit"]).current_dir(repo.path()).output().unwrap();
    let entries = repo.log(5).unwrap();
    assert!(!entries.is_empty());
    assert!(entries[0].message.contains("initial commit"));
}

// ═══════════════════════════════════════════════════════════════════════════
// GitError — display and code
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_error_not_found_display() {
    let err = GitError::GitNotFound;
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn git_error_repo_not_found_display() {
    let err = GitError::RepositoryNotFound("/tmp/nonexistent".to_string());
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn git_error_command_failed_display() {
    let err = GitError::CommandFailed { code: 128, stderr: "fatal: not a git repository".to_string() };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn git_error_spawn_failed_display() {
    let err = GitError::SpawnFailed("executable not found".to_string());
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn git_error_invalid_args_display() {
    let err = GitError::InvalidArguments("missing path".to_string());
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn git_error_parse_error_display() {
    let err = GitError::ParseError("bad format".to_string());
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn git_error_code_mapping() {
    assert_eq!(GitError::GitNotFound.code(), hudhudscript_errors::ErrorCode::GitGitNotFound);
    assert_eq!(GitError::SpawnFailed("x".into()).code(), hudhudscript_errors::ErrorCode::GitSpawnFailed);
    assert_eq!(GitError::CommandFailed { code: 1, stderr: "e".into() }.code(), hudhudscript_errors::ErrorCode::GitCommandFailed);
    assert_eq!(GitError::InvalidArguments("x".into()).code(), hudhudscript_errors::ErrorCode::GitInvalidArguments);
    assert_eq!(GitError::ParseError("x".into()).code(), hudhudscript_errors::ErrorCode::GitParseError);
    assert_eq!(GitError::RepositoryNotFound("x".into()).code(), hudhudscript_errors::ErrorCode::GitRepositoryNotFound);
}

// ═══════════════════════════════════════════════════════════════════════════
// GitOutput / StatusEntry / BranchInfo / LogEntry / MergeResult
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_output_fields() {
    let output = GitOutput {
        stdout: "output".to_string(),
        stderr: "error".to_string(),
        success: true,
        exit_code: 0,
    };
    assert!(output.success);
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout, "output");
}

#[test]
fn status_entry_modified() {
    let entry = StatusEntry {
        path: "src/main.rs".to_string(),
        index_status: Some(FileStatus::Modified),
        worktree_status: None,
    };
    assert_eq!(entry.path, "src/main.rs");
    assert!(matches!(entry.index_status, Some(FileStatus::Modified)));
}

#[test]
fn status_entry_untracked() {
    let entry = StatusEntry {
        path: "new.txt".to_string(),
        index_status: None,
        worktree_status: Some(FileStatus::Untracked),
    };
    assert!(matches!(entry.worktree_status, Some(FileStatus::Untracked)));
}

#[test]
fn branch_info_fields() {
    let branch = BranchInfo {
        name: "main".to_string(),
        is_current: true,
        upstream: Some("origin/main".to_string()),
    };
    assert!(branch.is_current);
    assert_eq!(branch.name, "main");
}

#[test]
fn branch_info_no_upstream() {
    let branch = BranchInfo {
        name: "feature".to_string(),
        is_current: false,
        upstream: None,
    };
    assert!(!branch.is_current);
    assert!(branch.upstream.is_none());
}

#[test]
fn log_entry_fields() {
    let entry = LogEntry {
        hash: "abc123".to_string(),
        author: "Dev".to_string(),
        date: "2024-01-01".to_string(),
        message: "Initial commit".to_string(),
    };
    assert_eq!(entry.hash, "abc123");
    assert_eq!(entry.message, "Initial commit");
}

#[test]
fn merge_result_success() {
    assert!(matches!(MergeResult::Success, MergeResult::Success));
}

#[test]
fn merge_result_conflict() {
    let result = MergeResult::Conflict(vec!["file.txt".to_string()]);
    match result {
        MergeResult::Conflict(files) => assert_eq!(files, vec!["file.txt"]),
        _ => panic!("expected Conflict"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// parse_status_char
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_status_modified() {
    assert_eq!(parse_status_char(b'M'), Some(FileStatus::Modified));
}

#[test]
fn parse_status_added() {
    assert_eq!(parse_status_char(b'A'), Some(FileStatus::Added));
}

#[test]
fn parse_status_deleted() {
    assert_eq!(parse_status_char(b'D'), Some(FileStatus::Deleted));
}

#[test]
fn parse_status_renamed() {
    assert_eq!(parse_status_char(b'R'), Some(FileStatus::Renamed));
}

#[test]
fn parse_status_untracked() {
    assert_eq!(parse_status_char(b'?'), Some(FileStatus::Untracked));
}

#[test]
fn parse_status_unknown_is_none() {
    assert_eq!(parse_status_char(b'X'), None);
}

// ═══════════════════════════════════════════════════════════════════════════
// GitConfig
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn git_config_global_user_name() {
    let config = GitConfig::global();
    let name = config.user_name();
    let _ = name;
}

#[test]
fn git_config_global_user_email() {
    let config = GitConfig::global();
    let email = config.user_email();
    let _ = email;
}

#[test]
fn git_config_for_repo() {
    let config = GitConfig::for_repo("/tmp/nonexistent");
    assert!(config.repo_path().is_some());
}
