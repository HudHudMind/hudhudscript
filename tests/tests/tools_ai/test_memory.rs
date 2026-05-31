//! Tests extracted from hudhudscript-tools-ai/src/memory.rs
//! Skipped (already in tools_ai_test_lib.rs): test_store_and_recall,
//! test_overwrite_same_key, test_recall_nonexistent_key, test_recall_by_id,
//! test_recall_by_id_nonexistent, test_search, test_search_no_results,
//! test_forget, test_forget_nonexistent, test_clear_agent,
//! test_store_with_metadata, test_memory_store_default

use hudhudscript_tools_ai::{InMemoryBackend, MemoryBackend, MemoryError, MemoryStore};

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
fn test_search_different_agent() {
    let store = MemoryStore::new();
    store.store("agent-1", "topic", "important data").unwrap();
    let results = store.search("agent-2", "important", 5).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_tool_recall_missing_key_and_query() {
    let store = MemoryStore::new();
    let result = store.tool_recall(&serde_json::json!({
        "agent_id": "agent-1"
    }));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "provide either 'key' or 'query'");
}

#[test]
fn test_tool_recall_key_not_found() {
    let store = MemoryStore::new();
    let result = store
        .tool_recall(&serde_json::json!({
            "agent_id": "agent-1",
            "key": "nonexistent"
        }))
        .unwrap();
    assert_eq!(result["found"], false);
}

#[test]
fn test_tool_store_missing_fields() {
    let store = MemoryStore::new();
    assert!(store.tool_store(&serde_json::json!({})).is_err());
    assert!(store
        .tool_store(&serde_json::json!({"agent_id": "a"}))
        .is_err());
    assert!(store
        .tool_store(&serde_json::json!({"agent_id": "a", "key": "k"}))
        .is_err());
}

#[test]
fn test_tool_forget_missing_id() {
    let store = MemoryStore::new();
    let result = store.tool_forget(&serde_json::json!({}));
    assert!(result.is_err());
}

#[test]
fn test_tool_recall_missing_agent_id() {
    let store = MemoryStore::new();
    let result = store.tool_recall(&serde_json::json!({"key": "k"}));
    assert!(result.is_err());
}

#[test]
fn test_in_memory_backend_default() {
    let backend = InMemoryBackend::default();
    assert!(backend.list_agent("any").unwrap().is_empty());
}

#[test]
fn test_memory_error_display() {
    let e1 = MemoryError::NotFound("id123".to_string());
    assert!(format!("{}", e1).contains("Memory entry not found: id123"));

    let e2 = MemoryError::Backend("connection lost".to_string());
    assert!(format!("{}", e2).contains("Backend error: connection lost"));
}

#[test]
fn test_clear_agent_none() {
    let store = MemoryStore::new();
    let deleted = store.clear("nonexistent").unwrap();
    assert_eq!(deleted, 0);
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
