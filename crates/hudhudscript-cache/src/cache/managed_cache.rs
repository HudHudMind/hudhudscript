use crate::command_cache::CommandCache;
use crate::dedup::DedupStore;
use crate::error::CacheError;
use crate::eviction::{EvictionEngine, EvictionPolicy};
use crate::index::{CacheEntryType, CacheIndex, IndexEntry};
use crate::quota::{QuotaAlert, QuotaConfig, QuotaMonitor};
use crate::stats::{CacheStatistics, PruneResult};
use chrono::Utc;
use hudhudscript_governance::{Constitution, ConstitutionId, Law, LawId, Rule, RuleId};

/// Configuration for the managed cache
#[derive(Debug, Clone)]
pub struct ManagedCacheConfig {
    pub max_items: usize,
    pub eviction_policy: EvictionPolicy,
    pub quota_config: QuotaConfig,
    pub enable_dedup: bool,
}

impl Default for ManagedCacheConfig {
    fn default() -> Self {
        Self {
            max_items: 1000,
            eviction_policy: EvictionPolicy::Lru,
            quota_config: QuotaConfig::default(),
            enable_dedup: true,
        }
    }
}

pub struct ManagedCache {
    inner: CommandCache,
    eviction: EvictionEngine,
    quota: QuotaMonitor,
    index: CacheIndex,
    dedup: DedupStore,
    dedup_enabled: bool,
    alerts: Vec<QuotaAlert>,
}

impl ManagedCache {
    pub fn new() -> Self {
        Self::with_config(ManagedCacheConfig::default())
    }

    pub fn with_config(config: ManagedCacheConfig) -> Self {
        Self {
            inner: CommandCache::with_capacity(config.max_items),
            eviction: EvictionEngine::new(config.eviction_policy),
            quota: QuotaMonitor::with_config(config.quota_config),
            index: CacheIndex::new(),
            dedup: DedupStore::new(),
            dedup_enabled: config.enable_dedup,
            alerts: Vec::new(),
        }
    }

    pub fn inner(&self) -> &CommandCache {
        &self.inner
    }

    pub fn eviction(&self) -> &EvictionEngine {
        &self.eviction
    }

    pub fn quota(&self) -> &QuotaMonitor {
        &self.quota
    }

    pub fn index(&self) -> &CacheIndex {
        &self.index
    }

    pub fn dedup_store(&self) -> &DedupStore {
        &self.dedup
    }

    pub fn drain_alerts(&mut self) -> Vec<QuotaAlert> {
        std::mem::take(&mut self.alerts)
    }

    pub fn store_constitution(
        &mut self,
        constitution: Constitution,
    ) -> Result<ConstitutionId, CacheError> {
        let key = constitution.id.clone();
        let serialized = serde_json::to_vec(&constitution)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        let size = serialized.len() as u64;

        if self.dedup_enabled {
            self.dedup.register(&key, &serialized);
        }

        self.evict_if_needed();
        let id = self.inner.store_constitution(constitution)?;

        self.eviction.record_access(&key);
        self.index.insert(IndexEntry {
            key: key.clone(),
            entry_type: CacheEntryType::Constitution,
            size_bytes: size,
            created_at: Utc::now(),
            content_hash: self.dedup.hash_for_key(&key).cloned(),
        });
        if let Some(alert) = self.quota.record_addition(size) {
            self.alerts.push(alert);
        }
        Ok(id)
    }

    pub fn resolve_constitution(&mut self, id: &ConstitutionId) -> Option<&Constitution> {
        if self.inner.constitutions.contains_key(id) {
            self.eviction.record_access(id);
        }
        self.inner.resolve_constitution(id)
    }

    pub fn store_law(&mut self, law: Law) -> Result<LawId, CacheError> {
        let key = law.id.clone();
        let serialized =
            serde_json::to_vec(&law).map_err(|e| CacheError::SerializationError(e.to_string()))?;
        let size = serialized.len() as u64;

        if self.dedup_enabled {
            self.dedup.register(&key, &serialized);
        }

        self.evict_if_needed();
        let id = self.inner.store_law(law)?;

        self.eviction.record_access(&key);
        self.index.insert(IndexEntry {
            key: key.clone(),
            entry_type: CacheEntryType::Law,
            size_bytes: size,
            created_at: Utc::now(),
            content_hash: self.dedup.hash_for_key(&key).cloned(),
        });
        if let Some(alert) = self.quota.record_addition(size) {
            self.alerts.push(alert);
        }
        Ok(id)
    }

    pub fn resolve_law(&mut self, id: &LawId) -> Option<&Law> {
        if self.inner.laws.contains_key(id) {
            self.eviction.record_access(id);
        }
        self.inner.resolve_law(id)
    }

    pub fn store_rule(&mut self, rule: Rule) -> Result<RuleId, CacheError> {
        let key = rule.id.clone();
        let serialized =
            serde_json::to_vec(&rule).map_err(|e| CacheError::SerializationError(e.to_string()))?;
        let size = serialized.len() as u64;

        if self.dedup_enabled {
            self.dedup.register(&key, &serialized);
        }

        self.evict_if_needed();
        let id = self.inner.store_rule(rule)?;

        self.eviction.record_access(&key);
        self.index.insert(IndexEntry {
            key: key.clone(),
            entry_type: CacheEntryType::Rule,
            size_bytes: size,
            created_at: Utc::now(),
            content_hash: self.dedup.hash_for_key(&key).cloned(),
        });
        if let Some(alert) = self.quota.record_addition(size) {
            self.alerts.push(alert);
        }
        Ok(id)
    }

    pub fn resolve_rule(&mut self, id: &RuleId) -> Option<&Rule> {
        if self.inner.rules.contains_key(id) {
            self.eviction.record_access(id);
        }
        self.inner.resolve_rule(id)
    }

    pub fn prune(&mut self, count: usize) -> PruneResult {
        let victims: Vec<String> = self
            .eviction
            .select_victims(count)
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut bytes_reclaimed = 0u64;
        let mut removed_keys = Vec::new();

        for key in &victims {
            if let Some(entry) = self.index.remove(key) {
                bytes_reclaimed += entry.size_bytes;
            }

            let removed = self.inner.constitutions.remove(key).is_some()
                || self.inner.laws.remove(key).is_some()
                || self.inner.rules.remove(key).is_some();

            if removed {
                removed_keys.push(key.clone());
            }

            self.eviction.remove(key);
            self.dedup.remove(key);
        }

        if let Some(alert) = self.quota.record_removal(bytes_reclaimed) {
            self.alerts.push(alert);
        }

        PruneResult {
            entries_removed: removed_keys.len(),
            bytes_reclaimed,
            removed_keys,
        }
    }

    pub fn clear(&mut self) {
        self.inner = CommandCache::with_capacity(self.inner.max_size);
        self.eviction.clear();
        self.index.clear();
        self.dedup.clear();

        let current = self.quota.current_bytes();
        if let Some(alert) = self.quota.record_removal(current) {
            self.alerts.push(alert);
        }
    }

    pub fn statistics(&self) -> CacheStatistics {
        let total = self.inner.constitutions.len() + self.inner.laws.len() + self.inner.rules.len();

        let avg_size = if total > 0 {
            self.index.total_size_bytes() / total as u64
        } else {
            0
        };

        CacheStatistics {
            constitution_count: self.inner.constitutions.len(),
            law_count: self.inner.laws.len(),
            rule_count: self.inner.rules.len(),
            total_items: total,
            max_capacity: self.inner.max_size,
            total_size_bytes: self.index.total_size_bytes(),
            unique_contents: self.dedup.unique_content_count(),
            duplicate_count: self.dedup.duplicate_key_count(),
            dedup_savings_bytes: self.dedup.estimated_savings(avg_size),
            quota_usage_percent: self.quota.usage_percent(),
            quota_remaining_bytes: self.quota.remaining_bytes(),
            eviction_policy: self.eviction.policy(),
        }
    }

    fn evict_if_needed(&mut self) {
        let total = self.inner.constitutions.len() + self.inner.laws.len() + self.inner.rules.len();

        if total < self.inner.max_size {
            return;
        }

        if let Some(victim_key) = self.eviction.select_victim().map(|s| s.to_string()) {
            let mut bytes = 0u64;
            if let Some(entry) = self.index.remove(&victim_key) {
                bytes = entry.size_bytes;
            }

            self.inner.constitutions.remove(&victim_key);
            self.inner.laws.remove(&victim_key);
            self.inner.rules.remove(&victim_key);
            self.eviction.remove(&victim_key);
            self.dedup.remove(&victim_key);

            self.inner.constitution_lru.remove(&victim_key);
            self.inner.law_lru.remove(&victim_key);
            self.inner.rule_lru.remove(&victim_key);

            if let Some(alert) = self.quota.record_removal(bytes) {
                self.alerts.push(alert);
            }
        }
    }
}

impl Default for ManagedCache {
    fn default() -> Self {
        Self::new()
    }
}
