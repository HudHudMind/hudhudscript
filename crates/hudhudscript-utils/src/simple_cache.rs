//! A simple bounded cache with clear-on-overflow eviction.
//!
//! When the number of entries reaches `max_size`, the entire cache is cleared
//! before inserting the new entry.  This is intentionally simple — callers that
//! need true LRU or TTL semantics should use the `hudhudscript-cache` crate.

use std::collections::HashMap;

/// A generic, bounded, in-memory cache.
///
/// Eviction policy: when the cache is full (`len() >= max_size`), **all**
/// entries are cleared before the next insert.  This keeps the implementation
/// allocation-free at steady state while bounding memory usage.
pub struct SimpleLruCache<V> {
    map: HashMap<String, V>,
    max_size: usize,
}

impl<V> SimpleLruCache<V> {
    /// Create a new cache that holds at most `max_size` entries.
    pub fn new(max_size: usize) -> Self {
        Self {
            map: HashMap::new(),
            max_size,
        }
    }

    /// Insert a key-value pair.
    ///
    /// If the cache has reached `max_size`, all existing entries are cleared
    /// first.
    pub fn insert(&mut self, key: String, value: V) {
        if self.map.len() >= self.max_size {
            self.map.clear();
        }
        self.map.insert(key, value);
    }

    /// Look up a value by key.
    pub fn get(&self, key: &str) -> Option<&V> {
        self.map.get(key)
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl<V: Clone> SimpleLruCache<V> {
    /// Return a cached value or compute, cache, and return it.
    pub fn get_or_compute<F>(&mut self, key: &str, compute: F) -> V
    where
        F: FnOnce() -> V,
    {
        if let Some(value) = self.map.get(key) {
            return value.clone();
        }
        let value = compute();
        self.insert(key.to_string(), value.clone());
        value
    }
}

impl<V> Default for SimpleLruCache<V> {
    fn default() -> Self {
        Self::new(100)
    }
}
