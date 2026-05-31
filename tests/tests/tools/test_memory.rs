use hudhudscript_tools::memory::{
    InMemoryBackend, MemoryBackend, MemoryEntry, MemoryError, MemoryStore,
};
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_store_and_recall() {
    let store = MemoryStore::new();
    let id = store.store("agent-1", "user_name", "Alice").unwrap();
    assert!(!id.is_empty());

    let entry = store.recall("agent-1", "user_name").unwrap().unwrap();
    assert_eq!(entry.content, "Alice");
    assert_eq!(entry.key, "user_name");
}

#[test]
fn test_overwrite_same_key() {
    let store = MemoryStore::new();
    store.store("agent-1", "topic", "first version").unwrap();
    store.store("agent-1", "topic", "second version").unwrap();

    // Only one entry should remain
    let all = store.list("agent-1").unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].content, "second version");
}

#[test]
fn test_search() {
    let store = MemoryStore::new();
    store
        .store("agent-1", "weather", "It is sunny in Dubai")
        .unwrap();
    store
        .store("agent-1", "traffic", "Light traffic on Sheikh Zayed Road")
        .unwrap();
    store
        .store("agent-1", "news", "HudHud releases v0.3.25")
        .unwrap();

    let results = store.search("agent-1", "Dubai sunny", 5).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].key, "weather");
}

#[test]
fn test_forget() {
    let store = MemoryStore::new();
    let id = store.store("agent-1", "tmp", "temp data").unwrap();
    assert!(store.forget(&id).unwrap());
    assert!(store.recall("agent-1", "tmp").unwrap().is_none());
}

#[test]
fn test_clear_agent() {
    let store = MemoryStore::new();
    store.store("agent-1", "a", "data a").unwrap();
    store.store("agent-1", "b", "data b").unwrap();
    store.store("agent-2", "c", "other agent").unwrap();

    let deleted = store.clear("agent-1").unwrap();
    assert_eq!(deleted, 2);
    assert_eq!(store.list("agent-1").unwrap().len(), 0);
    assert_eq!(store.list("agent-2").unwrap().len(), 1);
}

#[test]
fn test_tool_store_and_recall() {
    let store = MemoryStore::new();
    let stored = store
        .tool_store(&serde_json::json!({
            "agent_id": "agent-1",
            "key": "preferences",
            "content": "user likes dark mode"
        }))
        .unwrap();
    assert_eq!(stored["stored"], true);

    let recalled = store
        .tool_recall(&serde_json::json!({
            "agent_id": "agent-1",
            "key": "preferences"
        }))
        .unwrap();
    assert_eq!(recalled["content"], "user likes dark mode");
}

#[test]
fn test_tool_recall_search() {
    let store = MemoryStore::new();
    store
        .store("agent-1", "ctx1", "The user lives in Abu Dhabi")
        .unwrap();
    store
        .store("agent-1", "ctx2", "The user prefers Arabic language")
        .unwrap();

    let result = store
        .tool_recall(&serde_json::json!({
            "agent_id": "agent-1",
            "query": "Abu Dhabi",
            "limit": 3
        }))
        .unwrap();

    let results = result["results"].as_array().unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_tool_forget() {
    let store = MemoryStore::new();
    let stored = store
        .tool_store(&serde_json::json!({
            "agent_id": "a1",
            "key": "tmp",
            "content": "x"
        }))
        .unwrap();
    let id = stored["id"].as_str().unwrap().to_string();

    let result = store.tool_forget(&serde_json::json!({ "id": id })).unwrap();
    assert_eq!(result["deleted"], true);
}

#[test]
fn test_recall_nonexistent_key() {
    let store = MemoryStore::new();
    let result = store.recall("agent-1", "no_such_key").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_recall_by_id() {
    let store = MemoryStore::new();
    let id = store.store("agent-1", "key1", "value1").unwrap();
    let entry = store.recall_by_id(&id).unwrap().unwrap();
    assert_eq!(entry.content, "value1");
    assert_eq!(entry.key, "key1");
    assert_eq!(entry.agent_id, "agent-1");
}

#[test]
fn test_recall_by_id_nonexistent() {
    let store = MemoryStore::new();
    let result = store.recall_by_id("nonexistent-id").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_store_with_metadata() {
    let store = MemoryStore::new();
    let mut meta = HashMap::new();
    meta.insert("source".to_string(), serde_json::json!("test"));
    let id = store
        .store_with_metadata("agent-1", "tagged", "tagged content", meta)
        .unwrap();

    let entry = store.recall_by_id(&id).unwrap().unwrap();
    assert_eq!(entry.content, "tagged content");
    assert_eq!(entry.metadata["source"], serde_json::json!("test"));
}

#[test]
fn test_forget_nonexistent() {
    let store = MemoryStore::new();
    let deleted = store.forget("nonexistent-id").unwrap();
    assert!(!deleted);
}

#[test]
fn test_search_no_matches() {
    let store = MemoryStore::new();
    store.store("agent-1", "weather", "sunny in Dubai").unwrap();
    let results = store.search("agent-1", "zzzzzz_nonexistent", 5).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_search_different_agent_isolation() {
    let store = MemoryStore::new();
    store.store("agent-1", "topic", "relevant content").unwrap();
    store
        .store("agent-2", "topic", "other agent content")
        .unwrap();

    let results = store.search("agent-1", "relevant", 5).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].agent_id, "agent-1");
}

#[test]
fn test_search_respects_limit() {
    let store = MemoryStore::new();
    for i in 0..10 {
        store
            .store(
                "agent-1",
                &format!("key_{}", i),
                &format!("data about topic {}", i),
            )
            .unwrap();
    }
    let results = store.search("agent-1", "topic", 3).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_clear_agent_returns_zero_when_empty() {
    let store = MemoryStore::new();
    let deleted = store.clear("agent-nonexistent").unwrap();
    assert_eq!(deleted, 0);
}

#[test]
fn test_list_empty_agent() {
    let store = MemoryStore::new();
    let list = store.list("agent-nonexistent").unwrap();
    assert_eq!(list.len(), 0);
}

#[test]
fn test_tool_store_missing_agent_id() {
    let store = MemoryStore::new();
    let result = store.tool_store(&serde_json::json!({"key": "k", "content": "c"}));
    assert!(result.is_err());
}

#[test]
fn test_tool_store_missing_key() {
    let store = MemoryStore::new();
    let result = store.tool_store(&serde_json::json!({"agent_id": "a", "content": "c"}));
    assert!(result.is_err());
}

#[test]
fn test_tool_store_missing_content() {
    let store = MemoryStore::new();
    let result = store.tool_store(&serde_json::json!({"agent_id": "a", "key": "k"}));
    assert!(result.is_err());
}

#[test]
fn test_tool_recall_missing_agent_id() {
    let store = MemoryStore::new();
    let result = store.tool_recall(&serde_json::json!({"key": "k"}));
    assert!(result.is_err());
}

#[test]
fn test_tool_recall_missing_key_and_query() {
    let store = MemoryStore::new();
    let result = store.tool_recall(&serde_json::json!({"agent_id": "a"}));
    assert!(result.is_err());
}

#[test]
fn test_tool_recall_key_not_found() {
    let store = MemoryStore::new();
    let result = store
        .tool_recall(&serde_json::json!({"agent_id": "a", "key": "missing"}))
        .unwrap();
    assert_eq!(result["found"], false);
}

#[test]
fn test_tool_forget_missing_id() {
    let store = MemoryStore::new();
    let result = store.tool_forget(&serde_json::json!({}));
    assert!(result.is_err());
}

#[test]
fn test_memory_store_default() {
    let store = MemoryStore::default();
    assert_eq!(store.list("any").unwrap().len(), 0);
}

#[test]
fn test_in_memory_backend_default() {
    let _backend = InMemoryBackend::default();
}

#[test]
fn test_memory_entry_timestamps() {
    let store = MemoryStore::new();
    let id = store.store("agent-1", "ts_test", "data").unwrap();
    let entry = store.recall_by_id(&id).unwrap().unwrap();
    // created_at and accessed_at should be the same non-zero value
    assert_eq!(entry.created_at, entry.accessed_at);
    assert!(entry.created_at > 0);
}

#[test]
fn test_search_sorted_by_relevance() {
    let store = MemoryStore::new();
    store.store("agent-1", "k1", "apple banana cherry").unwrap();
    store.store("agent-1", "k2", "apple banana").unwrap();
    store.store("agent-1", "k3", "apple").unwrap();

    let results = store.search("agent-1", "apple banana cherry", 5).unwrap();
    // k1 matches all 3 tokens, should be first
    assert_eq!(results[0].key, "k1");
}

// ---- MemoryStore with_backend ----

#[test]
fn test_memory_store_with_backend() {
    let backend = Arc::new(InMemoryBackend::new());
    let store = MemoryStore::with_backend(backend);
    let id = store.store("a", "k", "v").unwrap();
    assert!(!id.is_empty());
    let entry = store.recall("a", "k").unwrap().unwrap();
    assert_eq!(entry.content, "v");
}

// ---- MemoryStore clone shares backend ----

#[test]
fn test_memory_store_clone_shares_backend() {
    let store = MemoryStore::new();
    let store2 = store.clone();
    store.store("a", "key1", "value1").unwrap();
    let entry = store2.recall("a", "key1").unwrap();
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().content, "value1");
}

// ---- MemoryEntry metadata ----

#[test]
fn test_memory_entry_new_has_empty_metadata() {
    let entry = MemoryEntry::new("agent", "key", "content");
    assert!(entry.metadata.is_empty());
    assert_eq!(entry.score, 0.0);
    assert!(!entry.id.is_empty());
}

// ---- MemoryEntry serialization roundtrip ----

#[test]
fn test_memory_entry_serialization() {
    let entry = MemoryEntry::new("agent-1", "test_key", "test_content");
    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: MemoryEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.agent_id, "agent-1");
    assert_eq!(deserialized.key, "test_key");
    assert_eq!(deserialized.content, "test_content");
    // score is skip_serializing, so it should be default 0.0
    assert_eq!(deserialized.score, 0.0);
}

// ---- MemoryError display ----

#[test]
fn test_memory_error_display() {
    let err = MemoryError::NotFound("id123".to_string());
    assert!(err.to_string().contains("id123"));

    let err = MemoryError::Backend("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));
}

// ---- Search with empty query ----

#[test]
fn test_search_empty_query() {
    let store = MemoryStore::new();
    store.store("a", "k", "content").unwrap();
    // Empty query has no tokens, so no entries should match (score = 0)
    let results = store.search("a", "", 5).unwrap();
    assert_eq!(results.len(), 0);
}

// ---- Search is case-insensitive ----

#[test]
fn test_search_case_insensitive() {
    let store = MemoryStore::new();
    store.store("a", "weather", "SUNNY in DUBAI").unwrap();
    let results = store.search("a", "sunny dubai", 5).unwrap();
    assert_eq!(results.len(), 1);
}

// ---- tool_recall with default limit ----

#[test]
fn test_tool_recall_search_default_limit() {
    let store = MemoryStore::new();
    for i in 0..10 {
        store
            .store("a", &format!("k{}", i), &format!("data topic {}", i))
            .unwrap();
    }
    let result = store
        .tool_recall(&serde_json::json!({
            "agent_id": "a",
            "query": "topic"
        }))
        .unwrap();
    let results = result["results"].as_array().unwrap();
    // Default limit is 5
    assert!(results.len() <= 5);
}

// ---- InMemoryBackend: store deduplicates same agent+key ----

#[test]
fn test_in_memory_backend_dedup() {
    let backend = InMemoryBackend::new();
    backend.store(MemoryEntry::new("a", "k", "v1")).unwrap();
    backend.store(MemoryEntry::new("a", "k", "v2")).unwrap();
    let entries = backend.list_agent("a").unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, "v2");
}

// ---- InMemoryBackend: different agents same key ----

#[test]
fn test_in_memory_backend_different_agents_same_key() {
    let backend = InMemoryBackend::new();
    backend.store(MemoryEntry::new("a1", "k", "v1")).unwrap();
    backend.store(MemoryEntry::new("a2", "k", "v2")).unwrap();
    assert_eq!(backend.list_agent("a1").unwrap().len(), 1);
    assert_eq!(backend.list_agent("a2").unwrap().len(), 1);
}
