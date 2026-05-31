//! External tests for `hudhudscript_parser::cache` module.

use hudhudscript_parser::ParseCache;

#[test]
fn test_cache_insert_and_get() {
    let cache = ParseCache::new(10);
    let source = "let x = 42".to_string();
    let ast = vec![];

    cache.insert(source.clone(), ast.clone());
    assert_eq!(cache.get(&source), Some(ast));
}

#[test]
fn test_cache_miss() {
    let cache = ParseCache::new(10);
    assert_eq!(cache.get("nonexistent"), None);
}

#[test]
fn test_cache_clear() {
    let cache = ParseCache::new(10);
    cache.insert("test".to_string(), vec![]);
    assert_eq!(cache.size(), 1);

    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_cache_max_size() {
    let cache = ParseCache::new(2);

    cache.insert("1".to_string(), vec![]);
    cache.insert("2".to_string(), vec![]);
    assert_eq!(cache.size(), 2);

    // Should clear when exceeding max_size
    cache.insert("3".to_string(), vec![]);
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_cache_default() {
    let cache = ParseCache::default();
    assert_eq!(cache.size(), 0);
    // Default max_size is 100 — we can insert up to 100 without clearing
    for i in 0..100 {
        cache.insert(format!("source_{}", i), vec![]);
    }
    assert_eq!(cache.size(), 100);
    // 101st insert triggers clear
    cache.insert("overflow".to_string(), vec![]);
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_cache_overwrite_same_key() {
    let cache = ParseCache::new(10);
    cache.insert("key".to_string(), vec![]);
    cache.insert("key".to_string(), vec![]);
    // Same key overwritten, size should be 1
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_cache_get_returns_cloned_data() {
    let cache = ParseCache::new(10);
    let ast = vec![];
    cache.insert("src".to_string(), ast.clone());
    let result = cache.get("src");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), ast);
}

#[test]
fn test_cache_clear_then_get() {
    let cache = ParseCache::new(10);
    cache.insert("src".to_string(), vec![]);
    cache.clear();
    assert_eq!(cache.get("src"), None);
}

#[test]
fn test_cache_size_zero() {
    let cache = ParseCache::new(0);
    // max_size is 0, every insert triggers clear first
    cache.insert("a".to_string(), vec![]);
    assert_eq!(cache.size(), 1);
}

#[test]
fn test_cache_multiple_gets() {
    let cache = ParseCache::new(10);
    cache.insert("a".to_string(), vec![]);
    // Multiple gets should all succeed
    assert!(cache.get("a").is_some());
    assert!(cache.get("a").is_some());
    assert!(cache.get("a").is_some());
}
