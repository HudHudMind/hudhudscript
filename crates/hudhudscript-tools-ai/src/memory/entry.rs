use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A single memory entry stored by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier
    pub id: String,
    /// Agent that owns this entry (namespaces memory per agent)
    pub agent_id: String,
    /// Human-readable key / topic
    pub key: String,
    /// Stored content
    pub content: String,
    /// Arbitrary metadata (tags, source, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
    /// When the entry was created (Unix timestamp seconds)
    pub created_at: u64,
    /// When the entry was last accessed
    pub accessed_at: u64,
    /// Relevance score after a retrieval query (not persisted)
    #[serde(skip)]
    pub score: f32,
}

impl MemoryEntry {
    pub fn new(agent_id: &str, key: &str, content: &str) -> Self {
        let now = super::unix_now();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            key: key.to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
            created_at: now,
            accessed_at: now,
            score: 0.0,
        }
    }
}
