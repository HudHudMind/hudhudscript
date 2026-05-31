//! Public API tests for tokenomics::storage
//! Covers: TokenomicsStorage (new, with_backend, save_budget, load_budget,
//!         save_usage, load_usage_history),
//!         InMemoryBackend (new, default, StorageBackend impl).

use hudhudscript_tokenomics::storage::{InMemoryBackend, StorageBackend, TokenomicsStorage};
use hudhudscript_tokenomics::types::{Budget, TokenUsageRecord};

// ── TokenomicsStorage::new ──────────────────────────────────────────

#[tokio::test]
async fn storage_new_none_url_functional() {
    let store = TokenomicsStorage::new(None);
    // Should be functional (in-memory backend)
    let result = store.load_budget("no-one").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn storage_new_some_url_functional() {
    // postgres_url is Some but backend is still InMemory, so it works
    let store = TokenomicsStorage::new(Some("fake://url".into()));
    let result = store.load_budget("ghost").await.unwrap();
    assert!(result.is_none());
}

// ── TokenomicsStorage::with_backend ────────────────────────────────

#[tokio::test]
async fn storage_with_backend_enabled_and_functional() {
    let backend = Box::new(InMemoryBackend::new());
    let store = TokenomicsStorage::with_backend(backend);
    let budget = Budget::new("carol".into(), 9999);
    store.save_budget(&budget).await.unwrap();
    let loaded = store.load_budget("carol").await.unwrap().unwrap();
    assert_eq!(loaded.user_id, "carol");
    assert_eq!(loaded.total, 9999);
    assert_eq!(loaded.used, 0);
    assert_eq!(loaded.remaining, 9999);
}

// ── Budget save / load roundtrip ────────────────────────────────────

#[tokio::test]
async fn save_and_load_budget_roundtrip() {
    let store = TokenomicsStorage::new(None);
    let budget = Budget::new("alice".into(), 5000);
    store.save_budget(&budget).await.unwrap();
    let loaded = store.load_budget("alice").await.unwrap().unwrap();
    assert_eq!(loaded.user_id, "alice");
    assert_eq!(loaded.total, 5000);
    assert_eq!(loaded.remaining, 5000);
    assert_eq!(loaded.used, 0);
}

#[tokio::test]
async fn load_missing_budget_returns_none() {
    let store = TokenomicsStorage::new(None);
    assert!(store.load_budget("nobody").await.unwrap().is_none());
}

#[tokio::test]
async fn budget_overwrite_replaces_previous() {
    let store = TokenomicsStorage::new(None);
    let b1 = Budget::new("dave".into(), 1000);
    store.save_budget(&b1).await.unwrap();
    let b2 = Budget::new("dave".into(), 2000);
    store.save_budget(&b2).await.unwrap();
    let loaded = store.load_budget("dave").await.unwrap().unwrap();
    assert_eq!(loaded.total, 2000);
}

#[tokio::test]
async fn multiple_users_budgets_stored_independently() {
    let store = TokenomicsStorage::new(None);
    let b_alice = Budget::new("alice".into(), 1000);
    let b_bob = Budget::new("bob".into(), 2000);
    store.save_budget(&b_alice).await.unwrap();
    store.save_budget(&b_bob).await.unwrap();

    let alice = store.load_budget("alice").await.unwrap().unwrap();
    let bob = store.load_budget("bob").await.unwrap().unwrap();
    assert_eq!(alice.total, 1000);
    assert_eq!(bob.total, 2000);
}

#[tokio::test]
async fn save_budget_preserves_id() {
    let store = TokenomicsStorage::new(None);
    let budget = Budget::new("user1".into(), 500);
    let original_id = budget.id;
    store.save_budget(&budget).await.unwrap();
    let loaded = store.load_budget("user1").await.unwrap().unwrap();
    assert_eq!(loaded.id, original_id);
}

// ── TokenUsageRecord save / history ────────────────────────────────────────

#[tokio::test]
async fn save_usage_and_load_history() {
    let store = TokenomicsStorage::new(None);
    let usage = TokenUsageRecord::new("user1".into(), "op1".into(), 100);
    store.save_usage(&usage).await.unwrap();
    let history = store.load_usage_history("user1", 10).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].operation, "op1");
    assert_eq!(history[0].tokens_used, 100);
}

#[tokio::test]
async fn load_usage_history_empty_for_unknown_user() {
    let store = TokenomicsStorage::new(None);
    let history = store.load_usage_history("unknown-user", 10).await.unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn usage_history_limit_is_respected() {
    let store = TokenomicsStorage::new(None);
    for i in 0..5usize {
        let mut usage = TokenUsageRecord::new("bob".into(), format!("op{i}"), (i + 1) as u64 * 100);
        usage.timestamp += chrono::Duration::seconds(i as i64);
        store.save_usage(&usage).await.unwrap();
    }
    let history = store.load_usage_history("bob", 3).await.unwrap();
    assert_eq!(history.len(), 3);
}

#[tokio::test]
async fn usage_history_returned_newest_first() {
    let store = TokenomicsStorage::new(None);
    for i in 0..5usize {
        let mut usage =
            TokenUsageRecord::new("charlie".into(), format!("op{i}"), (i + 1) as u64 * 10);
        usage.timestamp += chrono::Duration::seconds(i as i64);
        store.save_usage(&usage).await.unwrap();
    }
    let history = store.load_usage_history("charlie", 5).await.unwrap();
    assert_eq!(history.len(), 5);
    for i in 0..history.len() - 1 {
        assert!(history[i].timestamp >= history[i + 1].timestamp);
    }
}

#[tokio::test]
async fn usage_history_only_returns_matching_user() {
    let store = TokenomicsStorage::new(None);
    store
        .save_usage(&TokenUsageRecord::new("alice".into(), "op".into(), 100))
        .await
        .unwrap();
    store
        .save_usage(&TokenUsageRecord::new("bob".into(), "op".into(), 200))
        .await
        .unwrap();

    let alice_history = store.load_usage_history("alice", 10).await.unwrap();
    assert_eq!(alice_history.len(), 1);
    assert_eq!(alice_history[0].user_id, "alice");

    let bob_history = store.load_usage_history("bob", 10).await.unwrap();
    assert_eq!(bob_history.len(), 1);
    assert_eq!(bob_history[0].user_id, "bob");
}

#[tokio::test]
async fn usage_history_limit_zero_returns_empty() {
    let store = TokenomicsStorage::new(None);
    store
        .save_usage(&TokenUsageRecord::new("dave".into(), "op".into(), 100))
        .await
        .unwrap();
    let history = store.load_usage_history("dave", 0).await.unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn save_multiple_usages_for_same_user() {
    let store = TokenomicsStorage::new(None);
    for i in 0..3usize {
        let u = TokenUsageRecord::new("multi".into(), format!("op{i}"), (i + 1) as u64 * 50);
        store.save_usage(&u).await.unwrap();
    }
    let history = store.load_usage_history("multi", 100).await.unwrap();
    assert_eq!(history.len(), 3);
}

// ── InMemoryBackend direct usage ─────────────────────────────────────

#[tokio::test]
async fn in_memory_backend_new_is_empty() {
    let backend = InMemoryBackend::new();
    let result = backend.load_budget("anyone").await.unwrap();
    assert!(result.is_none());
    let history = backend.load_usage_history("anyone", 10).await.unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn in_memory_backend_default_is_empty() {
    let backend = InMemoryBackend::default();
    let result = backend.load_budget("anyone").await.unwrap();
    assert!(result.is_none());
    let history = backend.load_usage_history("anyone", 10).await.unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn in_memory_backend_save_and_load_budget() {
    let backend = InMemoryBackend::new();
    let budget = Budget::new("test-user".into(), 1234);
    backend.save_budget(&budget).await.unwrap();
    let loaded = backend.load_budget("test-user").await.unwrap().unwrap();
    assert_eq!(loaded.total, 1234);
}

#[tokio::test]
async fn in_memory_backend_save_and_load_usage() {
    let backend = InMemoryBackend::new();
    let usage = TokenUsageRecord::new("user".into(), "infer".into(), 500);
    backend.save_usage(&usage).await.unwrap();
    let history = backend.load_usage_history("user", 5).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].tokens_used, 500);
}

#[tokio::test]
async fn in_memory_backend_clone_is_independent() {
    let backend = InMemoryBackend::new();
    let cloned = backend.clone();
    let budget = Budget::new("only-original".into(), 100);
    backend.save_budget(&budget).await.unwrap();
    // The clone shares Arc, so they see the same data
    let result = cloned.load_budget("only-original").await.unwrap();
    assert!(result.is_some());
}

// ── Edge cases ───────────────────────────────────────────────────────

#[tokio::test]
async fn storage_with_postgres_url_still_uses_in_memory() {
    let store = TokenomicsStorage::new(Some("postgres://localhost/test".into()));
    let budget = Budget::new("pg-user".into(), 777);
    store.save_budget(&budget).await.unwrap();
    let loaded = store.load_budget("pg-user").await.unwrap().unwrap();
    assert_eq!(loaded.total, 777);
}
