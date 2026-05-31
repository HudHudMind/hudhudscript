//! Public API tests for tokenomics::response_cache
//! Covers: ResponseCache (new, cache_key, get, put, put_with_embedding,
//!         get_semantic, evict_expired, stats, clear, len, is_empty),
//!         CacheStrategy, CachedResponse, CacheStats, cosine_similarity.

use chrono::Utc;
use hudhudscript_tokenomics::response_cache::{
    cosine_similarity, CacheStats, CacheStrategy, CachedResponse, ResponseCache,
};

// ── helpers ─────────────────────────────────────────────────────────

fn make_response(content: &str, cost: f64) -> CachedResponse {
    CachedResponse {
        content: content.to_string(),
        model: "test-model".to_string(),
        tokens_used: 100,
        cached_at: Utc::now(),
        ttl_seconds: 3600,
        hit_count: 0,
        estimated_cost_usd: cost,
    }
}

fn expired_response(content: &str) -> CachedResponse {
    CachedResponse {
        content: content.to_string(),
        model: "test-model".to_string(),
        tokens_used: 50,
        cached_at: Utc::now() - chrono::Duration::seconds(10),
        ttl_seconds: 0, // expired immediately
        hit_count: 0,
        estimated_cost_usd: 0.01,
    }
}

// ── ResponseCache construction ───────────────────────────────────────

#[test]
fn cache_new_starts_empty() {
    let cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn cache_new_accepts_exact_strategy() {
    let cache = ResponseCache::new(5, CacheStrategy::Exact, 0.8);
    assert_eq!(cache.len(), 0);
}

#[test]
fn cache_new_accepts_semantic_strategy() {
    let cache = ResponseCache::new(5, CacheStrategy::Semantic, 0.8);
    assert!(cache.is_empty());
}

#[test]
fn cache_new_accepts_hybrid_strategy() {
    let cache = ResponseCache::new(5, CacheStrategy::Hybrid, 0.8);
    assert!(cache.is_empty());
}

// ── ResponseCache::cache_key ─────────────────────────────────────────

#[test]
fn cache_key_is_deterministic() {
    let k1 = ResponseCache::cache_key("gpt-4", "hi", "sys", 0.7, 100);
    let k2 = ResponseCache::cache_key("gpt-4", "hi", "sys", 0.7, 100);
    assert_eq!(k1, k2);
}

#[test]
fn cache_key_different_temperature_gives_different_key() {
    let k1 = ResponseCache::cache_key("gpt-4", "hi", "sys", 0.7, 100);
    let k2 = ResponseCache::cache_key("gpt-4", "hi", "sys", 0.8, 100);
    assert_ne!(k1, k2);
}

#[test]
fn cache_key_different_model_gives_different_key() {
    let k1 = ResponseCache::cache_key("gpt-4", "hi", "", 0.5, 256);
    let k2 = ResponseCache::cache_key("claude-3", "hi", "", 0.5, 256);
    assert_ne!(k1, k2);
}

#[test]
fn cache_key_different_prompt_gives_different_key() {
    let k1 = ResponseCache::cache_key("m", "hello", "", 0.0, 100);
    let k2 = ResponseCache::cache_key("m", "world", "", 0.0, 100);
    assert_ne!(k1, k2);
}

#[test]
fn cache_key_different_max_tokens_gives_different_key() {
    let k1 = ResponseCache::cache_key("m", "p", "s", 0.5, 100);
    let k2 = ResponseCache::cache_key("m", "p", "s", 0.5, 200);
    assert_ne!(k1, k2);
}

// ── ResponseCache::put / len / is_empty ──────────────────────────────

#[test]
fn put_increases_len() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("a", 0.01));
    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());
}

#[test]
fn put_multiple_entries() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("a", 0.01));
    cache.put(2, make_response("b", 0.02));
    cache.put(3, make_response("c", 0.03));
    assert_eq!(cache.len(), 3);
}

#[test]
fn put_same_key_does_not_increase_len() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(42, make_response("first", 0.01));
    cache.put(42, make_response("second", 0.02));
    assert_eq!(cache.len(), 1);
}

// ── ResponseCache::get (Exact strategy) ──────────────────────────────

#[test]
fn exact_hit_returns_response() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    let key = ResponseCache::cache_key("gpt-4", "hello", "", 0.7, 256);
    cache.put(key, make_response("world", 0.05));
    let resp = cache.get(key, None);
    assert!(resp.is_some());
    assert_eq!(resp.unwrap().content, "world");
}

#[test]
fn exact_miss_returns_none() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    let key = ResponseCache::cache_key("gpt-4", "hello", "", 0.7, 256);
    let miss = cache.get(key, None);
    assert!(miss.is_none());
}

#[test]
fn exact_hit_increments_hits_stat() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    let key = 1u64;
    cache.put(key, make_response("v", 0.01));
    let _ = cache.get(key, None);
    assert_eq!(cache.stats().hits, 1);
    assert_eq!(cache.stats().exact_hits, 1);
}

#[test]
fn exact_miss_increments_misses_stat() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    let _ = cache.get(999, None);
    assert_eq!(cache.stats().misses, 1);
}

#[test]
fn exact_hit_tracks_cost_avoided() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    let key = 100u64;
    cache.put(key, make_response("v", 0.25));
    let _ = cache.get(key, None);
    let _ = cache.get(key, None);
    assert!((cache.stats().total_cost_avoided - 0.50).abs() < 1e-9);
}

// ── ResponseCache::get (Semantic strategy) ───────────────────────────

#[test]
fn semantic_hit_above_threshold() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Semantic, 0.9);
    let emb_a = vec![1.0f32, 0.0, 0.0];
    let emb_b = vec![0.99f32, 0.1, 0.0]; // very similar
    cache.put_with_embedding(1, make_response("semantic hit", 0.10), emb_a);
    let result = cache.get(999, Some(&emb_b));
    assert!(result.is_some());
    assert_eq!(result.unwrap().content, "semantic hit");
    assert_eq!(cache.stats().semantic_hits, 1);
}

#[test]
fn semantic_miss_below_threshold() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Semantic, 0.99);
    let emb_a = vec![1.0f32, 0.0, 0.0];
    let emb_b = vec![0.0f32, 1.0, 0.0]; // orthogonal
    cache.put_with_embedding(1, make_response("no match", 0.10), emb_a);
    let result = cache.get(999, Some(&emb_b));
    assert!(result.is_none());
    assert_eq!(cache.stats().misses, 1);
}

#[test]
fn semantic_strategy_no_embedding_counts_as_miss() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Semantic, 0.9);
    cache.put(1, make_response("value", 0.05));
    let result = cache.get(1, None); // no embedding
    assert!(result.is_none());
    assert_eq!(cache.stats().misses, 1);
}

// ── ResponseCache::get (Hybrid strategy) ─────────────────────────────

#[test]
fn hybrid_exact_match_wins() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Hybrid, 0.9);
    let key = 50u64;
    cache.put(key, make_response("hybrid-exact", 0.05));
    let resp = cache.get(key, None);
    assert!(resp.is_some());
    assert_eq!(resp.unwrap().content, "hybrid-exact");
    assert_eq!(cache.stats().exact_hits, 1);
}

#[test]
fn hybrid_exact_miss_falls_back_to_semantic() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Hybrid, 0.85);
    let emb_stored = vec![1.0f32, 0.0, 0.0];
    cache.put_with_embedding(100, make_response("semantic answer", 0.10), emb_stored);

    let emb_query = vec![0.98f32, 0.15, 0.0]; // similar to stored
    let result = cache.get(999, Some(&emb_query));
    assert!(
        result.is_some(),
        "hybrid should fall back to semantic on exact miss"
    );
    assert_eq!(result.unwrap().content, "semantic answer");
    assert_eq!(cache.stats().semantic_hits, 1);
    assert_eq!(cache.stats().exact_hits, 0);
}

#[test]
fn hybrid_total_miss_counts_as_miss() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Hybrid, 0.99);
    let emb_a = vec![1.0f32, 0.0, 0.0];
    cache.put_with_embedding(1, make_response("x", 0.01), emb_a);
    let emb_q = vec![0.0f32, 1.0, 0.0]; // orthogonal
    let result = cache.get(999, Some(&emb_q));
    assert!(result.is_none());
    assert_eq!(cache.stats().misses, 1);
}

#[test]
fn hybrid_exact_miss_no_embedding_is_a_miss() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Hybrid, 0.9);
    cache.put(1, make_response("x", 0.01));
    // key miss and no embedding
    let result = cache.get(999, None);
    assert!(result.is_none());
    assert_eq!(cache.stats().misses, 1);
}

// ── ResponseCache LRU eviction ────────────────────────────────────────

#[test]
fn lru_eviction_removes_oldest_entry() {
    let mut cache = ResponseCache::new(2, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("a", 0.01));
    cache.put(2, make_response("b", 0.02));
    // Touch key 1 to make it most recently used
    let _ = cache.get(1, None);
    // Insert key 3: should evict key 2 (LRU)
    cache.put(3, make_response("c", 0.03));
    assert_eq!(cache.len(), 2);
    assert!(
        cache.get(2, None).is_none(),
        "key 2 should have been evicted"
    );
    assert!(cache.get(1, None).is_some());
    assert!(cache.get(3, None).is_some());
}

#[test]
fn lru_eviction_increments_evictions_stat() {
    let mut cache = ResponseCache::new(1, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("a", 0.01));
    cache.put(2, make_response("b", 0.02)); // evicts 1
    assert_eq!(cache.stats().evictions, 1);
}

// ── ResponseCache::evict_expired ─────────────────────────────────────

#[test]
fn evict_expired_removes_expired_entries() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(1, expired_response("expired"));
    assert_eq!(cache.len(), 1);
    cache.evict_expired();
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.stats().evictions, 1);
}

#[test]
fn evict_expired_does_not_remove_valid_entries() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("live", 0.01)); // ttl=3600
    cache.evict_expired();
    assert_eq!(cache.len(), 1);
}

#[test]
fn evict_expired_removes_only_expired_entries() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("live", 0.01)); // ttl=3600
    cache.put(2, expired_response("expired"));
    cache.evict_expired();
    assert_eq!(cache.len(), 1);
    assert!(cache.get(1, None).is_some());
}

// ── ResponseCache::clear ──────────────────────────────────────────────

#[test]
fn clear_removes_all_entries() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("a", 0.01));
    cache.put(2, make_response("b", 0.02));
    assert_eq!(cache.len(), 2);
    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn clear_resets_all_stats() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Exact, 0.9);
    cache.put(1, make_response("a", 0.05));
    let _ = cache.get(1, None);
    cache.clear();
    assert_eq!(cache.stats().hits, 0);
    assert_eq!(cache.stats().misses, 0);
    assert_eq!(cache.stats().total_cost_avoided, 0.0);
}

// ── CacheStats ───────────────────────────────────────────────────────

#[test]
fn cache_stats_default_all_zeros() {
    let stats = CacheStats::default();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.exact_hits, 0);
    assert_eq!(stats.semantic_hits, 0);
    assert_eq!(stats.evictions, 0);
    assert_eq!(stats.total_cost_avoided, 0.0);
}

#[test]
fn cache_stats_clone_and_debug() {
    let stats = CacheStats {
        hits: 5,
        misses: 3,
        ..CacheStats::default()
    };
    let cloned = stats.clone();
    assert_eq!(cloned.hits, 5);
    let dbg = format!("{:?}", stats);
    assert!(dbg.contains("CacheStats"));
}

// ── cosine_similarity ────────────────────────────────────────────────

#[test]
fn cosine_similarity_identical_vectors_is_one() {
    let a = vec![1.0f32, 2.0, 3.0];
    let result = cosine_similarity(&a, &a);
    assert!((result - 1.0).abs() < 1e-6);
}

#[test]
fn cosine_similarity_orthogonal_vectors_is_zero() {
    let a = vec![1.0f32, 0.0];
    let b = vec![0.0f32, 1.0];
    assert!(cosine_similarity(&a, &b).abs() < 1e-6);
}

#[test]
fn cosine_similarity_empty_vectors_is_zero() {
    let result = cosine_similarity(&[], &[]);
    assert_eq!(result, 0.0);
}

#[test]
fn cosine_similarity_mismatched_lengths_is_zero() {
    let a = vec![1.0f32, 0.0];
    let b = vec![1.0f32, 0.0, 0.0];
    assert_eq!(cosine_similarity(&a, &b), 0.0);
}

#[test]
fn cosine_similarity_zero_norm_vector_is_zero() {
    let a = vec![0.0f32, 0.0, 0.0];
    let b = vec![1.0f32, 2.0, 3.0];
    assert_eq!(cosine_similarity(&a, &b), 0.0);
}

#[test]
fn cosine_similarity_opposite_vectors_is_negative_one() {
    let a = vec![1.0f32, 0.0];
    let b = vec![-1.0f32, 0.0];
    let result = cosine_similarity(&a, &b);
    assert!((result + 1.0).abs() < 1e-6);
}

#[test]
fn cosine_similarity_partial_overlap() {
    let a = vec![1.0f32, 1.0, 0.0];
    let b = vec![1.0f32, 0.0, 0.0];
    let result = cosine_similarity(&a, &b);
    // cos(45 degrees) ≈ 0.7071
    assert!((result - 0.7071).abs() < 0.001);
}

// ── CachedResponse serde ─────────────────────────────────────────────

#[test]
fn cached_response_serde_roundtrip() {
    let resp = make_response("hello world", 0.05);
    let json = serde_json::to_string(&resp).unwrap();
    let back: CachedResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(back.content, "hello world");
    assert!((back.estimated_cost_usd - 0.05).abs() < 1e-12);
    assert_eq!(back.tokens_used, 100);
}

#[test]
fn cached_response_clone() {
    let resp = make_response("test", 0.1);
    let cloned = resp.clone();
    assert_eq!(cloned.content, "test");
}

#[test]
fn cached_response_debug_format() {
    let resp = make_response("debug test", 0.02);
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("CachedResponse"));
}

// ── put_with_embedding ───────────────────────────────────────────────

#[test]
fn put_with_embedding_allows_semantic_lookup() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Semantic, 0.85);
    let emb = vec![1.0f32, 0.0, 0.0];
    cache.put_with_embedding(1, make_response("embedded", 0.05), emb);
    assert_eq!(cache.len(), 1);
    // Look up with same embedding
    let result = cache.get(999, Some(&[1.0f32, 0.0, 0.0]));
    assert!(result.is_some());
}

#[test]
fn put_with_embedding_overwrites_previous_embedding_for_same_key() {
    let mut cache = ResponseCache::new(10, CacheStrategy::Semantic, 0.85);
    cache.put_with_embedding(1, make_response("v1", 0.01), vec![1.0f32, 0.0]);
    cache.put_with_embedding(1, make_response("v2", 0.02), vec![0.0f32, 1.0]);
    assert_eq!(cache.len(), 1);
    // Now look up with the new embedding
    let result = cache.get(999, Some(&[0.0f32, 1.0]));
    assert!(result.is_some());
    assert_eq!(result.unwrap().content, "v2");
}
