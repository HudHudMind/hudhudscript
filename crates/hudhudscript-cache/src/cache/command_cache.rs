use crate::error::CacheError;
use hashlink::LinkedHashSet;
use hudhudscript_governance::{Constitution, ConstitutionId, Law, LawId, Rule, RuleId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default cache size limit
const DEFAULT_CACHE_SIZE: usize = 1000;

/// Command cache for governance structures with LRU eviction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCache {
    pub constitutions: HashMap<ConstitutionId, Constitution>,
    pub laws: HashMap<LawId, Law>,
    pub rules: HashMap<RuleId, Rule>,
    pub(crate) constitution_lru: LinkedHashSet<ConstitutionId>,
    pub(crate) law_lru: LinkedHashSet<LawId>,
    pub(crate) rule_lru: LinkedHashSet<RuleId>,
    pub max_size: usize,
}

impl Default for CommandCache {
    fn default() -> Self {
        Self {
            constitutions: HashMap::new(),
            laws: HashMap::new(),
            rules: HashMap::new(),
            constitution_lru: LinkedHashSet::new(),
            law_lru: LinkedHashSet::new(),
            rule_lru: LinkedHashSet::new(),
            max_size: DEFAULT_CACHE_SIZE,
        }
    }
}

impl CommandCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            constitutions: HashMap::new(),
            laws: HashMap::new(),
            rules: HashMap::new(),
            constitution_lru: LinkedHashSet::new(),
            law_lru: LinkedHashSet::new(),
            rule_lru: LinkedHashSet::new(),
            max_size,
        }
    }

    pub fn total_items(&self) -> usize {
        self.constitutions.len() + self.laws.len() + self.rules.len()
    }

    fn evict_if_needed(&mut self) {
        while self.total_items() >= self.max_size {
            if let Some(rule_id) = self.rule_lru.pop_front() {
                self.rules.remove(&rule_id);
            } else if let Some(law_id) = self.law_lru.pop_front() {
                self.laws.remove(&law_id);
            } else if let Some(const_id) = self.constitution_lru.pop_front() {
                self.constitutions.remove(&const_id);
            } else {
                break;
            }
        }
    }

    fn touch_constitution(&mut self, id: &ConstitutionId) {
        self.constitution_lru.remove(id);
        self.constitution_lru.insert(id.clone());
    }

    fn touch_law(&mut self, id: &LawId) {
        self.law_lru.remove(id);
        self.law_lru.insert(id.clone());
    }

    fn touch_rule(&mut self, id: &RuleId) {
        self.rule_lru.remove(id);
        self.rule_lru.insert(id.clone());
    }

    pub fn store_constitution(
        &mut self,
        constitution: Constitution,
    ) -> Result<ConstitutionId, CacheError> {
        let id = constitution.id.clone();
        if self.constitutions.contains_key(&id) {
            return Err(CacheError::IdCollision(id));
        }
        self.evict_if_needed();
        self.constitutions.insert(id.clone(), constitution);
        self.touch_constitution(&id);
        Ok(id)
    }

    pub fn resolve_constitution(&mut self, id: &ConstitutionId) -> Option<&Constitution> {
        if self.constitutions.contains_key(id) {
            self.touch_constitution(id);
            self.constitutions.get(id)
        } else {
            None
        }
    }

    pub fn store_law(&mut self, law: Law) -> Result<LawId, CacheError> {
        let id = law.id.clone();
        if self.laws.contains_key(&id) {
            return Err(CacheError::IdCollision(id));
        }
        self.evict_if_needed();
        self.laws.insert(id.clone(), law);
        self.touch_law(&id);
        Ok(id)
    }

    pub fn resolve_law(&mut self, id: &LawId) -> Option<&Law> {
        if self.laws.contains_key(id) {
            self.touch_law(id);
            self.laws.get(id)
        } else {
            None
        }
    }

    pub fn store_rule(&mut self, rule: Rule) -> Result<RuleId, CacheError> {
        let id = rule.id.clone();
        if self.rules.contains_key(&id) {
            return Err(CacheError::IdCollision(id));
        }
        self.evict_if_needed();
        self.rules.insert(id.clone(), rule);
        self.touch_rule(&id);
        Ok(id)
    }

    pub fn resolve_rule(&mut self, id: &RuleId) -> Option<&Rule> {
        if self.rules.contains_key(id) {
            self.touch_rule(id);
            self.rules.get(id)
        } else {
            None
        }
    }

    pub fn serialize_definitions(&self) -> Result<String, CacheError> {
        serde_json::to_string(self).map_err(|e| CacheError::SerializationError(e.to_string()))
    }

    pub fn deserialize_definitions(data: &str) -> Result<Self, CacheError> {
        let mut cache: Self = serde_json::from_str(data)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

        if cache.constitution_lru.is_empty() && !cache.constitutions.is_empty() {
            cache.constitution_lru = cache.constitutions.keys().cloned().collect();
        } else {
            cache
                .constitution_lru
                .retain(|id| cache.constitutions.contains_key(id));
        }

        if cache.law_lru.is_empty() && !cache.laws.is_empty() {
            cache.law_lru = cache.laws.keys().cloned().collect();
        } else {
            cache.law_lru.retain(|id| cache.laws.contains_key(id));
        }

        if cache.rule_lru.is_empty() && !cache.rules.is_empty() {
            cache.rule_lru = cache.rules.keys().cloned().collect();
        } else {
            cache.rule_lru.retain(|id| cache.rules.contains_key(id));
        }

        Ok(cache)
    }
}
