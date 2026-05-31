//! Redis-compatible caching for features and predictions
//!
//! Uses an in-memory HashMap with TTL expiration as the default backend.
//! The `CacheBackend` trait allows swapping in a real Redis client at runtime.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Trait for pluggable cache backends.
#[async_trait]
pub trait CacheBackend: Send + Sync {
    async fn get_raw(&self, key: &str) -> Result<Option<String>>;
    async fn set_raw(&self, key: &str, value: &str, ttl: Duration) -> Result<()>;
    async fn invalidate(&self, key: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: Instant,
}

/// Default in-memory cache with TTL support.
#[derive(Debug, Clone)]
pub struct InMemoryCacheBackend {
    data: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl InMemoryCacheBackend {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryCacheBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CacheBackend for InMemoryCacheBackend {
    async fn get_raw(&self, key: &str) -> Result<Option<String>> {
        let mut map = self.data.write().await;
        if let Some(entry) = map.get(key) {
            if Instant::now() < entry.expires_at {
                return Ok(Some(entry.value.clone()));
            }
            map.remove(key);
        }
        Ok(None)
    }

    async fn set_raw(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        let mut map = self.data.write().await;
        map.insert(
            key.to_string(),
            CacheEntry {
                value: value.to_string(),
                expires_at: Instant::now() + ttl,
            },
        );
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<()> {
        let mut map = self.data.write().await;
        map.remove(key);
        Ok(())
    }
}

/// Cache layer for ML features and predictions.
pub struct TokenomicsCache {
    backend: Box<dyn CacheBackend>,
}

impl TokenomicsCache {
    pub fn new(_redis_url: Option<String>) -> Self {
        Self {
            backend: Box::new(InMemoryCacheBackend::new()),
        }
    }

    /// Create a cache layer with a custom backend.
    pub fn with_backend(backend: Box<dyn CacheBackend>) -> Self {
        Self { backend }
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.backend.get_raw(key).await? {
            Some(raw) => {
                let value: T = serde_json::from_str(&raw)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()> {
        let raw = serde_json::to_string(value)?;
        self.backend
            .set_raw(key, &raw, Duration::from_secs(ttl_seconds))
            .await
    }

    pub async fn invalidate(&self, key: &str) -> Result<()> {
        self.backend.invalidate(key).await
    }
}
