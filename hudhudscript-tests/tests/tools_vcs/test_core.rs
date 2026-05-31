//! External tests for the `hudhudscript-tools-vcs` crate.
//!
//! Covers: parse_status_char, FileStatus serde, StatusEntry/LogEntry/BranchInfo
//! construction, GitOutput fields, GitConfig constructors, GitTool/GitRepo
//! constructors, register_git_tools, error validation, MergeResult variants.

use hudhudscript_tools_vcs::{
    parse_status_char, register_git_tools, BranchInfo, FileStatus, GitConfig, GitOutput, GitRepo,
    GitTool, LogEntry, MergeResult, StatusEntry,
};

// ---------------------------------------------------------------------------
// 1. parse_status_char — exhaustive mapping
// ---------------------------------------------------------------------------

#[test]
fn parse_status_char_maps_all_known_codes() {
    assert_eq!(parse_status_char(b'M'), Some(FileStatus::Modified));
    assert_eq!(parse_status_char(b'A'), Some(FileStatus::Added));
    assert_eq!(parse_status_char(b'D'), Some(FileStatus::Deleted));
    assert_eq!(parse_status_char(b'R'), Some(FileStatus::Renamed));
    assert_eq!(parse_status_char(b'C'), Some(FileStatus::Copied));
    assert_eq!(parse_status_char(b'?'), Some(FileStatus::Untracked));
    assert_eq!(parse_status_char(b'U'), Some(FileStatus::Unmerged));
}

#[test]
fn parse_status_char_returns_none_for_space_bang_and_unknown() {
    assert_eq!(parse_status_char(b' '), None);
    assert_eq!(parse_status_char(b'!'), None);
    assert_eq!(parse_status_char(b'X'), None);
    assert_eq!(parse_status_char(b'\0'), None);
}

// ---------------------------------------------------------------------------
// 2. FileStatus serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn file_status_serde_round_trip() {
    let statuses = vec![
        FileStatus::Modified,
        FileStatus::Added,
        FileStatus::Deleted,
        FileStatus::Renamed,
        FileStatus::Copied,
        FileStatus::Untracked,
        FileStatus::Unmerged,
    ];

    let json = serde_json::to_string(&statuses).expect("serialize");
    let deserialized: Vec<FileStatus> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(statuses, deserialized);
}

// ---------------------------------------------------------------------------
// 3. StatusEntry construction and serde
// ---------------------------------------------------------------------------

#[test]
fn status_entry_round_trip() {
    let entry = StatusEntry {
        path: "src/main.rs".to_string(),
        index_status: Some(FileStatus::Modified),
        worktree_status: None,
    };

    let json = serde_json::to_string(&entry).expect("serialize");
    let de: StatusEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(de.path, "src/main.rs");
    assert_eq!(de.index_status, Some(FileStatus::Modified));
    assert!(de.worktree_status.is_none());
}

// ---------------------------------------------------------------------------
// 4. LogEntry construction and serde
// ---------------------------------------------------------------------------

#[test]
fn log_entry_round_trip() {
    let entry = LogEntry {
        hash: "abc1234".to_string(),
        author: "test_user".to_string(),
        date: "2026-04-04 10:00:00 +0300".to_string(),
        message: "initial commit".to_string(),
    };

    let json = serde_json::to_string(&entry).expect("serialize");
    let de: LogEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(de.hash, "abc1234");
    assert_eq!(de.author, "test_user");
    assert_eq!(de.message, "initial commit");
}

// ---------------------------------------------------------------------------
// 5. BranchInfo construction and serde
// ---------------------------------------------------------------------------

#[test]
fn branch_info_round_trip() {
    let info = BranchInfo {
        name: "main".to_string(),
        is_current: true,
        upstream: Some("origin/main".to_string()),
    };

    let json = serde_json::to_string(&info).expect("serialize");
    let de: BranchInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(de.name, "main");
    assert!(de.is_current);
    assert_eq!(de.upstream.as_deref(), Some("origin/main"));

    // Branch without upstream
    let detached = BranchInfo {
        name: "feature-x".to_string(),
        is_current: false,
        upstream: None,
    };
    let json2 = serde_json::to_string(&detached).expect("serialize");
    let de2: BranchInfo = serde_json::from_str(&json2).expect("deserialize");
    assert!(!de2.is_current);
    assert!(de2.upstream.is_none());
}

// ---------------------------------------------------------------------------
// 6. MergeResult serde
// ---------------------------------------------------------------------------

#[test]
fn merge_result_serde_variants() {
    let success = MergeResult::Success;
    let json_s = serde_json::to_string(&success).expect("serialize success");
    let de_s: MergeResult = serde_json::from_str(&json_s).expect("deserialize success");
    assert!(matches!(de_s, MergeResult::Success));

    let conflict = MergeResult::Conflict(vec!["a.rs".to_string(), "b.rs".to_string()]);
    let json_c = serde_json::to_string(&conflict).expect("serialize conflict");
    let de_c: MergeResult = serde_json::from_str(&json_c).expect("deserialize conflict");
    match de_c {
        MergeResult::Conflict(paths) => {
            assert_eq!(paths.len(), 2);
            assert_eq!(paths[0], "a.rs");
            assert_eq!(paths[1], "b.rs");
        }
        _ => panic!("expected Conflict variant"),
    }
}

// ---------------------------------------------------------------------------
// 7. GitOutput serde
// ---------------------------------------------------------------------------

#[test]
fn git_output_serde_round_trip() {
    let output = GitOutput {
        stdout: "On branch main".to_string(),
        stderr: String::new(),
        success: true,
        exit_code: 0,
    };

    let json = serde_json::to_string(&output).expect("serialize");
    let de: GitOutput = serde_json::from_str(&json).expect("deserialize");
    assert!(de.success);
    assert_eq!(de.exit_code, 0);
    assert_eq!(de.stdout, "On branch main");
    assert!(de.stderr.is_empty());
}

// ---------------------------------------------------------------------------
// 8. GitTool constructors and workdir
// ---------------------------------------------------------------------------

#[test]
fn git_tool_constructors() {
    let tool = GitTool::new("/tmp/fake-repo");
    assert_eq!(tool.workdir(), std::path::Path::new("/tmp/fake-repo"));

    let cwd = GitTool::current_dir();
    assert_eq!(cwd.workdir(), std::path::Path::new("."));

    let default_tool = GitTool::default();
    assert_eq!(default_tool.workdir(), std::path::Path::new("."));
}

// ---------------------------------------------------------------------------
// 9. GitRepo::open stores the path correctly
// ---------------------------------------------------------------------------

#[test]
fn git_repo_open_stores_path() {
    let repo = GitRepo::open("/tmp/my-repo");
    assert_eq!(repo.path(), std::path::Path::new("/tmp/my-repo"));
}

// ---------------------------------------------------------------------------
// 10. GitConfig constructors
// ---------------------------------------------------------------------------

#[test]
fn git_config_constructors() {
    let repo_cfg = GitConfig::for_repo("/tmp/some-repo");
    assert_eq!(
        repo_cfg.repo_path(),
        Some(std::path::Path::new("/tmp/some-repo"))
    );

    let global_cfg = GitConfig::global();
    assert!(global_cfg.repo_path().is_none());
}

// ---------------------------------------------------------------------------
// 11. register_git_tools registers all 6 tools
// ---------------------------------------------------------------------------

#[test]
fn register_git_tools_registers_all_tools() {
    let registry = hudhudscript_tools_schema::ToolRegistry::new();
    let count = register_git_tools(&registry).expect("registration should succeed");
    assert_eq!(count, 6, "expected 6 git tools to be registered");

    // Verify each tool exists by name
    let expected_names = [
        "git_status",
        "git_commit",
        "git_push",
        "git_branch",
        "git_checkout",
        "git_log",
    ];
    for name in &expected_names {
        assert!(
            registry.get_tool(name).is_some(),
            "tool '{}' should be registered",
            name
        );
    }
}

// ---------------------------------------------------------------------------
// 12. register_git_tools fails on duplicate registration
// ---------------------------------------------------------------------------

#[test]
fn register_git_tools_duplicate_returns_error() {
    let registry = hudhudscript_tools_schema::ToolRegistry::new();
    let _ = register_git_tools(&registry).expect("first registration");
    let result = register_git_tools(&registry);
    assert!(result.is_err(), "duplicate registration should fail");
}
