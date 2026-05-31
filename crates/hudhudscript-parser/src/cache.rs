//! Parse result caching

use hudhudscript_ast::Stmt;
use hudhudscript_utils::SimpleLruCache;
use std::sync::{Arc, RwLock};

/// Parse result cache
pub struct ParseCache {
    cache: Arc<RwLock<SimpleLruCache<Vec<Stmt>>>>,
}

impl ParseCache {
    /// Create a new parse cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(SimpleLruCache::new(max_size))),
        }
    }

    /// Get cached result or None
    pub fn get(&self, source: &str) -> Option<Vec<Stmt>> {
        let cache = self.cache.read().ok()?;
        cache.get(source).cloned()
    }

    /// Insert result into cache
    pub fn insert(&self, source: String, ast: Vec<Stmt>) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(source, ast);
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new(100)
    }
}
