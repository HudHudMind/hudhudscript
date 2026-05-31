//! Content-addressable deduplication for cache entries
//!
//! Uses SHA-256 hashing to detect duplicate cache entries. When two governance
//! items have identical serialized content, the deduplication system can detect
//! this and avoid storing redundant data.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Content hash (SHA-256 hex string)
pub type ContentHash = String;

/// Result of a deduplication check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DedupResult {
    /// The content is new (no duplicates found)
    Unique {
        /// The computed content hash
        hash: ContentHash,
    },

    /// The content already exists in the cache under a different key
    Duplicate {
        /// The computed content hash
        hash: ContentHash,

        /// Keys that already have this content
        existing_keys: Vec<String>,
    },
}

/// Content-addressable deduplication store
///
/// Maintains a mapping from content hashes to the set of cache keys that
/// share that content. This allows efficient duplicate detection and
/// space-saving when identical governance items are stored under different IDs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DedupStore {
    /// Map from content hash to the set of keys that share that content
    hash_to_keys: HashMap<ContentHash, HashSet<String>>,

    /// Reverse map: key to its content hash (for efficient removal)
    key_to_hash: HashMap<String, ContentHash>,
}

impl DedupStore {
    /// Create an empty deduplication store
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute the SHA-256 hash of the given content bytes
    pub fn compute_hash(content: &[u8]) -> ContentHash {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }

    /// Check whether the given content already exists in the store.
    ///
    /// This does NOT register the content; call `register` to do that.
    pub fn check(&self, content: &[u8]) -> DedupResult {
        let hash = Self::compute_hash(content);

        if let Some(keys) = self.hash_to_keys.get(&hash) {
            if keys.is_empty() {
                DedupResult::Unique { hash }
            } else {
                DedupResult::Duplicate {
                    hash,
                    existing_keys: keys.iter().cloned().collect(),
                }
            }
        } else {
            DedupResult::Unique { hash }
        }
    }

    /// Register a key with its content hash.
    ///
    /// Returns the `DedupResult` indicating whether this is a new or duplicate entry.
    pub fn register(&mut self, key: &str, content: &[u8]) -> DedupResult {
        let hash = Self::compute_hash(content);

        // Check for existing entries with the same hash (excluding self)
        let existing: Vec<String> = self
            .hash_to_keys
            .get(&hash)
            .map(|keys| keys.iter().filter(|k| k.as_str() != key).cloned().collect())
            .unwrap_or_default();

        let result = if existing.is_empty() {
            DedupResult::Unique { hash: hash.clone() }
        } else {
            DedupResult::Duplicate {
                hash: hash.clone(),
                existing_keys: existing,
            }
        };

        // Remove old hash mapping if key was previously registered with different content
        if let Some(old_hash) = self.key_to_hash.get(key) {
            if old_hash != &hash {
                let old_hash_clone = old_hash.clone();
                if let Some(keys) = self.hash_to_keys.get_mut(&old_hash_clone) {
                    keys.remove(key);
                    if keys.is_empty() {
                        self.hash_to_keys.remove(&old_hash_clone);
                    }
                }
            }
        }

        // Register the new mapping
        self.hash_to_keys
            .entry(hash.clone())
            .or_default()
            .insert(key.to_string());
        self.key_to_hash.insert(key.to_string(), hash);

        result
    }

    /// Remove a key from the deduplication store
    pub fn remove(&mut self, key: &str) {
        if let Some(hash) = self.key_to_hash.remove(key) {
            if let Some(keys) = self.hash_to_keys.get_mut(&hash) {
                keys.remove(key);
                if keys.is_empty() {
                    self.hash_to_keys.remove(&hash);
                }
            }
        }
    }

    /// Get the content hash for a given key
    pub fn hash_for_key(&self, key: &str) -> Option<&ContentHash> {
        self.key_to_hash.get(key)
    }

    /// Get all keys that share the given content hash
    pub fn keys_for_hash(&self, hash: &str) -> Option<&HashSet<String>> {
        self.hash_to_keys.get(hash)
    }

    /// Total number of unique content hashes
    pub fn unique_content_count(&self) -> usize {
        self.hash_to_keys.len()
    }

    /// Total number of registered keys
    pub fn total_key_count(&self) -> usize {
        self.key_to_hash.len()
    }

    /// Number of keys that are duplicates (share a hash with at least one other key)
    pub fn duplicate_key_count(&self) -> usize {
        self.hash_to_keys
            .values()
            .filter(|keys| keys.len() > 1)
            .map(|keys| keys.len() - 1)
            .sum()
    }

    /// Estimated bytes saved through deduplication.
    ///
    /// `avg_entry_size` is the average size of a cache entry in bytes.
    /// The savings estimate is `duplicate_key_count * avg_entry_size`.
    pub fn estimated_savings(&self, avg_entry_size: u64) -> u64 {
        self.duplicate_key_count() as u64 * avg_entry_size
    }

    /// Clear the entire deduplication store
    pub fn clear(&mut self) {
        self.hash_to_keys.clear();
        self.key_to_hash.clear();
    }

    /// Whether the store is empty
    pub fn is_empty(&self) -> bool {
        self.key_to_hash.is_empty()
    }
}
