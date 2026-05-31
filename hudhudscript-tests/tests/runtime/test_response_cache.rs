use hudhudscript_runtime::provider::{LLMResponse, TokenUsage};
use hudhudscript_runtime::response_cache::{CacheConfig, CacheKey, ResponseCache};
use std::time::Duration;

fn make_response(content: &str) -> LLMResponse {
    LLMResponse {
        content: content.to_string(),
        tokens_used: TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
        model: "test-model".to_string(),
        finish_reason: "stop".to_string(),
        tool_calls: None,
    }
}

fn make_key(prompt: &str) -> CacheKey {
    CacheKey::new("test-model", prompt, Some(0.7), Some(100))
}

#[tokio::test]
async fn test_put_and_get() {
    let cache = ResponseCache::default();
    let key = make_key("hello");
    let resp = make_response("world");

    cache.put(key.clone(), resp.clone()).await;
    let cached = cache.get(&key).await;

    assert!(cached.is_some());
    assert_eq!(cached.unwrap().content, "world");
}

#[tokio::test]
async fn test_cache_miss() {
    let cache = ResponseCache::default();
    let key = make_key("nonexistent");
    assert!(cache.get(&key).await.is_none());
}

#[tokio::test]
async fn test_ttl_expiration() {
    let cache = ResponseCache::new(CacheConfig {
        max_entries: 100,
        ttl: Duration::from_millis(50),
    });

    let key = make_key("ephemeral");
    cache.put(key.clone(), make_response("temp")).await;

    // Should be present immediately.
    assert!(cache.get(&key).await.is_some());

    // Wait for TTL to expire.
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(cache.get(&key).await.is_none());
}

#[tokio::test]
async fn test_lru_eviction() {
    let cache = ResponseCache::new(CacheConfig {
        max_entries: 2,
        ttl: Duration::from_secs(60),
    });

    let k1 = make_key("first");
    let k2 = make_key("second");
    let k3 = make_key("third");

    cache.put(k1.clone(), make_response("1")).await;
    cache.put(k2.clone(), make_response("2")).await;

    // Access k1 to make it recently used.
    cache.get(&k1).await;

    // Adding k3 should evict k2 (least recently used).
    cache.put(k3.clone(), make_response("3")).await;

    assert!(cache.get(&k1).await.is_some());
    assert!(cache.get(&k2).await.is_none());
    assert!(cache.get(&k3).await.is_some());
}

#[tokio::test]
async fn test_invalidate() {
    let cache = ResponseCache::default();
    let key = make_key("removable");

    cache.put(key.clone(), make_response("bye")).await;
    assert!(cache.get(&key).await.is_some());

    let removed = cache.invalidate(&key).await;
    assert!(removed);
    assert!(cache.get(&key).await.is_none());

    // Invalidating again returns false.
    assert!(!cache.invalidate(&key).await);
}

#[tokio::test]
async fn test_clear() {
    let cache = ResponseCache::default();
    cache.put(make_key("a"), make_response("1")).await;
    cache.put(make_key("b"), make_response("2")).await;

    cache.clear().await;
    let stats = cache.stats().await;
    assert_eq!(stats.entries, 0);
}

#[tokio::test]
async fn test_stats() {
    let cache = ResponseCache::default();
    let key = make_key("stats-test");
    cache.put(key.clone(), make_response("data")).await;

    // Hit.
    cache.get(&key).await;
    // Miss.
    cache.get(&make_key("missing")).await;

    let stats = cache.stats().await;
    assert_eq!(stats.total_lookups, 2);
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.entries, 1);
    assert!((stats.hit_rate() - 0.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_hit_count_increments() {
    let cache = ResponseCache::default();
    let key = make_key("popular");
    cache.put(key.clone(), make_response("data")).await;

    for _ in 0..5 {
        cache.get(&key).await;
    }

    let stats = cache.stats().await;
    assert_eq!(stats.hits, 5);
}

#[tokio::test]
async fn test_cache_key_determinism() {
    let k1 = CacheKey::new("gpt-4", "hello world", Some(0.7), Some(100));
    let k2 = CacheKey::new("gpt-4", "hello world", Some(0.7), Some(100));
    assert_eq!(k1, k2);

    let k3 = CacheKey::new("gpt-4", "different", Some(0.7), Some(100));
    assert_ne!(k1, k3);
}

#[tokio::test]
async fn test_different_temperature_different_key() {
    let k1 = CacheKey::new("gpt-4", "hello", Some(0.0), Some(100));
    let k2 = CacheKey::new("gpt-4", "hello", Some(1.0), Some(100));
    assert_ne!(k1, k2);
}
