//! Real unit tests for hudhudscript-tools-vcs — git types, errors, config

use hudhudscript_tools_vcs::*;

// ── GitError ─────────────────────────────────────────────────────────────

#[test]
fn git_error_display_all_variants() {
    // Exercise Display for each variant — should not panic
    let _ = format!("{}", GitError::GitNotFound);
    let _ = format!(
        "{}",
        GitError::CommandFailed {
            code: 128,
            stderr: "err".into()
        }
    );
    let _ = format!("{}", GitError::SpawnFailed("oops".into()));
    let _ = format!("{}", GitError::InvalidArguments("bad".into()));
    let _ = format!("{}", GitError::RepositoryNotFound("/x".into()));
    let _ = format!("{}", GitError::ParseError("bad".into()));
}

#[test]
fn git_error_code_returns_valid() {
    let err = GitError::GitNotFound;
    let code = err.code();
    assert!(!code.short_code().is_empty());
}

// ── FileStatus ───────────────────────────────────────────────────────────

#[test]
fn file_status_parse_modified() {
    assert_eq!(parse_status_char(b'M'), Some(FileStatus::Modified));
}

#[test]
fn file_status_parse_added() {
    assert_eq!(parse_status_char(b'A'), Some(FileStatus::Added));
}

#[test]
fn file_status_parse_deleted() {
    assert_eq!(parse_status_char(b'D'), Some(FileStatus::Deleted));
}

#[test]
fn file_status_parse_renamed() {
    assert_eq!(parse_status_char(b'R'), Some(FileStatus::Renamed));
}

#[test]
fn file_status_parse_untracked() {
    assert_eq!(parse_status_char(b'?'), Some(FileStatus::Untracked));
}

#[test]
fn file_status_parse_unknown_returns_none() {
    assert_eq!(parse_status_char(b'X'), None);
    assert_eq!(parse_status_char(b' '), None);
}

// ── GitConfig ────────────────────────────────────────────────────────────

#[test]
fn git_config_global_does_not_panic() {
    let _config = GitConfig::global();
}

#[test]
fn git_config_for_repo_construction() {
    let config = GitConfig::for_repo("/tmp/test-repo");
    assert_eq!(
        config.repo_path(),
        Some(std::path::Path::new("/tmp/test-repo"))
    );
}

#[test]
fn git_config_get_nonexistent_key_does_not_panic() {
    let config = GitConfig::for_repo("/tmp/test-repo");
    let _result = config.get("user.name");
}

// ── GitOutput ────────────────────────────────────────────────────────────

#[test]
fn git_output_success() {
    let output = GitOutput {
        exit_code: 0,
        stdout: "abc123".into(),
        stderr: String::new(),
        success: true,
    };
    assert!(output.success);
    assert_eq!(output.stdout, "abc123");
}

#[test]
fn git_output_failure() {
    let output = GitOutput {
        exit_code: 128,
        stdout: String::new(),
        stderr: "error".into(),
        success: false,
    };
    assert!(!output.success);
    assert!(!output.stderr.is_empty());
}

// ── MergeResult ──────────────────────────────────────────────────────────

#[test]
fn merge_result_success() {
    assert!(matches!(MergeResult::Success, MergeResult::Success));
}

#[test]
fn merge_result_conflict() {
    let r = MergeResult::Conflict(vec!["file1.rs".to_string()]);
    assert!(matches!(r, MergeResult::Conflict(_)));
}

// ── BranchInfo ───────────────────────────────────────────────────────────

#[test]
fn branch_info_construction() {
    let branch = BranchInfo {
        name: "main".into(),
        is_current: true,
        upstream: Some("origin/main".into()),
    };
    assert_eq!(branch.name, "main");
    assert!(branch.is_current);
}

// ── StatusEntry ──────────────────────────────────────────────────────────

#[test]
fn status_entry_construction() {
    let entry = StatusEntry {
        path: "src/main.rs".into(),
        index_status: Some(FileStatus::Modified),
        worktree_status: Some(FileStatus::Modified),
    };
    assert_eq!(entry.path, "src/main.rs");
}

// ── LogEntry ─────────────────────────────────────────────────────────────

#[test]
fn log_entry_construction() {
    let entry = LogEntry {
        hash: "abc123".into(),
        author: "dev".into(),
        date: "2026-01-01".into(),
        message: "fix: bug".into(),
    };
    assert_eq!(entry.hash, "abc123");
    assert_eq!(entry.author, "dev");
    assert_eq!(entry.message, "fix: bug");
}
