//! Tests extracted from hudhudscript-rag/src/store.rs

use hudhudscript_rag::{DistanceMetric, StoreError, VectorStore, VectorStoreConfig};

fn test_config(dims: usize) -> VectorStoreConfig {
    VectorStoreConfig {
        name: "test-store".to_string(),
        dimensions: dims,
        distance_metric: DistanceMetric::Cosine,
        persist_path: None,
    }
}

#[test]
fn test_create_store() {
    let store = VectorStore::new(test_config(4)).unwrap();
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
}

#[test]
fn test_invalid_config() {
    let result = VectorStore::new(test_config(0));
    assert!(result.is_err());
}

#[test]
fn test_insert_and_query() {
    let mut store = VectorStore::new(test_config(4)).unwrap();

    let id1 = store
        .insert(
            "hello world",
            vec![1.0, 0.0, 0.0, 0.0],
            serde_json::json!({"source": "test"}),
        )
        .unwrap();
    let id2 = store
        .insert(
            "goodbye world",
            vec![0.0, 1.0, 0.0, 0.0],
            serde_json::json!({}),
        )
        .unwrap();

    assert_eq!(store.len(), 2);
    assert_ne!(id1, id2);

    // Query for nearest to [1, 0, 0, 0]
    let results = store.query(&[1.0, 0.0, 0.0, 0.0], 2).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].text, "hello world");
    assert_eq!(results[0].id, id1);
}

#[test]
fn test_insert_dimension_mismatch() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    let result = store.insert("oops", vec![1.0, 0.0], serde_json::json!({}));
    assert!(result.is_err());
}

#[test]
fn test_query_dimension_mismatch() {
    let store = VectorStore::new(test_config(4)).unwrap();
    let result = store.query(&[1.0], 1);
    assert!(result.is_err());
}

#[test]
fn test_delete() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    let id = store
        .insert(
            "to be deleted",
            vec![1.0, 0.0, 0.0, 0.0],
            serde_json::json!({}),
        )
        .unwrap();
    assert_eq!(store.len(), 1);

    assert!(store.delete(&id));
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());

    // Deleting again should return false
    assert!(!store.delete(&id));
}

#[test]
fn test_delete_nonexistent() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    assert!(!store.delete("nonexistent-id"));
}

#[test]
fn test_query_after_delete() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    let id1 = store
        .insert("first", vec![1.0, 0.0, 0.0, 0.0], serde_json::json!({}))
        .unwrap();
    store
        .insert("second", vec![0.0, 1.0, 0.0, 0.0], serde_json::json!({}))
        .unwrap();

    store.delete(&id1);

    let results = store.query(&[1.0, 0.0, 0.0, 0.0], 5).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].text, "second");
}

#[test]
fn test_metadata_preserved() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    let meta = serde_json::json!({"key": "value", "number": 42});
    store
        .insert("with meta", vec![1.0, 0.0, 0.0, 0.0], meta.clone())
        .unwrap();

    let results = store.query(&[1.0, 0.0, 0.0, 0.0], 1).unwrap();
    assert_eq!(results[0].metadata, meta);
}

#[test]
fn test_save_and_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_store.bin");

    let mut store = VectorStore::new(test_config(4)).unwrap();
    store
        .insert(
            "persisted entry",
            vec![1.0, 0.0, 0.0, 0.0],
            serde_json::json!({"persisted": true}),
        )
        .unwrap();
    store
        .insert(
            "another entry",
            vec![0.0, 1.0, 0.0, 0.0],
            serde_json::json!({}),
        )
        .unwrap();

    store.save_to_disk(&path).unwrap();

    let loaded = VectorStore::load_from_disk(&path).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.config().name, "test-store");

    let results = loaded.query(&[1.0, 0.0, 0.0, 0.0], 1).unwrap();
    assert_eq!(results[0].text, "persisted entry");
    assert_eq!(results[0].metadata["persisted"], true);
}

#[test]
fn test_load_nonexistent_path() {
    let result = VectorStore::load_from_disk("/nonexistent/path/store.bin");
    assert!(result.is_err());
}

#[test]
fn test_save_to_disk_invalid_path() {
    let store = VectorStore::new(test_config(4)).unwrap();
    let result = store.save_to_disk("/nonexistent/deep/path/store.bin");
    assert!(matches!(result, Err(StoreError::PersistError(_))));
}

#[test]
fn test_load_invalid_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.bin");
    std::fs::write(&path, b"not valid json").unwrap();
    let result = VectorStore::load_from_disk(&path);
    assert!(matches!(result, Err(StoreError::PersistError(_))));
}

#[test]
fn test_config_accessor() {
    let store = VectorStore::new(test_config(8)).unwrap();
    assert_eq!(store.config().dimensions, 8);
    assert_eq!(store.config().name, "test-store");
    assert_eq!(store.config().distance_metric, DistanceMetric::Cosine);
}

#[test]
fn test_store_error_display() {
    let e = StoreError::DimensionMismatch {
        expected: 4,
        got: 2,
    };
    assert!(format!("{}", e).contains("dimension mismatch: expected 4, got 2"));

    let e = StoreError::NotFound("abc".to_string());
    assert!(format!("{}", e).contains("entry not found: abc"));

    let e = StoreError::PersistError("disk full".to_string());
    assert!(format!("{}", e).contains("persistence error: disk full"));

    let e = StoreError::InvalidConfig("bad".to_string());
    assert!(format!("{}", e).contains("invalid configuration: bad"));
}

#[test]
fn test_multiple_inserts_and_queries() {
    let mut store = VectorStore::new(test_config(4)).unwrap();

    // Insert several entries
    for i in 0..20 {
        let mut v = vec![0.0f32; 4];
        v[i % 4] = 1.0;
        store
            .insert(&format!("entry {}", i), v, serde_json::json!({"index": i}))
            .unwrap();
    }

    assert_eq!(store.len(), 20);

    let results = store.query(&[1.0, 0.0, 0.0, 0.0], 3).unwrap();
    assert_eq!(results.len(), 3);
    // All top-3 should be entries that had v[0] = 1.0
    for r in &results {
        assert!(
            r.score < 0.01,
            "expected near-zero distance, got {}",
            r.score
        );
    }
}
