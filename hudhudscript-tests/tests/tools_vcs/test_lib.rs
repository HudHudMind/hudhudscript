use hudhudscript_tools_vcs::*;

// =========================================================================
// GithubTool
// =========================================================================

#[test]
fn test_github_tool_creation() {
    let t1 = GithubTool::new(Some("fake-token".into()));
    let t2 = GithubTool::new(None);
    // Both should construct without panic. Verify no-token case uses empty string.
    assert!(t1.list_issues("owner", "repo").is_err()); // no real token → HTTP error
    assert!(t2.list_issues("owner", "repo").is_err());
}

// =========================================================================
// GitOutput
// =========================================================================

#[test]
fn git_output_fields_accessible() {
    let output = GitOutput {
        stdout: "ok".to_string(),
        stderr: String::new(),
        success: true,
        exit_code: 0,
    };
    assert_eq!(output.stdout, "ok");
    assert!(output.success);
    assert_eq!(output.exit_code, 0);
}

#[test]
fn git_output_clone() {
    let output = GitOutput {
        stdout: "data".to_string(),
        stderr: "warn".to_string(),
        success: false,
        exit_code: 1,
    };
    let cloned = output.clone();
    assert_eq!(cloned.stdout, "data");
    assert_eq!(cloned.exit_code, 1);
}

// =========================================================================
// FileStatus
// =========================================================================

#[test]
fn file_status_eq() {
    assert_eq!(FileStatus::Modified, FileStatus::Modified);
    assert_ne!(FileStatus::Modified, FileStatus::Added);
}

#[test]
fn file_status_clone() {
    let s = FileStatus::Deleted;
    let s2 = s.clone();
    assert_eq!(s, s2);
}

#[test]
fn file_status_all_variants() {
    let variants = [
        FileStatus::Modified,
        FileStatus::Added,
        FileStatus::Deleted,
        FileStatus::Renamed,
        FileStatus::Copied,
        FileStatus::Untracked,
        FileStatus::Unmerged,
    ];
    assert_eq!(variants.len(), 7);
}

// =========================================================================
// StatusEntry
// =========================================================================

#[test]
fn status_entry_construction() {
    let entry = StatusEntry {
        path: "src/main.rs".to_string(),
        index_status: Some(FileStatus::Modified),
        worktree_status: None,
    };
    assert_eq!(entry.path, "src/main.rs");
    assert_eq!(entry.index_status, Some(FileStatus::Modified));
    assert!(entry.worktree_status.is_none());
}

#[test]
fn status_entry_serialize_deserialize() {
    let entry = StatusEntry {
        path: "file.txt".to_string(),
        index_status: Some(FileStatus::Added),
        worktree_status: Some(FileStatus::Modified),
    };
    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: StatusEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.path, "file.txt");
}

// =========================================================================
// LogEntry
// =========================================================================

#[test]
fn log_entry_construction() {
    let entry = LogEntry {
        hash: "abc1234".to_string(),
        author: "Dev".to_string(),
        date: "2025-01-01".to_string(),
        message: "initial commit".to_string(),
    };
    assert_eq!(entry.hash, "abc1234");
    assert_eq!(entry.message, "initial commit");
}

#[test]
fn log_entry_serialize_deserialize() {
    let entry = LogEntry {
        hash: "def".to_string(),
        author: "User".to_string(),
        date: "2025-06-01".to_string(),
        message: "feat: add tests".to_string(),
    };
    let json = serde_json::to_string(&entry).unwrap();
    let back: LogEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(back.author, "User");
}

// =========================================================================
// BranchInfo
// =========================================================================

#[test]
fn branch_info_construction() {
    let info = BranchInfo {
        name: "main".to_string(),
        is_current: true,
        upstream: Some("origin/main".to_string()),
    };
    assert_eq!(info.name, "main");
    assert!(info.is_current);
    assert_eq!(info.upstream, Some("origin/main".to_string()));
}

#[test]
fn branch_info_no_upstream() {
    let info = BranchInfo {
        name: "feature".to_string(),
        is_current: false,
        upstream: None,
    };
    assert!(info.upstream.is_none());
}

#[test]
fn branch_info_serialize_deserialize() {
    let info = BranchInfo {
        name: "dev".to_string(),
        is_current: false,
        upstream: None,
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: BranchInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "dev");
}

// =========================================================================
// MergeResult
// =========================================================================

#[test]
fn merge_result_success() {
    let result = MergeResult::Success;
    assert!(matches!(result, MergeResult::Success));
}

#[test]
fn merge_result_conflict() {
    let result = MergeResult::Conflict(vec!["file.txt".to_string(), "other.rs".to_string()]);
    if let MergeResult::Conflict(files) = result {
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], "file.txt");
    } else {
        panic!("expected Conflict");
    }
}

#[test]
fn merge_result_serialize() {
    let result = MergeResult::Success;
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Success"));
}

// =========================================================================
// GitError
// =========================================================================

#[test]
fn git_error_display_not_found() {
    let err = GitError::GitNotFound;
    assert!(format!("{err}").contains("git command not found on PATH"));
}

#[test]
fn git_error_display_command_failed() {
    let err = GitError::CommandFailed {
        code: 128,
        stderr: "fatal: not a git repository".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("128"));
    assert!(msg.contains("not a git repository"));
}

#[test]
fn git_error_display_spawn_failed() {
    let err = GitError::SpawnFailed("permission denied".to_string());
    assert!(format!("{err}").contains("permission denied"));
}

#[test]
fn git_error_display_invalid_args() {
    let err = GitError::InvalidArguments("empty branch name".to_string());
    assert!(format!("{err}").contains("empty branch name"));
}

#[test]
fn git_error_display_repo_not_found() {
    let err = GitError::RepositoryNotFound("/tmp/noexist".to_string());
    assert!(format!("{err}").contains("/tmp/noexist"));
}

#[test]
fn git_error_display_parse_error() {
    let err = GitError::ParseError("bad format".to_string());
    assert!(format!("{err}").contains("bad format"));
}

// =========================================================================
// GitTool — construction
// =========================================================================

#[test]
fn git_tool_new() {
    let tool = GitTool::new("/tmp/repo");
    assert_eq!(tool.workdir(), std::path::Path::new("/tmp/repo"));
}

#[test]
fn git_tool_current_dir() {
    let tool = GitTool::current_dir();
    assert_eq!(tool.workdir(), std::path::Path::new("."));
}

#[test]
fn git_tool_default() {
    let tool = GitTool::default();
    assert_eq!(tool.workdir(), std::path::Path::new("."));
}

// =========================================================================
// GitTool — validation (empty args)
// =========================================================================

#[test]
fn git_tool_commit_empty_message() {
    let tool = GitTool::new("/tmp");
    let err = tool.commit("", false).unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_tool_branch_create_empty_name() {
    let tool = GitTool::new("/tmp");
    let err = tool.branch_create("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_tool_branch_delete_empty_name() {
    let tool = GitTool::new("/tmp");
    let err = tool.branch_delete("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_tool_checkout_empty_target() {
    let tool = GitTool::new("/tmp");
    let err = tool.checkout("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_tool_checkout_new_branch_empty() {
    let tool = GitTool::new("/tmp");
    let err = tool.checkout_new_branch("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_tool_add_empty_paths() {
    let tool = GitTool::new("/tmp");
    let err = tool.add(&[]).unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

// =========================================================================
// GitRepo — construction and validation
// =========================================================================

#[test]
fn git_repo_open() {
    let repo = GitRepo::open("/tmp/test_repo");
    assert_eq!(repo.path(), std::path::Path::new("/tmp/test_repo"));
}

#[test]
fn git_repo_discover_on_real_repo() {
    // This test runs in a known git repo
    let repo = GitRepo::discover("..");
    assert!(repo.is_ok());
    let repo = repo.unwrap();
    assert!(repo.path().exists());
}

#[test]
fn git_repo_discover_nonexistent() {
    let result = GitRepo::discover("/tmp/definitely_not_a_git_repo_12345");
    assert!(result.is_err());
}

#[test]
fn git_repo_add_empty_paths() {
    let repo = GitRepo::open("/tmp");
    let err = repo.add(&[]).unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_repo_commit_empty_message() {
    let repo = GitRepo::open("/tmp");
    let err = repo.commit("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_repo_create_branch_empty_name() {
    let repo = GitRepo::open("/tmp");
    let err = repo.create_branch("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_repo_delete_branch_empty_name() {
    let repo = GitRepo::open("/tmp");
    let err = repo.delete_branch("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_repo_switch_branch_empty_name() {
    let repo = GitRepo::open("/tmp");
    let err = repo.switch_branch("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_repo_merge_empty_branch() {
    let repo = GitRepo::open("/tmp");
    let err = repo.merge("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

#[test]
fn git_repo_rebase_empty_onto() {
    let repo = GitRepo::open("/tmp");
    let err = repo.rebase("").unwrap_err();
    assert!(matches!(err, GitError::InvalidArguments(_)));
}

// =========================================================================
// GitRepo — operations on real repo
// =========================================================================

#[test]
fn git_repo_status_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let status = repo.status();
    assert!(status.is_ok());
}

#[test]
fn git_repo_log_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let log = repo.log(5).unwrap();
    assert!(log.len() <= 5);
    assert!(!log.is_empty());
    assert!(!log[0].hash.is_empty());
}

#[test]
fn git_repo_current_branch_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let branch = repo.current_branch().unwrap();
    assert!(!branch.is_empty());
}

#[test]
fn git_repo_branches_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let branches = repo.branches().unwrap();
    assert!(!branches.is_empty());
    let current = branches.iter().filter(|b| b.is_current).count();
    assert_eq!(current, 1);
}

#[test]
fn git_repo_diff_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let diff = repo.diff(None);
    assert!(diff.is_ok());
}

#[test]
fn git_repo_diff_staged_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let diff = repo.diff_staged();
    assert!(diff.is_ok());
}

#[test]
fn git_repo_remotes_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let remotes = repo.remotes().unwrap();
    // Most repos have at least "origin"
    assert!(!remotes.is_empty());
}

#[test]
fn git_repo_remote_url_on_real_repo() {
    let repo = GitRepo::discover("..").unwrap();
    let remotes = repo.remotes().unwrap();
    if !remotes.is_empty() {
        let url = repo.remote_url(&remotes[0]);
        assert!(url.is_ok());
        assert!(!url.unwrap().is_empty());
    }
}

// =========================================================================
// GitConfig
// =========================================================================

#[test]
fn git_config_for_repo() {
    let config = GitConfig::for_repo("..");
    assert!(config.repo_path().is_some());
    assert_eq!(
        config.repo_path().unwrap(),
        std::path::Path::new("..")
    );
}

#[test]
fn git_config_global() {
    let config = GitConfig::global();
    assert!(config.repo_path().is_none());
}

#[test]
fn git_config_get_user_name() {
    let config = GitConfig::for_repo("..");
    let name = config.user_name();
    assert!(name.is_ok());
}

#[test]
fn git_config_get_user_email() {
    let config = GitConfig::for_repo("..");
    let email = config.user_email();
    assert!(email.is_ok());
}

#[test]
fn git_config_get_nonexistent_key() {
    let config = GitConfig::for_repo("..");
    let result = config.get("hudhud.nonexistent.key.xyz");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

// =========================================================================
// GitTool — operations on real repo
// =========================================================================

#[test]
fn git_tool_status_on_real_repo() {
    let tool = GitTool::new("..");
    let output = tool.status().unwrap();
    assert!(output.success);
}

#[test]
fn git_tool_status_porcelain_on_real_repo() {
    let tool = GitTool::new("..");
    let output = tool.status_porcelain().unwrap();
    // porcelain output always succeeds even on clean repos
    assert!(output.success);
}

#[test]
fn git_tool_log_default_on_real_repo() {
    let tool = GitTool::new("..");
    let output = tool.log(Some(3)).unwrap();
    assert!(output.success);
    assert!(!output.stdout.is_empty());
}

#[test]
fn git_tool_log_format_on_real_repo() {
    let tool = GitTool::new("..");
    let output = tool.log_format(Some(2), Some("%H")).unwrap();
    assert!(output.success);
}

#[test]
fn git_tool_branch_list_on_real_repo() {
    let tool = GitTool::new("..");
    let output = tool.branch_list().unwrap();
    assert!(output.success);
}

#[test]
fn git_tool_diff_unstaged_on_real_repo() {
    let tool = GitTool::new("..");
    let output = tool.diff(false).unwrap();
    assert!(output.success);
}

#[test]
fn git_tool_diff_staged_on_real_repo() {
    let tool = GitTool::new("..");
    let output = tool.diff(true).unwrap();
    assert!(output.success);
}
