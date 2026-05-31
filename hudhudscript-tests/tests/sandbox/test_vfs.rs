//! Tests extracted from hudhudscript-sandbox/src/vfs.rs

use hudhudscript_sandbox::{vfs::WorkspaceVfs, SandboxError};
use std::path::{Path, PathBuf};

fn workspace() -> WorkspaceVfs {
    WorkspaceVfs::new("/workspace").unwrap()
}

#[test]
fn test_resolve_simple_path() {
    let vfs = workspace();
    let resolved = vfs.resolve("data/file.txt").unwrap();
    assert_eq!(resolved, PathBuf::from("/workspace/data/file.txt"));
}

#[test]
fn test_resolve_absolute_virtual_path() {
    let vfs = workspace();
    // Callers may pass /data/file.txt meaning "inside the workspace"
    let resolved = vfs.resolve("/data/file.txt").unwrap();
    assert_eq!(resolved, PathBuf::from("/workspace/data/file.txt"));
}

#[test]
fn test_resolve_dot_dot_escape_rejected() {
    let vfs = workspace();
    // Attempt to escape via ../../../etc/passwd
    let err = vfs.resolve("../../../etc/passwd").unwrap_err();
    assert!(matches!(err, SandboxError::FileSystemDenied(_)));
}

#[test]
fn test_resolve_complex_traversal_rejected() {
    let vfs = workspace();
    let err = vfs.resolve("data/../../etc/shadow").unwrap_err();
    assert!(matches!(err, SandboxError::FileSystemDenied(_)));
}

#[test]
fn test_resolve_dot_components_normalised() {
    let vfs = workspace();
    let resolved = vfs.resolve("data/./subdir/../file.txt").unwrap();
    assert_eq!(resolved, PathBuf::from("/workspace/data/file.txt"));
}

#[test]
fn test_check_read_inside_workspace() {
    let vfs = workspace();
    assert!(vfs.check_read("data/file.txt").is_ok());
}

#[test]
fn test_check_read_escape_rejected() {
    let vfs = workspace();
    assert!(vfs.check_read("../../etc/passwd").is_err());
}

#[test]
fn test_check_write_inside_workspace() {
    let vfs = workspace();
    assert!(vfs.check_write("output/result.json").is_ok());
}

#[test]
fn test_read_only_vfs_rejects_writes() {
    let vfs = WorkspaceVfs::read_only("/workspace").unwrap();
    let err = vfs.check_write("output/result.json").unwrap_err();
    assert!(matches!(err, SandboxError::FileSystemDenied(_)));
}

#[test]
fn test_relative_root_rejected() {
    let err = WorkspaceVfs::new("relative/path").unwrap_err();
    assert!(matches!(err, SandboxError::InvalidConfig(_)));
}

#[test]
fn test_to_virtual_path() {
    let vfs = workspace();
    let virtual_path = vfs.to_virtual("/workspace/data/file.txt");
    assert_eq!(virtual_path, Some(PathBuf::from("data/file.txt")));
}

#[test]
fn test_to_virtual_outside_workspace() {
    let vfs = workspace();
    let result = vfs.to_virtual("/etc/passwd");
    assert!(result.is_none());
}

#[test]
fn test_path_components() {
    let vfs = workspace();
    let comps = vfs.path_components("a/b/c.txt").unwrap();
    assert_eq!(comps, vec!["a", "b", "c.txt"]);
}

#[test]
fn test_is_read_only() {
    let vfs = WorkspaceVfs::read_only("/workspace").unwrap();
    assert!(vfs.is_read_only());

    let vfs2 = workspace();
    assert!(!vfs2.is_read_only());
}

#[test]
fn test_read_only_vfs_allows_reads() {
    let vfs = WorkspaceVfs::read_only("/workspace").unwrap();
    assert!(vfs.check_read("data/file.txt").is_ok());
}

#[test]
fn test_check_write_escape_rejected() {
    let vfs = workspace();
    assert!(vfs.check_write("../../etc/passwd").is_err());
}

#[test]
fn test_to_virtual_root_itself() {
    let vfs = workspace();
    let result = vfs.to_virtual("/workspace");
    assert_eq!(result, Some(PathBuf::from("")));
}

#[test]
fn test_path_components_nested() {
    let vfs = workspace();
    let comps = vfs.path_components("a/b/c/d.txt").unwrap();
    assert_eq!(comps, vec!["a", "b", "c", "d.txt"]);
}

#[test]
fn test_resolve_root_path() {
    let vfs = workspace();
    let resolved = vfs.resolve("").unwrap();
    assert_eq!(resolved, PathBuf::from("/workspace"));
}

#[test]
fn test_workspace_root_accessor() {
    let vfs = workspace();
    assert_eq!(vfs.workspace_root(), Path::new("/workspace"));
}
