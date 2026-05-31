use hudhudscript_tools::git::{register_git_tools, GitError, GitOutput, GitTool};
use hudhudscript_tools::registry::ToolRegistry;
use std::path::Path;

#[test]
fn test_git_tool_default_workdir() {
    let tool = GitTool::current_dir();
    assert_eq!(tool.workdir(), Path::new("."));
}

#[test]
fn test_git_tool_custom_workdir() {
    let tool = GitTool::new("/tmp");
    assert_eq!(tool.workdir(), Path::new("/tmp"));
}

#[test]
fn test_commit_empty_message_rejected() {
    let tool = GitTool::new("/tmp");
    let result = tool.commit("", false);
    assert!(matches!(result, Err(GitError::InvalidArguments(_))));
}

#[test]
fn test_checkout_empty_target_rejected() {
    let tool = GitTool::new("/tmp");
    let result = tool.checkout("");
    assert!(matches!(result, Err(GitError::InvalidArguments(_))));
}

#[test]
fn test_branch_create_empty_name_rejected() {
    let tool = GitTool::new("/tmp");
    let result = tool.branch_create("");
    assert!(matches!(result, Err(GitError::InvalidArguments(_))));
}

#[test]
fn test_add_empty_paths_rejected() {
    let tool = GitTool::new("/tmp");
    let result = tool.add(&[]);
    assert!(matches!(result, Err(GitError::InvalidArguments(_))));
}

#[test]
fn test_git_status_in_real_repo() {
    // Run from the actual repo root so git can find .git
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let tool = GitTool::new(repo);
    // status should succeed (or at least not return GitNotFound)
    match tool.status() {
        Ok(out) => assert!(out.exit_code == 0),
        Err(GitError::GitNotFound) => {} // git not installed in CI — skip
        Err(e) => panic!("unexpected error: {e}"),
    }
}

#[test]
fn test_git_log_in_real_repo() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let tool = GitTool::new(repo);
    match tool.log(Some(5)) {
        Ok(out) => assert!(out.success),
        Err(GitError::GitNotFound) => {}
        Err(e) => panic!("unexpected error: {e}"),
    }
}

#[test]
fn test_register_git_tools() {
    let registry = ToolRegistry::new();
    let count = register_git_tools(&registry).unwrap();
    assert_eq!(count, 6);
    for name in &[
        "git_status",
        "git_commit",
        "git_push",
        "git_branch",
        "git_checkout",
        "git_log",
    ] {
        assert!(registry.get_tool(name).is_some(), "missing tool: {name}");
    }
}

#[test]
fn test_git_tool_default_trait() {
    let tool = GitTool::default();
    assert_eq!(tool.workdir(), Path::new("."));
}

#[test]
fn test_git_error_display() {
    let err = GitError::GitNotFound;
    assert!(err.to_string().contains("git command not found on PATH"));

    let err = GitError::InvalidArguments("empty message".to_string());
    assert!(err.to_string().contains("empty message"));

    let err = GitError::RepositoryNotFound("/no/repo".to_string());
    assert!(err.to_string().contains("/no/repo"));
}

#[test]
fn test_checkout_new_branch_empty_rejected() {
    let tool = GitTool::new("/tmp");
    let result = tool.checkout_new_branch("");
    assert!(matches!(result, Err(GitError::InvalidArguments(_))));
}

#[test]
fn test_branch_delete_empty_rejected() {
    let tool = GitTool::new("/tmp");
    let result = tool.branch_delete("");
    assert!(matches!(result, Err(GitError::InvalidArguments(_))));
}

#[test]
fn test_register_git_tools_metadata_tags() {
    let registry = ToolRegistry::new();
    register_git_tools(&registry).unwrap();

    let meta = registry.get_metadata("git_commit").unwrap();
    assert_eq!(meta.server, "built-in");
    assert!(meta.tags.contains(&"git".to_string()));
    assert!(meta.tags.contains(&"standard".to_string()));
}

#[test]
fn test_git_status_porcelain_in_real_repo() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let tool = GitTool::new(repo);
    match tool.status_porcelain() {
        Ok(out) => assert_eq!(out.exit_code, 0),
        Err(GitError::GitNotFound) => {}
        Err(e) => panic!("unexpected error: {e}"),
    }
}

#[test]
fn test_git_error_command_failed_display() {
    let err = GitError::CommandFailed {
        code: 128,
        stderr: "fatal: not a git repository".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("128"));
    assert!(msg.contains("not a git repository"));
}

#[test]
fn test_git_error_spawn_failed_display() {
    let err = GitError::SpawnFailed("permission denied".to_string());
    assert!(err.to_string().contains("permission denied"));
}

#[test]
fn test_git_output_from_output() {
    let output = std::process::Output {
        status: std::process::ExitStatus::default(),
        stdout: b"hello\n".to_vec(),
        stderr: b"warn\n".to_vec(),
    };
    let git_out = GitOutput::from_output(output);
    assert_eq!(git_out.stdout, "hello");
    assert_eq!(git_out.stderr, "warn");
}

#[test]
fn test_git_diff_in_real_repo() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let tool = GitTool::new(repo);
    // diff (not staged) should succeed in a git repo
    match tool.diff(false) {
        Ok(out) => assert!(out.success),
        Err(GitError::GitNotFound) => {}
        Err(GitError::CommandFailed { .. }) => {} // possible in bare repos
        Err(e) => panic!("unexpected error: {e}"),
    }
}

#[test]
fn test_git_log_format_in_real_repo() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let tool = GitTool::new(repo);
    match tool.log_format(Some(3), Some("%h %s")) {
        Ok(out) => assert!(out.success),
        Err(GitError::GitNotFound) => {}
        Err(e) => panic!("unexpected error: {e}"),
    }
}

#[test]
fn test_git_log_default_count() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let tool = GitTool::new(repo);
    match tool.log(None) {
        Ok(out) => assert!(out.success),
        Err(GitError::GitNotFound) => {}
        Err(e) => panic!("unexpected error: {e}"),
    }
}

#[test]
fn test_git_branch_list_in_real_repo() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let tool = GitTool::new(repo);
    match tool.branch_list() {
        Ok(out) => assert!(out.success),
        Err(GitError::GitNotFound) => {}
        Err(e) => panic!("unexpected error: {e}"),
    }
}
