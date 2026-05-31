//! Unified Cache trait for HudHudScript (#819)
//!
//! This trait defines a common interface for all cache implementations across
//! the workspace. Crates that implement caching (parser, interpreter-core,
//! tokenomics, runtime, etc.) can implement this trait to ensure a consistent
//! API surface.
//!
//! ## Design Principles
//!
//! 1. **Generic**: Works with any key/value types
//! 2. **Bounded**: All caches have a configurable maximum size
//! 3. **Observable**: Statistics (hits, misses, evictions) are always available
//! 4. **Simple**: No async — see `AsyncCache` for async variants

use std::fmt;

/// Statistics for cache usage monitoring.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of evictions performed
    pub evictions: u64,
    /// Current number of entries
    pub size: usize,
    /// Maximum capacity
    pub capacity: usize,
}

impl CacheStats {
    /// Cache hit rate as a fraction [0.0, 1.0].
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl fmt::Display for CacheStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CacheStats {{ hits: {}, misses: {}, evictions: {}, size: {}/{}, hit_rate: {:.1}% }}",
            self.hits,
            self.misses,
            self.evictions,
            self.size,
            self.capacity,
            self.hit_rate() * 100.0,
        )
    }
}

/// Unified cache trait for synchronous caches.
///
/// All cache implementations in the workspace should implement this trait
/// to provide a consistent interface for get/put/remove/clear operations
/// and observability through statistics.
///
/// # Type Parameters
///
/// * `K` - The key type (typically `String` or `&str`)
/// * `V` - The value type
pub trait Cache<K, V> {
    /// Retrieve a value from the cache.
    ///
    /// Returns `Some(&V)` on cache hit, `None` on cache miss.
    /// Implementations should update hit/miss statistics.
    fn get(&self, key: &K) -> Option<&V>;

    /// Insert a key-value pair into the cache.
    ///
    /// If the cache is full, the implementation should evict entries
    /// according to its eviction policy (LRU, LFU, clear-all, etc.)
    fn put(&mut self, key: K, value: V);

    /// Remove a specific entry from the cache.
    ///
    /// Returns `true` if the entry was present and removed.
    fn remove(&mut self, key: &K) -> bool;

    /// Remove all entries from the cache.
    fn clear(&mut self);

    /// Number of entries currently stored.
    fn len(&self) -> usize;

    /// Whether the cache is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Maximum capacity of the cache.
    fn capacity(&self) -> usize;

    /// Get cache usage statistics.
    fn stats(&self) -> CacheStats;
}

/// Eviction policy for cache implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used — evict the entry that hasn't been accessed longest
    Lru,
    /// Least Frequently Used — evict the entry with fewest accesses
    Lfu,
    /// Clear all entries when capacity is reached (simplest, used by SimpleLruCache)
    ClearAll,
    /// Time-To-Live — evict entries older than a configured duration
    Ttl,
}
