use hudhudscript_cache::{DedupResult, DedupStore};

#[test]
fn test_compute_hash_deterministic() {
    let h1 = DedupStore::compute_hash(b"hello world");
    let h2 = DedupStore::compute_hash(b"hello world");
    assert_eq!(h1, h2);
}

#[test]
fn test_compute_hash_different_content() {
    let h1 = DedupStore::compute_hash(b"hello");
    let h2 = DedupStore::compute_hash(b"world");
    assert_ne!(h1, h2);
}

#[test]
fn test_register_unique() {
    let mut store = DedupStore::new();
    let result = store.register("key1", b"content A");

    assert!(matches!(result, DedupResult::Unique { .. }));
    assert_eq!(store.total_key_count(), 1);
    assert_eq!(store.unique_content_count(), 1);
}

#[test]
fn test_register_duplicate() {
    let mut store = DedupStore::new();
    store.register("key1", b"same content");
    let result = store.register("key2", b"same content");

    match result {
        DedupResult::Duplicate { existing_keys, .. } => {
            assert_eq!(existing_keys, vec!["key1".to_string()]);
        }
        _ => panic!("Expected Duplicate result"),
    }

    assert_eq!(store.total_key_count(), 2);
    assert_eq!(store.unique_content_count(), 1);
    assert_eq!(store.duplicate_key_count(), 1);
}

#[test]
fn test_check_without_registering() {
    let mut store = DedupStore::new();
    store.register("key1", b"content");

    let result = store.check(b"content");
    assert!(matches!(result, DedupResult::Duplicate { .. }));

    let result = store.check(b"new content");
    assert!(matches!(result, DedupResult::Unique { .. }));
}

#[test]
fn test_remove() {
    let mut store = DedupStore::new();
    store.register("key1", b"content");
    store.register("key2", b"content");

    store.remove("key1");

    assert_eq!(store.total_key_count(), 1);
    assert!(store.hash_for_key("key1").is_none());
    assert!(store.hash_for_key("key2").is_some());
}

#[test]
fn test_remove_last_key_for_hash() {
    let mut store = DedupStore::new();
    store.register("key1", b"content");

    store.remove("key1");
    assert_eq!(store.unique_content_count(), 0);
    assert!(store.is_empty());
}

#[test]
fn test_re_register_with_different_content() {
    let mut store = DedupStore::new();
    store.register("key1", b"content A");
    let hash_a = store.hash_for_key("key1").cloned().unwrap();

    store.register("key1", b"content B");
    let hash_b = store.hash_for_key("key1").cloned().unwrap();

    assert_ne!(hash_a, hash_b);
    assert_eq!(store.total_key_count(), 1);
    assert!(store.keys_for_hash(&hash_a).is_none());
}

#[test]
fn test_estimated_savings() {
    let mut store = DedupStore::new();
    store.register("key1", b"same");
    store.register("key2", b"same");
    store.register("key3", b"same");
    store.register("key4", b"different");

    assert_eq!(store.duplicate_key_count(), 2);
    assert_eq!(store.estimated_savings(100), 200);
}

#[test]
fn test_clear() {
    let mut store = DedupStore::new();
    store.register("key1", b"content");
    store.register("key2", b"content");

    store.clear();
    assert!(store.is_empty());
    assert_eq!(store.unique_content_count(), 0);
    assert_eq!(store.total_key_count(), 0);
}

#[test]
fn test_remove_nonexistent_key() {
    let mut store = DedupStore::new();
    store.remove("nonexistent"); // should not panic
    assert!(store.is_empty());
}

#[test]
fn test_hash_for_key_after_register() {
    let mut store = DedupStore::new();
    store.register("k1", b"data");
    let hash = store.hash_for_key("k1").unwrap();
    let expected = DedupStore::compute_hash(b"data");
    assert_eq!(hash, &expected);
}

#[test]
fn test_keys_for_hash_returns_correct_set() {
    let mut store = DedupStore::new();
    store.register("k1", b"same");
    store.register("k2", b"same");
    store.register("k3", b"diff");

    let hash = DedupStore::compute_hash(b"same");
    let keys = store.keys_for_hash(&hash).unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains("k1"));
    assert!(keys.contains("k2"));
}

#[test]
fn test_keys_for_hash_nonexistent_returns_none() {
    let store = DedupStore::new();
    assert!(store.keys_for_hash("abc").is_none());
}

#[test]
fn test_re_register_same_content_same_key() {
    let mut store = DedupStore::new();
    store.register("k1", b"data");
    let result = store.register("k1", b"data");
    assert!(matches!(result, DedupResult::Unique { .. }));
    assert_eq!(store.total_key_count(), 1);
    assert_eq!(store.unique_content_count(), 1);
}

#[test]
fn test_duplicate_key_count_no_duplicates() {
    let mut store = DedupStore::new();
    store.register("k1", b"a");
    store.register("k2", b"b");
    assert_eq!(store.duplicate_key_count(), 0);
    assert_eq!(store.estimated_savings(100), 0);
}

#[test]
fn test_compute_hash_empty_content() {
    let hash = DedupStore::compute_hash(b"");
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}
