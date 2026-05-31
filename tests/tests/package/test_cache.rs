//! External tests for hudhudscript-package::cache — PackageCache.

use flate2::write::GzEncoder;
use flate2::Compression;
use hudhudscript_package::PackageCache;
use std::io::Write;

#[test]
fn test_cache_lifecycle() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();

    assert!(!cache.is_installed("test-pkg", "1.0.0").unwrap());

    cache.install("test-pkg", "1.0.0", &[]).unwrap();
    assert!(cache.is_installed("test-pkg", "1.0.0").unwrap());

    let list = cache.list_installed().unwrap();
    assert!(list.contains(&"test-pkg".to_string()));

    cache.remove("test-pkg").unwrap();
    assert!(!cache.is_installed("test-pkg", "1.0.0").unwrap());
}

#[test]
fn test_cache_extract_tarball() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();

    let tarball = {
        let mut builder = tar::Builder::new(Vec::new());

        let content = b"print(\"hello\")\n";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "main.hud", &content[..])
            .unwrap();

        let tar_data = builder.into_inner().unwrap();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&tar_data).unwrap();
        encoder.finish().unwrap()
    };

    cache.install("my-pkg", "0.1.0", &tarball).unwrap();
    assert!(cache.is_installed("my-pkg", "0.1.0").unwrap());

    let extracted = cache.package_path("my-pkg", "0.1.0").join("main.hud");
    assert!(extracted.exists());
    let content = std::fs::read_to_string(extracted).unwrap();
    assert!(content.contains("hello"));
}

#[test]
fn test_cache_clean() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    cache.install("a", "1.0.0", &[]).unwrap();
    cache.install("b", "2.0.0", &[]).unwrap();

    cache.clean().unwrap();
    let list = cache.list_installed().unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_package_path() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    let path = cache.package_path("my-pkg", "1.2.3");
    assert!(path.ends_with("my-pkg/1.2.3"));
}

#[test]
fn test_cache_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    assert_eq!(cache.cache_dir(), tmp.path());
}

#[test]
fn test_remove_nonexistent_package() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    // Removing a package that does not exist should succeed silently
    let result = cache.remove("does-not-exist");
    assert!(result.is_ok());
}

#[test]
fn test_multiple_versions_same_package() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    cache.install("pkg", "1.0.0", &[]).unwrap();
    cache.install("pkg", "2.0.0", &[]).unwrap();

    assert!(cache.is_installed("pkg", "1.0.0").unwrap());
    assert!(cache.is_installed("pkg", "2.0.0").unwrap());
    assert!(!cache.is_installed("pkg", "3.0.0").unwrap());

    // list_installed returns package names (directories), not versions
    let list = cache.list_installed().unwrap();
    assert_eq!(list.len(), 1);
    assert!(list.contains(&"pkg".to_string()));
}

#[test]
fn test_cache_install_empty_tarball_creates_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();

    cache.install("empty-pkg", "0.1.0", &[]).unwrap();
    assert!(cache.is_installed("empty-pkg", "0.1.0").unwrap());

    let pkg_path = cache.package_path("empty-pkg", "0.1.0");
    assert!(pkg_path.is_dir());
}

#[test]
fn test_cache_clean_on_empty_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    // Clean on empty cache should work
    cache.clean().unwrap();
    let list = cache.list_installed().unwrap();
    assert!(list.is_empty());
}

#[test]
fn test_cache_remove_and_list() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    cache.install("a", "1.0.0", &[]).unwrap();
    cache.install("b", "1.0.0", &[]).unwrap();
    cache.install("c", "1.0.0", &[]).unwrap();

    let list = cache.list_installed().unwrap();
    assert_eq!(list.len(), 3);

    cache.remove("b").unwrap();
    let list = cache.list_installed().unwrap();
    assert_eq!(list.len(), 2);
    assert!(!list.contains(&"b".to_string()));
}

#[test]
fn test_cache_install_overwrites_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();

    // Install empty, then re-install empty - should not error
    cache.install("pkg", "1.0.0", &[]).unwrap();
    cache.install("pkg", "1.0.0", &[]).unwrap();
    assert!(cache.is_installed("pkg", "1.0.0").unwrap());
}

#[test]
fn test_cache_clone() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = PackageCache::new(&tmp.path().to_path_buf()).unwrap();
    let cloned = cache.clone();
    assert_eq!(cloned.cache_dir(), cache.cache_dir());
}
