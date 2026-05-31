//! Unified cache index across all cache types
//!
//! Provides a single index that tracks every entry stored in the cache,
//! regardless of its governance type (constitution, law, or rule). The
//! index enables efficient lookup by key, iteration over all entries,
//! and provides the foundation for cross-cutting concerns like eviction
//! and quota monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type of governance item stored in the cache
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheEntryType {
    /// A constitution
    Constitution,

    /// A law
    Law,

    /// A rule
    Rule,
}

impl std::fmt::Display for CacheEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constitution => write!(f, "constitution"),
            Self::Law => write!(f, "law"),
            Self::Rule => write!(f, "rule"),
        }
    }
}

/// Metadata stored in the index for each cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    /// The unique key (same as the governance item's ID)
    pub key: String,

    /// What kind of governance item this is
    pub entry_type: CacheEntryType,

    /// Estimated size of the entry in bytes (serialized size)
    pub size_bytes: u64,

    /// When the entry was first inserted
    pub created_at: DateTime<Utc>,

    /// Content-addressable hash (SHA-256 hex), if computed
    pub content_hash: Option<String>,
}

/// Unified cache index that tracks all entries across governance types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheIndex {
    /// Key -> IndexEntry mapping
    entries: HashMap<String, IndexEntry>,
}

impl CacheIndex {
    /// Create an empty index
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update an entry in the index
    pub fn insert(&mut self, entry: IndexEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }

    /// Remove an entry by key, returning it if it existed
    pub fn remove(&mut self, key: &str) -> Option<IndexEntry> {
        self.entries.remove(key)
    }

    /// Look up an entry by key
    pub fn get(&self, key: &str) -> Option<&IndexEntry> {
        self.entries.get(key)
    }

    /// Check whether the index contains the given key
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Total number of entries in the index
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Total estimated size of all indexed entries in bytes
    pub fn total_size_bytes(&self) -> u64 {
        self.entries.values().map(|e| e.size_bytes).sum()
    }

    /// Count of entries by type
    pub fn count_by_type(&self, entry_type: CacheEntryType) -> usize {
        self.entries
            .values()
            .filter(|e| e.entry_type == entry_type)
            .count()
    }

    /// Iterator over all index entries
    pub fn iter(&self) -> impl Iterator<Item = (&String, &IndexEntry)> {
        self.entries.iter()
    }

    /// Return all keys in the index
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Return all keys that have the given content hash.
    ///
    /// This is used by the deduplication system to find duplicate entries.
    pub fn keys_with_hash(&self, hash: &str) -> Vec<&str> {
        self.entries
            .values()
            .filter(|e| e.content_hash.as_deref() == Some(hash))
            .map(|e| e.key.as_str())
            .collect()
    }

    /// Clear the entire index
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Build a summary of cache contents by type
    pub fn summary(&self) -> IndexSummary {
        let mut constitutions = 0usize;
        let mut laws = 0usize;
        let mut rules = 0usize;
        let mut total_bytes = 0u64;

        for entry in self.entries.values() {
            total_bytes += entry.size_bytes;
            match entry.entry_type {
                CacheEntryType::Constitution => constitutions += 1,
                CacheEntryType::Law => laws += 1,
                CacheEntryType::Rule => rules += 1,
            }
        }

        IndexSummary {
            total_entries: self.entries.len(),
            constitutions,
            laws,
            rules,
            total_size_bytes: total_bytes,
        }
    }
}

/// Summary statistics for the cache index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSummary {
    /// Total number of entries
    pub total_entries: usize,

    /// Number of constitutions
    pub constitutions: usize,

    /// Number of laws
    pub laws: usize,

    /// Number of rules
    pub rules: usize,

    /// Total estimated size in bytes
    pub total_size_bytes: u64,
}
