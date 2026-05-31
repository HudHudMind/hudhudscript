use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::codesign_ops as codesign_impl;
use std::io::Write;

fn codesign_hash_file(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    codesign_impl::codesign_hash_file(args)
}
fn codesign_generate_manifest(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    codesign_impl::codesign_generate_manifest(args)
}
fn codesign_verify_manifest(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    codesign_impl::codesign_verify_manifest(args)
}

#[test]
fn test_hash_file_sha256() {
    let dir = std::env::temp_dir().join("hudhud_codesign_test");
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join("test_hash.txt");
    let mut f = std::fs::File::create(&file_path).unwrap();
    f.write_all(b"hello").unwrap();

    let result = codesign_hash_file(&[
        Value16::string(file_path.to_string_lossy().to_string()),
        Value16::string("sha256".to_string()),
    ])
    .unwrap();

    if let Some(h) = result.as_str() {
        assert_eq!(
            h,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    } else {
        panic!("expected string");
    }

    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn test_hash_file_sha512() {
    let dir = std::env::temp_dir().join("hudhud_codesign_test");
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join("test_hash512.txt");
    let mut f = std::fs::File::create(&file_path).unwrap();
    f.write_all(b"hello").unwrap();

    let result = codesign_hash_file(&[
        Value16::string(file_path.to_string_lossy().to_string()),
        Value16::string("sha512".to_string()),
    ])
    .unwrap();

    if let Some(h) = result.as_str() {
        assert_eq!(h.len(), 128); // SHA-512 = 64 bytes = 128 hex chars
    } else {
        panic!("expected string");
    }

    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn test_hash_file_default_algorithm() {
    let dir = std::env::temp_dir().join("hudhud_codesign_test");
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join("test_hash_default.txt");
    let mut f = std::fs::File::create(&file_path).unwrap();
    f.write_all(b"hello").unwrap();

    // No algorithm argument — should default to sha256
    let result =
        codesign_hash_file(&[Value16::string(file_path.to_string_lossy().to_string())]).unwrap();

    if let Some(h) = result.as_str() {
        assert_eq!(
            h,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    } else {
        panic!("expected string");
    }

    let _ = std::fs::remove_file(&file_path);
}

#[test]
fn test_generate_and_verify_manifest() {
    let dir = std::env::temp_dir().join("hudhud_codesign_manifest_test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Create test files
    std::fs::write(dir.join("a.txt"), b"aaa").unwrap();
    std::fs::write(dir.join("b.txt"), b"bbb").unwrap();

    let manifest =
        codesign_generate_manifest(&[Value16::string(dir.to_string_lossy().to_string())]).unwrap();

    if let Some(m) = manifest.as_object() {
        assert!(m.contains_key("a.txt"));
        assert!(m.contains_key("b.txt"));
    } else {
        panic!("expected object");
    }

    // Verify should pass
    let verify_result = codesign_verify_manifest(&[
        Value16::string(dir.to_string_lossy().to_string()),
        manifest.clone(),
    ])
    .unwrap();

    if let Some(r) = verify_result.as_object() {
        assert_eq!(r.get("valid"), Some(&Value16::boolean(true)));
    } else {
        panic!("expected object");
    }

    // Tamper with a file and verify should fail
    std::fs::write(dir.join("a.txt"), b"tampered").unwrap();

    let verify_result2 =
        codesign_verify_manifest(&[Value16::string(dir.to_string_lossy().to_string()), manifest])
            .unwrap();

    if let Some(r) = verify_result2.as_object() {
        assert_eq!(r.get("valid"), Some(&Value16::boolean(false)));
    } else {
        panic!("expected object");
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_hash_file_not_found() {
    let result = codesign_hash_file(&[
        Value16::string("/nonexistent/path/file.txt".to_string()),
        Value16::string("sha256".to_string()),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_unsupported_hash_algorithm() {
    let dir = std::env::temp_dir().join("hudhud_codesign_test");
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join("test_unsupported.txt");
    std::fs::write(&file_path, b"data").unwrap();

    let result = codesign_hash_file(&[
        Value16::string(file_path.to_string_lossy().to_string()),
        Value16::string("md5".to_string()),
    ]);
    assert!(result.is_err());

    let _ = std::fs::remove_file(&file_path);
}
