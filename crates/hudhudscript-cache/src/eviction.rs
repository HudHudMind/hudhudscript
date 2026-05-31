//! Eviction policy engine for cache lifecycle management
//!
//! Provides LRU (Least Recently Used) and LFU (Least Frequently Used) eviction
//! policies that can be applied to cache entries. The eviction engine tracks access
//! patterns and determines which entries should be removed when the cache exceeds
//! its capacity.

use hashlink::LinkedHashSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Eviction policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EvictionPolicy {
    /// Least Recently Used — evicts entries that have not been accessed for the longest time
    #[default]
    Lru,

    /// Least Frequently Used — evicts entries with the lowest access count
    Lfu,
}

/// Metadata tracked per cache entry for eviction decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionEntry {
    /// Unique key identifying the cache entry
    pub key: String,

    /// Number of times this entry has been accessed
    pub access_count: u64,

    /// Monotonically increasing counter value at last access (used for LRU ordering)
    pub last_access_tick: u64,
}

/// Eviction engine that tracks access patterns and selects victims for removal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionEngine {
    /// Active eviction policy
    policy: EvictionPolicy,

    /// Per-key eviction metadata
    entries: HashMap<String, EvictionEntry>,

    /// Monotonically increasing tick counter (incremented on every access)
    tick: u64,

    /// LRU ordering queue (least-recently-used at front, most-recently-used
    /// at back).  Only maintained when policy is `Lru`.  `LinkedHashSet`
    /// gives O(1) insert/remove/touch via an intrusive doubly-linked list
    /// indexed by a HashMap — replaces the previous `VecDeque<String>`
    /// whose `.position() + .remove()` touch pattern was O(n).
    /// Audit v3 Finding 8.1.
    lru_order: LinkedHashSet<String>,
}

impl EvictionEngine {
    /// Create a new eviction engine with the given policy
    pub fn new(policy: EvictionPolicy) -> Self {
        Self {
            policy,
            entries: HashMap::new(),
            tick: 0,
            lru_order: LinkedHashSet::new(),
        }
    }

    /// Return the active eviction policy
    pub fn policy(&self) -> EvictionPolicy {
        self.policy
    }

    /// Record an access (insert or lookup) for the given key.
    ///
    /// If the key is new it will be registered; if it already exists its
    /// access metadata will be updated.
    pub fn record_access(&mut self, key: &str) {
        self.tick += 1;
        let current_tick = self.tick;

        if let Some(entry) = self.entries.get_mut(key) {
            entry.access_count += 1;
            entry.last_access_tick = current_tick;
        } else {
            self.entries.insert(
                key.to_string(),
                EvictionEntry {
                    key: key.to_string(),
                    access_count: 1,
                    last_access_tick: current_tick,
                },
            );
        }

        // Maintain LRU queue — remove then insert to move to back.
        if self.policy == EvictionPolicy::Lru {
            self.lru_order.remove(key);
            self.lru_order.insert(key.to_string());
        }
    }

    /// Remove tracking data for the given key (e.g. after the entry was evicted)
    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
        if self.policy == EvictionPolicy::Lru {
            // O(1) remove via LinkedHashSet.
            self.lru_order.remove(key);
        }
    }

    /// Select the next eviction victim according to the active policy.
    ///
    /// Returns `None` if the engine has no tracked entries.
    pub fn select_victim(&self) -> Option<&str> {
        match self.policy {
            EvictionPolicy::Lru => self.select_lru_victim(),
            EvictionPolicy::Lfu => self.select_lfu_victim(),
        }
    }

    /// Select up to `count` eviction victims, ordered from least-valuable to
    /// most-valuable according to the active policy.
    pub fn select_victims(&self, count: usize) -> Vec<&str> {
        match self.policy {
            EvictionPolicy::Lru => self.select_lru_victims(count),
            EvictionPolicy::Lfu => self.select_lfu_victims(count),
        }
    }

    /// Number of entries being tracked
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the engine has no tracked entries
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all tracking data
    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru_order.clear();
        self.tick = 0;
    }

    /// Return the access count for a given key, or `None` if untracked
    pub fn access_count(&self, key: &str) -> Option<u64> {
        self.entries.get(key).map(|e| e.access_count)
    }

    // --- LRU helpers ---

    fn select_lru_victim(&self) -> Option<&str> {
        // Front = least-recently-used.  LinkedHashSet iterates in insertion
        // order; `front()` returns the head.
        self.lru_order.front().map(|s| s.as_str())
    }

    fn select_lru_victims(&self, count: usize) -> Vec<&str> {
        self.lru_order
            .iter()
            .take(count)
            .map(|s| s.as_str())
            .collect()
    }

    // --- LFU helpers ---

    fn select_lfu_victim(&self) -> Option<&str> {
        self.entries
            .values()
            .min_by_key(|e| (e.access_count, e.last_access_tick))
            .map(|e| e.key.as_str())
    }

    fn select_lfu_victims(&self, count: usize) -> Vec<&str> {
        let mut entries: Vec<&EvictionEntry> = self.entries.values().collect();
        entries.sort_by_key(|e| (e.access_count, e.last_access_tick));
        entries.iter().take(count).map(|e| e.key.as_str()).collect()
    }
}
