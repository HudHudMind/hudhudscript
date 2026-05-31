//! Simple cache eviction utilities for bounded HashMap caches.
//!
//! Used by both VM and interpreter to prevent unbounded memory growth
//! in long-running processes (Issue #979).
//!
//! Strategy: when a cache exceeds its size limit, evict entries down to
//! half the limit. Since `HashMap` does not track insertion order, eviction
//! is arbitrary — effectively random eviction, which is still far better
//! than unbounded growth.

use std::collections::HashMap;
use std::hash::Hash;

/// Default maximum cache size for module caches.
pub const MAX_MODULE_CACHE: usize = 256;

/// Default maximum cache size for MCP client/tool caches.
pub const MAX_MCP_CACHE: usize = 64;

/// Default maximum cache size for RAG/vector stores.
pub const MAX_RAG_STORE_CACHE: usize = 128;

/// Default maximum cache size for plugin registries.
pub const MAX_PLUGIN_CACHE: usize = 128;

/// Enforce a size limit on a `HashMap`. If the map exceeds `max_size`,
/// arbitrary entries are removed until the map contains at most `max_size / 2`
/// entries.
///
/// Returns the number of entries evicted. Generic over the hasher so both
/// `std::collections::HashMap` and `rustc_hash::FxHashMap` work (PERF-28).
pub fn enforce_cache_limit<K: Eq + Hash, V, S: std::hash::BuildHasher>(
    map: &mut HashMap<K, V, S>,
    max_size: usize,
) -> usize {
    if map.len() <= max_size {
        return 0;
    }
    let target = max_size / 2;
    let keep = target.min(map.len());
    // Drain everything, then re-insert only `keep` entries.
    let entries: Vec<(K, V)> = map.drain().collect();
    let evicted = entries.len() - keep;
    for (k, v) in entries.into_iter().take(keep) {
        map.insert(k, v);
    }
    evicted
}
