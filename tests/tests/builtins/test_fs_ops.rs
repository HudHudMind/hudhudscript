//! FS builtin integration tests (Kural 7 follow-up).
//!
//! Previously imported seven `fs_*` helper functions from
//! `hudhudscript_builtins::builtins::fs_ops` and called them directly.
//! Those helpers were a second copy of exactly the same operations
//! already implemented in
//! `hudhudscript_shared_builtins::fs_builtins::handle_fs_method` and
//! wired into both runtimes through it; the hudhudscript-builtins
//! wrappers had no other callers in the workspace and were deleted as
//! dead shadow code. The tests live on and exercise the canonical
//! shared path so any regression in the real filesystem dispatch is
//! caught immediately.

use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::fs_builtins;

#[test]
fn test_fs_stat() {
    let result = fs_builtins::dispatch("stat", &[Value16::string("/tmp".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("is_dir"), Some(&Value16::boolean(true)));
        assert_eq!(obj.get("is_file"), Some(&Value16::boolean(false)));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_fs_symlink_and_readlink() {
    let tmp = tempfile::TempDir::new().unwrap();
    let target = tmp.path().join("target.txt");
    let link = tmp.path().join("link.txt");
    std::fs::write(&target, "hello").unwrap();

    fs_builtins::dispatch(
        "symlink",
        &[
            Value16::string(target.to_str().unwrap().to_string()),
            Value16::string(link.to_str().unwrap().to_string()),
        ],
    )
    .unwrap();

    let result = fs_builtins::dispatch(
        "readlink",
        &[Value16::string(link.to_str().unwrap().to_string())],
    )
    .unwrap();
    assert_eq!(
        result,
        Value16::string(target.to_str().unwrap().to_string())
    );
}

#[test]
fn test_fs_mkdir_p() {
    let tmp = tempfile::TempDir::new().unwrap();
    let deep = tmp.path().join("a/b/c/d");
    fs_builtins::dispatch(
        "mkdir_p",
        &[Value16::string(deep.to_str().unwrap().to_string())],
    )
    .unwrap();
    assert!(deep.exists());
}

#[test]
fn test_fs_copy_and_rename() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    let renamed = tmp.path().join("renamed.txt");
    std::fs::write(&src, "content").unwrap();

    let bytes = fs_builtins::dispatch(
        "copy",
        &[
            Value16::string(src.to_str().unwrap().to_string()),
            Value16::string(dst.to_str().unwrap().to_string()),
        ],
    )
    .unwrap();
    assert_eq!(bytes, Value16::number(7.0));

    fs_builtins::dispatch(
        "rename",
        &[
            Value16::string(dst.to_str().unwrap().to_string()),
            Value16::string(renamed.to_str().unwrap().to_string()),
        ],
    )
    .unwrap();
    assert!(renamed.exists());
    assert!(!dst.exists());
}

#[test]
fn test_fs_watch() {
    let result = fs_builtins::dispatch("watch", &[Value16::string("/tmp".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("exists"), Some(&Value16::boolean(true)));
    } else {
        panic!("Expected object");
    }
}
