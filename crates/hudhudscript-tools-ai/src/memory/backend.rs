use std::collections::HashMap;
use std::sync::RwLock;

use super::{MemoryEntry, MemoryError};

/// Pluggable storage backend for the memory store.
///
/// Swap in a real embedding / vector DB implementation for semantic search.
pub trait MemoryBackend: Send + Sync {
    /// Store an entry; overwrites entries with the same (agent_id, key)
    fn store(&self, entry: MemoryEntry) -> Result<String, MemoryError>;

    /// Retrieve an entry by its id
    fn get_by_id(&self, id: &str) -> Result<Option<MemoryEntry>, MemoryError>;

    /// Retrieve an entry by (agent_id, key)
    fn get_by_key(&self, agent_id: &str, key: &str) -> Result<Option<MemoryEntry>, MemoryError>;

    /// Keyword / semantic search — returns up to `limit` entries sorted by relevance
    fn search(
        &self,
        agent_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// Delete an entry by id; returns `true` when deleted
    fn delete(&self, id: &str) -> Result<bool, MemoryError>;

    /// List all entries for an agent
    fn list_agent(&self, agent_id: &str) -> Result<Vec<MemoryEntry>, MemoryError>;

    /// Delete all entries for an agent
    fn clear_agent(&self, agent_id: &str) -> Result<usize, MemoryError>;
}

/// Simple in-memory backend with keyword-based relevance scoring.
///
/// Relevance is computed as the number of query tokens that appear in the
/// entry key or content (case-insensitive).  A real RAG backend would replace
/// this with embedding similarity.
pub struct InMemoryBackend {
    entries: RwLock<HashMap<String, MemoryEntry>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBackend for InMemoryBackend {
    fn store(&self, entry: MemoryEntry) -> Result<String, MemoryError> {
        let id = entry.id.clone();
        let mut map = self.entries.write().unwrap_or_else(|e| e.into_inner());

        let agent_id = entry.agent_id.clone();
        let key = entry.key.clone();
        map.retain(|_, e| !(e.agent_id == agent_id && e.key == key));

        map.insert(id.clone(), entry);
        Ok(id)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        let map = self.entries.read().unwrap_or_else(|e| e.into_inner());
        Ok(map.get(id).cloned())
    }

    fn get_by_key(&self, agent_id: &str, key: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        let map = self.entries.read().unwrap_or_else(|e| e.into_inner());
        Ok(map
            .values()
            .find(|e| e.agent_id == agent_id && e.key == key)
            .cloned())
    }

    fn search(
        &self,
        agent_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        let tokens: Vec<&str> = query.split_whitespace().collect();
        let map = self.entries.read().unwrap_or_else(|e| e.into_inner());

        let mut scored: Vec<MemoryEntry> = map
            .values()
            .filter(|e| e.agent_id == agent_id)
            .map(|e| {
                let haystack = format!("{} {}", e.key, e.content).to_lowercase();
                let hits = tokens
                    .iter()
                    .filter(|t| haystack.contains(&t.to_lowercase()))
                    .count();
                let mut entry = e.clone();
                entry.score = hits as f32 / tokens.len().max(1) as f32;
                entry
            })
            .filter(|e| e.score > 0.0)
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit);
        Ok(scored)
    }

    fn delete(&self, id: &str) -> Result<bool, MemoryError> {
        let mut map = self.entries.write().unwrap_or_else(|e| e.into_inner());
        Ok(map.remove(id).is_some())
    }

    fn list_agent(&self, agent_id: &str) -> Result<Vec<MemoryEntry>, MemoryError> {
        let map = self.entries.read().unwrap_or_else(|e| e.into_inner());
        Ok(map
            .values()
            .filter(|e| e.agent_id == agent_id)
            .cloned()
            .collect())
    }

    fn clear_agent(&self, agent_id: &str) -> Result<usize, MemoryError> {
        let mut map = self.entries.write().unwrap_or_else(|e| e.into_inner());
        let before = map.len();
        map.retain(|_, e| e.agent_id != agent_id);
        Ok(before - map.len())
    }
}
