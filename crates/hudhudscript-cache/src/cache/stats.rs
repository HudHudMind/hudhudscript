use crate::eviction::EvictionPolicy;
use serde::{Deserialize, Serialize};

/// Comprehensive cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatistics {
    /// Number of constitutions
    pub constitution_count: usize,
    /// Number of laws
    pub law_count: usize,
    /// Number of rules
    pub rule_count: usize,
    /// Total number of items across all types
    pub total_items: usize,
    /// Maximum cache capacity (item count)
    pub max_capacity: usize,
    /// Estimated total size in bytes (from the index)
    pub total_size_bytes: u64,
    /// Number of unique content hashes (from dedup)
    pub unique_contents: usize,
    /// Number of duplicate entries detected
    pub duplicate_count: usize,
    /// Estimated bytes saved through deduplication
    pub dedup_savings_bytes: u64,
    /// Current quota usage as a percentage
    pub quota_usage_percent: f64,
    /// Remaining quota in bytes
    pub quota_remaining_bytes: u64,
    /// Active eviction policy
    pub eviction_policy: EvictionPolicy,
}

/// Result of a prune operation
#[derive(Debug, Clone)]
pub struct PruneResult {
    /// Number of entries removed
    pub entries_removed: usize,
    /// Estimated bytes reclaimed
    pub bytes_reclaimed: u64,
    /// Keys that were removed
    pub removed_keys: Vec<String>,
}
