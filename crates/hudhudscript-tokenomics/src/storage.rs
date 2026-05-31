//! Database persistence for tokenomics data
//!
//! Uses an in-memory BTreeMap as the default backend. The `StorageBackend` trait
//! allows swapping in a real database (PostgreSQL/TimescaleDB) at runtime.

use crate::error::Result;
use crate::types::{Budget, TokenUsageRecord};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for pluggable storage backends.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Persist a budget record.
    async fn save_budget(&self, budget: &Budget) -> Result<()>;
    /// Load the most recent budget for a user.
    async fn load_budget(&self, user_id: &str) -> Result<Option<Budget>>;
    /// Persist a token-usage record.
    async fn save_usage(&self, usage: &TokenUsageRecord) -> Result<()>;
    /// Load the most recent `limit` usage records for a user (newest first).
    async fn load_usage_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<TokenUsageRecord>>;
}

// ── In-memory backend ────────────────────────────────────────────────

/// Default in-memory backend backed by `BTreeMap`.
#[derive(Debug, Clone)]
pub struct InMemoryBackend {
    budgets: Arc<RwLock<BTreeMap<String, Budget>>>,
    usage: Arc<RwLock<BTreeMap<(String, i64, String), TokenUsageRecord>>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            budgets: Arc::new(RwLock::new(BTreeMap::new())),
            usage: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for InMemoryBackend {
    async fn save_budget(&self, budget: &Budget) -> Result<()> {
        let mut map = self.budgets.write().await;
        map.insert(budget.user_id.clone(), budget.clone());
        Ok(())
    }

    async fn load_budget(&self, user_id: &str) -> Result<Option<Budget>> {
        let map = self.budgets.read().await;
        Ok(map.get(user_id).cloned())
    }

    async fn save_usage(&self, usage: &TokenUsageRecord) -> Result<()> {
        let key = (
            usage.user_id.clone(),
            usage.timestamp.timestamp_nanos_opt().unwrap_or(0),
            usage.id.to_string(),
        );
        let mut map = self.usage.write().await;
        map.insert(key, usage.clone());
        Ok(())
    }

    async fn load_usage_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<TokenUsageRecord>> {
        let map = self.usage.read().await;
        let mut records: Vec<TokenUsageRecord> = map
            .iter()
            .filter(|((uid, _, _), _)| uid == user_id)
            .map(|(_, v)| v.clone())
            .collect();
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.truncate(limit);
        Ok(records)
    }
}

// ── File-based backend ──────────────────────────────────────────────

/// JSON-file-based storage backend.
///
/// Layout on disk:
/// ```text
/// {base_dir}/
///   budgets/{user_id}.json          — latest Budget per user
///   usage/{user_id}.jsonl           — one TokenUsageRecord per line (append-only)
/// ```
#[derive(Debug, Clone)]
pub struct FileStorageBackend {
    base_dir: std::path::PathBuf,
}

impl FileStorageBackend {
    /// Create a new `FileStorageBackend` rooted at `base_dir`.
    ///
    /// The directory tree is created lazily on first write.
    pub fn new(base_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Convenience constructor that stores data under `~/.hudhud/tokenomics`.
    pub fn default_path() -> std::io::Result<Self> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
        Ok(Self::new(
            std::path::PathBuf::from(home)
                .join(".hudhud")
                .join("tokenomics"),
        ))
    }

    fn budgets_dir(&self) -> std::path::PathBuf {
        self.base_dir.join("budgets")
    }

    fn usage_dir(&self) -> std::path::PathBuf {
        self.base_dir.join("usage")
    }

    fn budget_path(&self, user_id: &str) -> std::path::PathBuf {
        self.budgets_dir().join(format!("{}.json", user_id))
    }

    fn usage_path(&self, user_id: &str) -> std::path::PathBuf {
        self.usage_dir().join(format!("{}.jsonl", user_id))
    }
}

#[async_trait]
impl StorageBackend for FileStorageBackend {
    async fn save_budget(&self, budget: &Budget) -> Result<()> {
        let dir = self.budgets_dir();
        tokio::fs::create_dir_all(&dir).await?;
        let path = self.budget_path(&budget.user_id);
        let json = serde_json::to_string_pretty(budget)?;
        tokio::fs::write(&path, json.as_bytes()).await?;
        Ok(())
    }

    async fn load_budget(&self, user_id: &str) -> Result<Option<Budget>> {
        let path = self.budget_path(user_id);
        match tokio::fs::read_to_string(&path).await {
            Ok(contents) => {
                let budget: Budget = serde_json::from_str(&contents)?;
                Ok(Some(budget))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn save_usage(&self, usage: &TokenUsageRecord) -> Result<()> {
        let dir = self.usage_dir();
        tokio::fs::create_dir_all(&dir).await?;
        let path = self.usage_path(&usage.user_id);
        let mut line = serde_json::to_string(usage)?;
        line.push('\n');
        // Append to the JSONL file.
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        file.write_all(line.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    async fn load_usage_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<TokenUsageRecord>> {
        let path = self.usage_path(user_id);
        let contents = match tokio::fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        let mut records: Vec<TokenUsageRecord> = contents
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        // Newest first
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        records.truncate(limit);
        Ok(records)
    }
}

// ── Public facade ────────────────────────────────────────────────────

/// Storage layer for tokenomics data.
pub struct TokenomicsStorage {
    backend: Box<dyn StorageBackend>,
    _enabled: bool,
}

impl TokenomicsStorage {
    pub fn new(postgres_url: Option<String>) -> Self {
        Self {
            backend: Box::new(InMemoryBackend::new()),
            _enabled: postgres_url.is_some(),
        }
    }

    /// Create a storage layer with a custom backend.
    pub fn with_backend(backend: Box<dyn StorageBackend>) -> Self {
        Self {
            backend,
            _enabled: true,
        }
    }

    pub async fn save_budget(&self, budget: &Budget) -> Result<()> {
        self.backend.save_budget(budget).await
    }

    pub async fn load_budget(&self, user_id: &str) -> Result<Option<Budget>> {
        self.backend.load_budget(user_id).await
    }

    pub async fn save_usage(&self, usage: &TokenUsageRecord) -> Result<()> {
        self.backend.save_usage(usage).await
    }

    pub async fn load_usage_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<TokenUsageRecord>> {
        self.backend.load_usage_history(user_id, limit).await
    }
}
