use std::collections::HashMap;
use std::sync::Arc;

use super::{InMemoryBackend, MemoryBackend, MemoryEntry, MemoryError};

/// High-level memory store for agents.
///
/// Wraps a `MemoryBackend` and exposes a clean API that matches the
/// `memory_store` / `memory_recall` tool call signatures expected by agents.
#[derive(Clone)]
pub struct MemoryStore {
    backend: Arc<dyn MemoryBackend>,
}

impl MemoryStore {
    /// Create a new store with the default in-memory backend
    pub fn new() -> Self {
        Self {
            backend: Arc::new(InMemoryBackend::new()),
        }
    }

    /// Create a store with a custom backend
    pub fn with_backend(backend: Arc<dyn MemoryBackend>) -> Self {
        Self { backend }
    }

    /// Store a piece of context for an agent under a given key
    pub fn store(&self, agent_id: &str, key: &str, content: &str) -> Result<String, MemoryError> {
        let entry = MemoryEntry::new(agent_id, key, content);
        self.backend.store(entry)
    }

    /// Store with extra metadata
    pub fn store_with_metadata(
        &self,
        agent_id: &str,
        key: &str,
        content: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<String, MemoryError> {
        let mut entry = MemoryEntry::new(agent_id, key, content);
        entry.metadata = metadata;
        self.backend.store(entry)
    }

    /// Retrieve a memory by key
    pub fn recall(&self, agent_id: &str, key: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        self.backend.get_by_key(agent_id, key)
    }

    /// Recall by entry id
    pub fn recall_by_id(&self, id: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        self.backend.get_by_id(id)
    }

    /// Semantic / keyword search across an agent's memory
    pub fn search(
        &self,
        agent_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>, MemoryError> {
        self.backend.search(agent_id, query, limit)
    }

    /// Delete a memory entry by id
    pub fn forget(&self, id: &str) -> Result<bool, MemoryError> {
        self.backend.delete(id)
    }

    /// List all memories for an agent
    pub fn list(&self, agent_id: &str) -> Result<Vec<MemoryEntry>, MemoryError> {
        self.backend.list_agent(agent_id)
    }

    /// Delete all memories for an agent
    pub fn clear(&self, agent_id: &str) -> Result<usize, MemoryError> {
        self.backend.clear_agent(agent_id)
    }

    /// Execute the `memory_store` tool call
    pub fn tool_store(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let agent_id = args["agent_id"].as_str().ok_or("missing agent_id")?;
        let key = args["key"].as_str().ok_or("missing key")?;
        let content = args["content"].as_str().ok_or("missing content")?;

        let id = self
            .store(agent_id, key, content)
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({ "id": id, "key": key, "stored": true }))
    }

    /// Execute the `memory_recall` tool call
    pub fn tool_recall(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let agent_id = args["agent_id"].as_str().ok_or("missing agent_id")?;

        if let Some(key) = args["key"].as_str() {
            match self.recall(agent_id, key).map_err(|e| e.to_string())? {
                Some(entry) => Ok(serde_json::json!({
                    "id": entry.id,
                    "key": entry.key,
                    "content": entry.content,
                    "created_at": entry.created_at
                })),
                None => Ok(serde_json::json!({ "found": false })),
            }
        } else if let Some(query) = args["query"].as_str() {
            let limit = args["limit"].as_u64().unwrap_or(5) as usize;
            let results = self
                .search(agent_id, query, limit)
                .map_err(|e| e.to_string())?;

            let entries: Vec<serde_json::Value> = results
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "id": e.id,
                        "key": e.key,
                        "content": e.content,
                        "score": e.score
                    })
                })
                .collect();

            Ok(serde_json::json!({ "results": entries }))
        } else {
            Err("provide either 'key' or 'query'".to_string())
        }
    }

    /// Execute the `memory_forget` tool call
    pub fn tool_forget(&self, args: &serde_json::Value) -> Result<serde_json::Value, String> {
        let id = args["id"].as_str().ok_or("missing id")?;
        let deleted = self.forget(id).map_err(|e| e.to_string())?;
        Ok(serde_json::json!({ "deleted": deleted }))
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}
