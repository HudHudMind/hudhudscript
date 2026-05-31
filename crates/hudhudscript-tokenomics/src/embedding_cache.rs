//! Embedding cache with dimensionality reduction
//!
//! Caches embedding vectors to avoid redundant embedding API calls. Supports
//! Matryoshka-style dimensionality reduction (truncation + re-normalization)
//! and INT8 quantization for storage savings.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

/// A cached embedding vector with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEmbedding {
    pub text_hash: u64,
    pub embedding: Vec<f32>,
    pub model_version: String,
    pub dimensions: usize,
    pub created_at: DateTime<Utc>,
}

/// Aggregate statistics for the embedding cache.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmbeddingCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
    pub storage_bytes_saved: u64,
}

/// In-memory embedding cache with optional dimensionality reduction and
/// INT8 quantization.
#[derive(Debug)]
pub struct EmbeddingCache {
    cache: HashMap<u64, CachedEmbedding>,
    model_version: String,
    target_dimensions: Option<usize>,
    quantize: bool,
    max_entries: usize,
    stats: EmbeddingCacheStats,
}

impl EmbeddingCache {
    /// Create a new embedding cache.
    ///
    /// - `model_version` — current model version; lookups for stale versions
    ///   return `None`.
    /// - `target_dimensions` — if `Some(n)`, embeddings are truncated to `n`
    ///   dimensions via Matryoshka reduction on insert.
    /// - `quantize` — if `true`, embeddings are quantized to INT8 before
    ///   storage (lossy).
    /// - `max_entries` — maximum number of cached embeddings.
    pub fn new(
        model_version: String,
        target_dimensions: Option<usize>,
        quantize: bool,
        max_entries: usize,
    ) -> Self {
        Self {
            cache: HashMap::new(),
            model_version,
            target_dimensions,
            quantize,
            max_entries,
            stats: EmbeddingCacheStats::default(),
        }
    }

    /// Look up a cached embedding by text hash. Returns `None` if the cached
    /// entry's model version does not match the current version.
    pub fn get(&mut self, text_hash: u64) -> Option<&CachedEmbedding> {
        if let Some(entry) = self.cache.get(&text_hash) {
            if entry.model_version == self.model_version {
                self.stats.hits += 1;
                return Some(entry);
            }
            // Stale version — treat as miss.
        }
        self.stats.misses += 1;
        None
    }

    /// Insert an embedding into the cache. Applies dimensionality reduction
    /// and/or quantization if configured.
    pub fn put(&mut self, text_hash: u64, mut embedding: Vec<f32>) {
        // Evict oldest entry if at capacity (simple FIFO by insertion order of HashMap).
        if self.cache.len() >= self.max_entries && !self.cache.contains_key(&text_hash) {
            if let Some(&oldest_key) = self.cache.keys().next() {
                self.cache.remove(&oldest_key);
            }
        }

        let original_bytes = (embedding.len() * 4) as u64;

        // Matryoshka reduction
        if let Some(target) = self.target_dimensions {
            embedding = reduce_dimensions(&embedding, target);
        }

        // Optional INT8 quantization (store as f32 representation of quantized values)
        if self.quantize {
            let quantized = quantize_int8(&embedding);
            embedding = dequantize_int8(&quantized);
        }

        let stored_bytes = (embedding.len() * 4) as u64;
        if original_bytes > stored_bytes {
            self.stats.storage_bytes_saved += original_bytes - stored_bytes;
        }

        let dims = embedding.len();
        self.cache.insert(
            text_hash,
            CachedEmbedding {
                text_hash,
                embedding,
                model_version: self.model_version.clone(),
                dimensions: dims,
                created_at: Utc::now(),
            },
        );
    }

    /// Remove all entries whose model version matches `old_version`.
    pub fn invalidate_version(&mut self, old_version: &str) {
        let keys_to_remove: Vec<u64> = self
            .cache
            .iter()
            .filter(|(_, v)| v.model_version == old_version)
            .map(|(k, _)| *k)
            .collect();

        let count = keys_to_remove.len() as u64;
        for key in keys_to_remove {
            self.cache.remove(&key);
        }
        self.stats.invalidations += count;
    }

    /// Return current cache statistics.
    pub fn stats(&self) -> &EmbeddingCacheStats {
        &self.stats
    }

    /// Number of entries in the cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Compute a hash for the given text using `DefaultHasher`.
pub fn text_hash(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Matryoshka dimensionality reduction: truncate to `target` dimensions and
/// re-normalize to unit length.
pub fn reduce_dimensions(embedding: &[f32], target: usize) -> Vec<f32> {
    let truncated: Vec<f32> = embedding.iter().take(target).copied().collect();
    let norm: f64 = truncated
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    if norm == 0.0 {
        return truncated;
    }
    truncated
        .iter()
        .map(|x| (*x as f64 / norm) as f32)
        .collect()
}

/// Quantize f32 values to INT8 range [-127, 127].
pub fn quantize_int8(embedding: &[f32]) -> Vec<i8> {
    if embedding.is_empty() {
        return Vec::new();
    }
    let max_abs = embedding.iter().map(|x| x.abs()).fold(0.0_f32, f32::max);
    if max_abs == 0.0 {
        return vec![0i8; embedding.len()];
    }
    embedding
        .iter()
        .map(|x| (x / max_abs * 127.0).round().clamp(-127.0, 127.0) as i8)
        .collect()
}

/// Dequantize INT8 values back to f32. Note: this is lossy — the original
/// scale is not preserved, so values are normalized to approximately [-1, 1].
pub fn dequantize_int8(quantized: &[i8]) -> Vec<f32> {
    quantized.iter().map(|x| *x as f32 / 127.0).collect()
}

/// Cosine similarity between two vectors.
///
/// Re-exports the canonical implementation from `response_cache`.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    crate::response_cache::cosine_similarity(a, b)
}
