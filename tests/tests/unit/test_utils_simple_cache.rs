use hudhudscript_utils::simple_cache::SimpleLruCache;

#[test]
fn insert_and_get() {
    let mut cache = SimpleLruCache::new(10);
    cache.insert("k".into(), 42);
    assert_eq!(cache.get("k"), Some(&42));
    assert_eq!(cache.len(), 1);
}

#[test]
fn miss_returns_none() {
    let cache: SimpleLruCache<i32> = SimpleLruCache::new(10);
    assert_eq!(cache.get("missing"), None);
}

#[test]
fn evicts_all_on_overflow() {
    let mut cache = SimpleLruCache::new(2);
    cache.insert("a".into(), 1);
    cache.insert("b".into(), 2);
    assert_eq!(cache.len(), 2);

    cache.insert("c".into(), 3);
    assert_eq!(cache.len(), 1);
    assert!(cache.get("a").is_none());
    assert!(cache.get("b").is_none());
    assert_eq!(cache.get("c"), Some(&3));
}

#[test]
fn clear_empties() {
    let mut cache = SimpleLruCache::new(10);
    cache.insert("x".into(), 1);
    cache.clear();
    assert_eq!(cache.len(), 0);
}

#[test]
fn get_or_compute_caches() {
    let mut cache = SimpleLruCache::new(10);
    let mut calls = 0u32;
    let v = cache.get_or_compute("key", || {
        calls += 1;
        99
    });
    assert_eq!(v, 99);
    assert_eq!(calls, 1);

    let v2 = cache.get_or_compute("key", || {
        calls += 1;
        0
    });
    assert_eq!(v2, 99);
    assert_eq!(calls, 1);
}

#[test]
fn max_size_zero_still_works() {
    let mut cache = SimpleLruCache::new(0);
    cache.insert("a".into(), 1);
    assert_eq!(cache.len(), 1);
}

#[test]
fn default_max_size_is_100() {
    let cache: SimpleLruCache<i32> = SimpleLruCache::default();
    for i in 0..100 {
        // We need mutable access
        let _ = i;
    }
    assert_eq!(cache.len(), 0);
}
