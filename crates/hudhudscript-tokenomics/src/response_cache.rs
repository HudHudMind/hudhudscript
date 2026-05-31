//! Response cache with exact match and semantic similarity support
//!
//! Caches LLM responses to avoid redundant API calls. Supports exact hash
//! matching, semantic similarity via embedding vectors, LRU eviction, and
//! TTL-based expiration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// A cached LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: u64,
    pub cached_at: DateTime<Utc>,
    pub ttl_seconds: u64,
    pub hit_count: u64,
    pub estimated_cost_usd: f64,
}

/// Aggregate cache statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub exact_hits: u64,
    pub semantic_hits: u64,
    pub total_cost_avoided: f64,
    pub evictions: u64,
}

/// Strategy used for cache lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheStrategy {
    /// Only exact hash matches.
    Exact,
    /// Only semantic similarity matches.
    Semantic,
    /// Try exact first, fall back to semantic.
    Hybrid,
}

/// An entry in the semantic cache, pairing an embedding vector with a response.
#[derive(Debug, Clone)]
struct SemanticEntry {
    embedding: Vec<f32>,
    _response: CachedResponse,
    key: u64,
}

/// Response cache supporting exact-match and semantic-similarity lookups.
#[derive(Debug)]
pub struct ResponseCache {
    exact: HashMap<u64, CachedResponse>,
    semantic: Vec<SemanticEntry>,
    access_order: Vec<u64>,
    max_entries: usize,
    strategy: CacheStrategy,
    semantic_threshold: f32,
    stats: CacheStats,
}

impl ResponseCache {
    /// Create a new response cache.
    pub fn new(max_entries: usize, strategy: CacheStrategy, semantic_threshold: f32) -> Self {
        Self {
            exact: HashMap::new(),
            semantic: Vec::new(),
            access_order: Vec::new(),
            max_entries,
            strategy,
            semantic_threshold,
            stats: CacheStats::default(),
        }
    }

    /// Compute a deterministic cache key from request parameters.
    pub fn cache_key(
        model: &str,
        prompt: &str,
        system: &str,
        temperature: f32,
        max_tokens: u64,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        model.hash(&mut hasher);
        prompt.hash(&mut hasher);
        system.hash(&mut hasher);
        temperature.to_bits().hash(&mut hasher);
        max_tokens.hash(&mut hasher);
        hasher.finish()
    }

    /// Look up a cached response by exact key or semantic similarity.
    pub fn get(&mut self, key: u64, embedding: Option<&[f32]>) -> Option<&CachedResponse> {
        // Evict expired entries lazily.
        self.evict_expired();

        match self.strategy {
            CacheStrategy::Exact => self.get_exact(key),
            CacheStrategy::Semantic => {
                if let Some(emb) = embedding {
                    self.get_semantic(emb)
                } else {
                    self.stats.misses += 1;
                    None
                }
            }
            CacheStrategy::Hybrid => {
                if self.exact.contains_key(&key) {
                    // Found exact — record stats and touch LRU.
                    self.stats.hits += 1;
                    self.stats.exact_hits += 1;
                    self.touch(key);
                    let entry = self.exact.get_mut(&key).unwrap();
                    entry.hit_count += 1;
                    self.stats.total_cost_avoided += entry.estimated_cost_usd;
                    return self.exact.get(&key);
                }
                if let Some(emb) = embedding {
                    return self.get_semantic(emb);
                }
                self.stats.misses += 1;
                None
            }
        }
    }

    fn get_exact(&mut self, key: u64) -> Option<&CachedResponse> {
        if self.exact.contains_key(&key) {
            self.stats.hits += 1;
            self.stats.exact_hits += 1;
            self.touch(key);
            let entry = self.exact.get_mut(&key).unwrap();
            entry.hit_count += 1;
            self.stats.total_cost_avoided += entry.estimated_cost_usd;
            self.exact.get(&key)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Find the best semantic match above the similarity threshold.
    pub fn get_semantic(&mut self, embedding: &[f32]) -> Option<&CachedResponse> {
        let mut best_score: f32 = -1.0;
        let mut best_key: Option<u64> = None;

        for entry in &self.semantic {
            let score = cosine_similarity(embedding, &entry.embedding);
            if score > best_score && score >= self.semantic_threshold {
                best_score = score;
                best_key = Some(entry.key);
            }
        }

        if let Some(key) = best_key {
            self.stats.hits += 1;
            self.stats.semantic_hits += 1;
            self.touch(key);
            if let Some(entry) = self.exact.get_mut(&key) {
                entry.hit_count += 1;
                self.stats.total_cost_avoided += entry.estimated_cost_usd;
            }
            self.exact.get(&key)
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a response into the exact cache.
    pub fn put(&mut self, key: u64, response: CachedResponse) {
        self.ensure_capacity();
        self.access_order.retain(|k| *k != key);
        self.access_order.push(key);
        self.exact.insert(key, response);
    }

    /// Insert a response with an associated embedding vector for semantic lookup.
    pub fn put_with_embedding(&mut self, key: u64, response: CachedResponse, embedding: Vec<f32>) {
        self.put(key, response);
        self.semantic.retain(|e| e.key != key);
        let resp_clone = self.exact.get(&key).unwrap().clone();
        self.semantic.push(SemanticEntry {
            embedding,
            _response: resp_clone,
            key,
        });
    }

    /// Remove all entries whose TTL has elapsed.
    pub fn evict_expired(&mut self) {
        let now = Utc::now();
        let expired_keys: Vec<u64> = self
            .exact
            .iter()
            .filter(|(_, v)| {
                let expiry = v.cached_at + chrono::Duration::seconds(v.ttl_seconds as i64);
                now > expiry
            })
            .map(|(k, _)| *k)
            .collect();

        for key in &expired_keys {
            self.exact.remove(key);
            self.semantic.retain(|e| e.key != *key);
            self.access_order.retain(|k| k != key);
            self.stats.evictions += 1;
        }
    }

    /// Return current cache statistics.
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all entries and reset statistics.
    pub fn clear(&mut self) {
        self.exact.clear();
        self.semantic.clear();
        self.access_order.clear();
        self.stats = CacheStats::default();
    }

    /// Number of entries in the exact cache.
    pub fn len(&self) -> usize {
        self.exact.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty()
    }

    // ── internal helpers ───────────────────────────────────────────

    fn touch(&mut self, key: u64) {
        self.access_order.retain(|k| *k != key);
        self.access_order.push(key);
    }

    fn ensure_capacity(&mut self) {
        while self.exact.len() >= self.max_entries {
            if let Some(oldest) = self.access_order.first().copied() {
                self.exact.remove(&oldest);
                self.semantic.retain(|e| e.key != oldest);
                self.access_order.remove(0);
                self.stats.evictions += 1;
            } else {
                break;
            }
        }
    }
}

/// Compute cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let x = *x as f64;
        let y = *y as f64;
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return 0.0;
    }
    (dot / denom) as f32
}
