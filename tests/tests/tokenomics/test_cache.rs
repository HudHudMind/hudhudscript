//! Tests for tokenomics::cache — TokenomicsCache and InMemoryCacheBackend

use hudhudscript_tokenomics::cache::*;

// ---------------------------------------------------------------------------
// InMemoryCacheBackend — construction
// ---------------------------------------------------------------------------

#[test]
fn test_in_memory_backend_new() {
    let backend = InMemoryCacheBackend::new();
    assert!(format!("{:?}", backend).contains("InMemoryCacheBackend"));
}

#[test]
fn test_in_memory_backend_default() {
    let backend = InMemoryCacheBackend::default();
    assert!(format!("{:?}", backend).contains("InMemoryCacheBackend"));
}

// ---------------------------------------------------------------------------
// InMemoryCacheBackend — raw operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_backend_set_and_get_raw() {
    let backend = InMemoryCacheBackend::new();
    backend
        .set_raw("key1", "value1", std::time::Duration::from_secs(60))
        .await
        .unwrap();
    let result = backend.get_raw("key1").await.unwrap();
    assert_eq!(result, Some("value1".to_string()));
}

#[tokio::test]
async fn test_backend_get_raw_missing_key() {
    let backend = InMemoryCacheBackend::new();
    let result = backend.get_raw("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_backend_invalidate() {
    let backend = InMemoryCacheBackend::new();
    backend
        .set_raw("k", "v", std::time::Duration::from_secs(60))
        .await
        .unwrap();
    backend.invalidate("k").await.unwrap();
    let result = backend.get_raw("k").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_backend_invalidate_nonexistent() {
    let backend = InMemoryCacheBackend::new();
    // Invalidating a key that doesn't exist should succeed silently
    backend.invalidate("ghost").await.unwrap();
}

#[tokio::test]
async fn test_backend_expired_key_removal() {
    let backend = InMemoryCacheBackend::new();
    backend
        .set_raw("expiring", "value", std::time::Duration::from_secs(0))
        .await
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let result = backend.get_raw("expiring").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_backend_overwrite_key() {
    let backend = InMemoryCacheBackend::new();
    backend
        .set_raw("k", "v1", std::time::Duration::from_secs(60))
        .await
        .unwrap();
    backend
        .set_raw("k", "v2", std::time::Duration::from_secs(60))
        .await
        .unwrap();
    let result = backend.get_raw("k").await.unwrap();
    assert_eq!(result, Some("v2".to_string()));
}

#[tokio::test]
async fn test_backend_multiple_keys() {
    let backend = InMemoryCacheBackend::new();
    for i in 0..5 {
        backend
            .set_raw(
                &format!("k{}", i),
                &format!("v{}", i),
                std::time::Duration::from_secs(60),
            )
            .await
            .unwrap();
    }
    for i in 0..5 {
        let result = backend.get_raw(&format!("k{}", i)).await.unwrap();
        assert_eq!(result, Some(format!("v{}", i)));
    }
}

// ---------------------------------------------------------------------------
// TokenomicsCache — construction
// ---------------------------------------------------------------------------

#[test]
fn test_cache_new_none_url() {
    let _cache = TokenomicsCache::new(None);
}

#[test]
fn test_cache_new_some_url() {
    // Even with a URL, it uses in-memory backend by default
    let _cache = TokenomicsCache::new(Some("redis://localhost".into()));
}

#[tokio::test]
async fn test_cache_with_backend() {
    let backend = Box::new(InMemoryCacheBackend::new());
    let cache = TokenomicsCache::with_backend(backend);
    cache.set("key", &"value", 60).await.unwrap();
    let val: Option<String> = cache.get("key").await.unwrap();
    assert_eq!(val, Some("value".to_string()));
}

// ---------------------------------------------------------------------------
// TokenomicsCache — typed set/get
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cache_set_and_get_u64() {
    let cache = TokenomicsCache::new(None);
    cache.set("k1", &42u64, 60).await.unwrap();
    let val: Option<u64> = cache.get("k1").await.unwrap();
    assert_eq!(val, Some(42));
}

#[tokio::test]
async fn test_cache_set_and_get_string() {
    let cache = TokenomicsCache::new(None);
    cache.set("k2", &"hello".to_string(), 60).await.unwrap();
    let val: Option<String> = cache.get("k2").await.unwrap();
    assert_eq!(val, Some("hello".to_string()));
}

#[tokio::test]
async fn test_cache_set_and_get_bool() {
    let cache = TokenomicsCache::new(None);
    cache.set("flag", &true, 60).await.unwrap();
    let val: Option<bool> = cache.get("flag").await.unwrap();
    assert_eq!(val, Some(true));
}

#[tokio::test]
async fn test_cache_set_and_get_vec() {
    let cache = TokenomicsCache::new(None);
    let data = vec![1u32, 2, 3, 4, 5];
    cache.set("vec", &data, 60).await.unwrap();
    let val: Option<Vec<u32>> = cache.get("vec").await.unwrap();
    assert_eq!(val, Some(vec![1, 2, 3, 4, 5]));
}

#[tokio::test]
async fn test_cache_set_and_get_struct() {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Point {
        x: f64,
        y: f64,
    }
    let cache = TokenomicsCache::new(None);
    cache
        .set("pt", &Point { x: 1.5, y: 2.5 }, 60)
        .await
        .unwrap();
    let val: Option<Point> = cache.get("pt").await.unwrap();
    assert_eq!(val, Some(Point { x: 1.5, y: 2.5 }));
}

// ---------------------------------------------------------------------------
// TokenomicsCache — missing key / invalidate
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cache_get_missing_key() {
    let cache = TokenomicsCache::new(None);
    let val: Option<String> = cache.get("nonexistent").await.unwrap();
    assert!(val.is_none());
}

#[tokio::test]
async fn test_cache_invalidate() {
    let cache = TokenomicsCache::new(None);
    cache.set("k", &"hello", 60).await.unwrap();
    cache.invalidate("k").await.unwrap();
    let val: Option<String> = cache.get("k").await.unwrap();
    assert!(val.is_none());
}

#[tokio::test]
async fn test_cache_invalidate_nonexistent() {
    let cache = TokenomicsCache::new(None);
    // Should not error
    cache.invalidate("ghost").await.unwrap();
}

#[tokio::test]
async fn test_cache_overwrite() {
    let cache = TokenomicsCache::new(None);
    cache.set("k", &1u64, 60).await.unwrap();
    cache.set("k", &2u64, 60).await.unwrap();
    let val: Option<u64> = cache.get("k").await.unwrap();
    assert_eq!(val, Some(2));
}

// ---------------------------------------------------------------------------
// TokenomicsCache — expiry
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cache_expired_entry() {
    let cache = TokenomicsCache::new(None);
    cache.set("exp", &999u64, 0).await.unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let val: Option<u64> = cache.get("exp").await.unwrap();
    assert!(val.is_none());
}

#[tokio::test]
async fn test_cache_non_expired_entry() {
    let cache = TokenomicsCache::new(None);
    cache.set("alive", &42u64, 300).await.unwrap();
    let val: Option<u64> = cache.get("alive").await.unwrap();
    assert_eq!(val, Some(42));
}
