//! Response Caching — Issue #630
//!
//! Provides a TTL-based, LRU-evicting cache for LLM responses.
//! Keyed by (model, messages hash, temperature, max_tokens) so that
//! identical requests within the TTL window are served from memory.

use crate::provider::LLMResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

// ---------------------------------------------------------------------------
// Cache key
// ---------------------------------------------------------------------------

/// A deterministic key derived from the request parameters that affect the
/// response.  Two requests with the same `CacheKey` are expected to produce
/// equivalent responses (modulo non-determinism at temperature > 0).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Model identifier (e.g. `"gpt-4o"`).
    pub model: String,
    /// A hash of the full message sequence (system + user prompts).
    pub messages_hash: u64,
    /// Temperature quantised to 2 decimal places (stored as `(temp * 100) as i32`).
    pub temperature_centis: i32,
    /// Max tokens requested.
    pub max_tokens: Option<usize>,
}

impl CacheKey {
    /// Build a cache key from request components.
    ///
    /// `messages_content` should be a concatenation of all message contents
    /// (system prompt + user prompt) that participate in the hash.
    pub fn new(
        model: impl Into<String>,
        messages_content: &str,
        temperature: Option<f64>,
        max_tokens: Option<usize>,
    ) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        messages_content.hash(&mut hasher);
        let messages_hash = hasher.finish();

        Self {
            model: model.into(),
            messages_hash,
            temperature_centis: temperature.map(|t| (t * 100.0) as i32).unwrap_or(70), // default 0.70
            max_tokens,
        }
    }
}

// ---------------------------------------------------------------------------
// Cached response
// ---------------------------------------------------------------------------

/// A cached LLM response with metadata.
#[derive(Debug, Clone)]
pub struct CachedResponse {
    /// The cached response.
    pub response: LLMResponse,
    /// When the response was inserted.
    pub created_at: Instant,
    /// Number of cache hits for this entry.
    pub hit_count: u64,
    /// Last time this entry was accessed.
    pub last_accessed: Instant,
}

// ---------------------------------------------------------------------------
// Cache statistics
// ---------------------------------------------------------------------------

/// Aggregate cache statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of `get()` calls.
    pub total_lookups: u64,
    /// Number of cache hits.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
    /// Estimated tokens saved by cache hits.
    pub saved_tokens: u64,
    /// Current number of entries in the cache.
    pub entries: usize,
    /// Number of entries evicted due to size or TTL.
    pub evictions: u64,
}

impl CacheStats {
    /// Cache hit rate as a fraction (0.0 .. 1.0).
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_lookups as f64
        }
    }
}

// ---------------------------------------------------------------------------
// Cache configuration
// ---------------------------------------------------------------------------

/// Configuration for `ResponseCache`.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries before LRU eviction kicks in.
    pub max_entries: usize,
    /// Time-to-live for each cache entry.
    pub ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }
}

// ---------------------------------------------------------------------------
// ResponseCache
// ---------------------------------------------------------------------------

/// A thread-safe, TTL-based LRU cache for LLM responses.
pub struct ResponseCache {
    entries: Arc<RwLock<HashMap<CacheKey, CachedResponse>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl ResponseCache {
    /// Create a new cache with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Look up a cached response.
    ///
    /// Returns `None` if the key is not present or the entry has expired.
    pub async fn get(&self, key: &CacheKey) -> Option<LLMResponse> {
        let mut stats = self.stats.write().await;
        stats.total_lookups += 1;

        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            // Check TTL.
            if entry.created_at.elapsed() > self.config.ttl {
                entries.remove(key);
                stats.misses += 1;
                stats.entries = entries.len();
                debug!("cache: expired entry for model '{}'", key.model);
                return None;
            }

            entry.hit_count += 1;
            entry.last_accessed = Instant::now();
            stats.hits += 1;
            stats.saved_tokens += entry.response.tokens_used.total_tokens as u64;
            debug!(
                "cache: hit for model '{}' (hits={})",
                key.model, entry.hit_count
            );
            Some(entry.response.clone())
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Insert a response into the cache.
    ///
    /// If the cache is at capacity, the least-recently-used entry is evicted.
    pub async fn put(&self, key: CacheKey, response: LLMResponse) {
        let mut entries = self.entries.write().await;

        // Evict expired entries first.
        let ttl = self.config.ttl;
        let before = entries.len();
        entries.retain(|_, v| v.created_at.elapsed() <= ttl);
        let expired = before - entries.len();

        // LRU eviction if still at capacity.
        let mut evicted = expired as u64;
        while entries.len() >= self.config.max_entries {
            if let Some(lru_key) = entries
                .iter()
                .min_by_key(|(_, v)| v.last_accessed)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&lru_key);
                evicted += 1;
            } else {
                break;
            }
        }

        let now = Instant::now();
        entries.insert(
            key,
            CachedResponse {
                response,
                created_at: now,
                hit_count: 0,
                last_accessed: now,
            },
        );

        // Update stats.
        let mut stats = self.stats.write().await;
        stats.entries = entries.len();
        stats.evictions += evicted;
    }

    /// Remove a specific entry from the cache.
    pub async fn invalidate(&self, key: &CacheKey) -> bool {
        let mut entries = self.entries.write().await;
        let removed = entries.remove(key).is_some();
        if removed {
            let mut stats = self.stats.write().await;
            stats.entries = entries.len();
        }
        removed
    }

    /// Remove all entries from the cache.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        let mut stats = self.stats.write().await;
        stats.entries = 0;
    }

    /// Return current cache statistics.
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}
