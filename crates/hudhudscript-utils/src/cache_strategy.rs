//! Response Cache Strategy Consolidation (#822)
//!
//! This module defines a unified cache strategy configuration that all
//! response cache implementations can share. Instead of each crate defining
//! its own strategy types, they should use `CacheStrategy` from this module.

use std::time::Duration;

/// Unified cache strategy for response caches.
///
/// Combines lookup mode (exact vs semantic vs hybrid) with eviction
/// configuration (TTL, max entries, eviction policy).
#[derive(Debug, Clone)]
pub struct CacheStrategy {
    /// How to look up cached responses.
    pub lookup: LookupMode,
    /// Maximum number of entries in the cache.
    pub max_entries: usize,
    /// Time-to-live for cached entries.
    pub ttl: Duration,
    /// Eviction policy when the cache is full.
    pub eviction: super::cache_trait::EvictionPolicy,
}

/// How cached responses are looked up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupMode {
    /// Exact key match (hash-based).
    Exact,
    /// Semantic similarity (embedding-based, cosine similarity).
    Semantic,
    /// Try exact match first, fall back to semantic.
    Hybrid,
}

impl Default for CacheStrategy {
    fn default() -> Self {
        Self {
            lookup: LookupMode::Exact,
            max_entries: 1000,
            ttl: Duration::from_secs(300), // 5 minutes
            eviction: super::cache_trait::EvictionPolicy::Lru,
        }
    }
}

impl CacheStrategy {
    /// Create a strategy with exact matching.
    pub fn exact(max_entries: usize, ttl: Duration) -> Self {
        Self {
            lookup: LookupMode::Exact,
            max_entries,
            ttl,
            eviction: super::cache_trait::EvictionPolicy::Lru,
        }
    }

    /// Create a strategy with semantic matching.
    pub fn semantic(max_entries: usize, ttl: Duration) -> Self {
        Self {
            lookup: LookupMode::Semantic,
            max_entries,
            ttl,
            eviction: super::cache_trait::EvictionPolicy::Lru,
        }
    }

    /// Create a hybrid strategy (exact + semantic fallback).
    pub fn hybrid(max_entries: usize, ttl: Duration) -> Self {
        Self {
            lookup: LookupMode::Hybrid,
            max_entries,
            ttl,
            eviction: super::cache_trait::EvictionPolicy::Lru,
        }
    }

    /// Whether this strategy uses semantic similarity.
    pub fn uses_embeddings(&self) -> bool {
        matches!(self.lookup, LookupMode::Semantic | LookupMode::Hybrid)
    }
}
