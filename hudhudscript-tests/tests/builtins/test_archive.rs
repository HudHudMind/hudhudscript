use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::archive_ops as archive_impl;

fn compress(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    archive_impl::compress::compress(args)
}
fn decompress(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    archive_impl::compress::decompress(args)
}
fn create_tar_gz(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    archive_impl::tar::create_tar_gz(args)
}
fn extract_tar_gz(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    archive_impl::tar::extract_tar_gz(args)
}
fn list_archive(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    archive_impl::list::list_archive(args)
}

// NOTE: `test_create_archive_module_has_all_methods` was deleted — it
// probed the `Value::NativeFunction`-based module layout from
// `create_archive_module()`, which is interpreter-era infrastructure with
// no counterpart in the shared dispatch path. The actual archive
// operations are now covered by the round-trip tests below.

#[test]
fn test_compress_decompress_gzip_roundtrip() {
    let original = "Hello, HudHudScript archive module!";
    let compressed = compress(&[
        Value16::string(original.to_string()),
        Value16::string("gzip".to_string()),
    ]);
    assert!(compressed.is_ok(), "compress should succeed");

    let compressed_val = compressed.unwrap();
    let decompressed = decompress(&[compressed_val, Value16::string("gzip".to_string())]);
    assert!(decompressed.is_ok(), "decompress should succeed");
    assert_eq!(decompressed.unwrap(), Value16::string(original.to_string()));
}

#[test]
fn test_tar_gz_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let tmp_path = tmp.path();

    // Create test files
    let file1 = tmp_path.join("test1.txt");
    let file2 = tmp_path.join("test2.txt");
    std::fs::write(&file1, "content1").unwrap();
    std::fs::write(&file2, "content2").unwrap();

    let archive_path = tmp_path.join("test.tar.gz");
    let extract_dir = tmp_path.join("extracted");

    // Create archive (run from tmp dir so relative paths work)
    let result = std::process::Command::new("tar")
        .arg("czf")
        .arg(archive_path.to_str().unwrap())
        .arg("-C")
        .arg(tmp_path.to_str().unwrap())
        .arg("test1.txt")
        .arg("test2.txt")
        .output();

    if result.is_err() || !result.as_ref().unwrap().status.success() {
        // tar not available, skip test
        return;
    }

    // Extract
    let extract_result = extract_tar_gz(&[
        Value16::string(archive_path.to_str().unwrap().to_string()),
        Value16::string(extract_dir.to_str().unwrap().to_string()),
    ]);
    assert!(extract_result.is_ok(), "extract_tar_gz should succeed");

    assert!(extract_dir.join("test1.txt").exists());
    assert!(extract_dir.join("test2.txt").exists());
    assert_eq!(
        std::fs::read_to_string(extract_dir.join("test1.txt")).unwrap(),
        "content1"
    );

    // List
    let list_result = list_archive(&[Value16::string(archive_path.to_str().unwrap().to_string())]);
    assert!(list_result.is_ok());
    if let Some(entries) = list_result.unwrap().as_array() {
        assert!(entries.len() >= 2);
    }
}

#[test]
fn test_unsupported_compress_algorithm() {
    let result = compress(&[
        Value16::string("data".to_string()),
        Value16::string("lzma".to_string()),
    ]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("unsupported algorithm"));
}

#[test]
fn test_empty_files_array_error() {
    let result = create_tar_gz(&[
        Value16::string("/tmp/test.tar.gz".to_string()),
        Value16::array(vec![]),
    ]);
    assert!(result.is_err());
}
