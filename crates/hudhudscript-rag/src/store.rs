use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::hnsw::HnswIndex;

/// Distance metric for vector comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

/// Configuration for a `VectorStore`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreConfig {
    /// Human-readable name for the store.
    pub name: String,
    /// Dimensionality of the vectors.
    pub dimensions: usize,
    /// Distance metric for similarity computation.
    pub distance_metric: DistanceMetric,
    /// Optional path for on-disk persistence.
    pub persist_path: Option<String>,
}

/// A search result from the vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Unique identifier for this entry.
    pub id: String,
    /// Similarity score (lower = more similar for distance metrics).
    pub score: f32,
    /// The original text.
    pub text: String,
    /// Arbitrary metadata attached to this entry.
    pub metadata: serde_json::Value,
}

/// Errors that can occur in the vector store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    DimensionMismatch { expected: usize, got: usize },
    NotFound(String),
    PersistError(String),
    InvalidConfig(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            StoreError::DimensionMismatch { expected, got } => {
                write!(f, "dimension mismatch: expected {}, got {}", expected, got)
            }
            StoreError::NotFound(s) => write!(f, "entry not found: {}", s),
            StoreError::PersistError(s) => write!(f, "persistence error: {}", s),
            StoreError::InvalidConfig(s) => write!(f, "invalid configuration: {}", s),
        }
    }
}

impl std::error::Error for StoreError {}

/// Internal record for a stored entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoreEntry {
    id: String,
    text: String,
    hnsw_index: usize,
    metadata: serde_json::Value,
}

/// Serializable snapshot of the entire store (for persistence).
#[derive(Debug, Serialize, Deserialize)]
struct StoreSnapshot {
    config: VectorStoreConfig,
    index: HnswIndex,
    entries: HashMap<String, StoreEntry>,
    id_to_hnsw: HashMap<String, usize>,
}

/// A vector store backed by an HNSW index.
///
/// Stores text + metadata alongside their vector representations and supports
/// insert, query, delete, and disk persistence.
pub struct VectorStore {
    config: VectorStoreConfig,
    index: HnswIndex,
    entries: HashMap<String, StoreEntry>,
    id_to_hnsw: HashMap<String, usize>,
}

impl VectorStore {
    /// Create a new vector store with the given configuration.
    #[must_use = "this returns a Result that should be checked"]
    pub fn new(config: VectorStoreConfig) -> Result<Self, StoreError> {
        if config.dimensions == 0 {
            return Err(StoreError::InvalidConfig(
                "dimensions must be > 0".to_string(),
            ));
        }
        let index = HnswIndex::new(config.dimensions, 16, 200);
        Ok(Self {
            config,
            index,
            entries: HashMap::new(),
            id_to_hnsw: HashMap::new(),
        })
    }

    /// Insert a text with its vector and metadata. Returns the UUID of the entry.
    #[must_use = "this returns a Result that should be checked"]
    pub fn insert(
        &mut self,
        text: &str,
        vector: Vec<f32>,
        metadata: serde_json::Value,
    ) -> Result<String, StoreError> {
        if vector.len() != self.config.dimensions {
            return Err(StoreError::DimensionMismatch {
                expected: self.config.dimensions,
                got: vector.len(),
            });
        }

        let id = uuid::Uuid::new_v4().to_string();
        let hnsw_idx = self.index.insert(vector);

        let entry = StoreEntry {
            id: id.clone(),
            text: text.to_string(),
            hnsw_index: hnsw_idx,
            metadata,
        };

        self.entries.insert(id.clone(), entry);
        self.id_to_hnsw.insert(id.clone(), hnsw_idx);
        Ok(id)
    }

    /// Query the store for the `top_k` most similar vectors.
    #[must_use = "this returns a Result that should be checked"]
    pub fn query(&self, vector: &[f32], top_k: usize) -> Result<Vec<SearchResult>, StoreError> {
        if vector.len() != self.config.dimensions {
            return Err(StoreError::DimensionMismatch {
                expected: self.config.dimensions,
                got: vector.len(),
            });
        }

        let raw_results = self.index.search(vector, top_k);

        // Build a reverse map: hnsw_index -> entry
        let hnsw_to_entry: HashMap<usize, &StoreEntry> =
            self.entries.values().map(|e| (e.hnsw_index, e)).collect();

        let mut results = Vec::new();
        for (hnsw_idx, dist) in raw_results {
            if let Some(entry) = hnsw_to_entry.get(&hnsw_idx) {
                results.push(SearchResult {
                    id: entry.id.clone(),
                    score: dist,
                    text: entry.text.clone(),
                    metadata: entry.metadata.clone(),
                });
            }
        }

        Ok(results)
    }

    /// Delete an entry by its UUID. Returns true if the entry was found and deleted.
    pub fn delete(&mut self, id: &str) -> bool {
        if let Some(entry) = self.entries.remove(id) {
            self.index.delete(entry.hnsw_index);
            self.id_to_hnsw.remove(id);
            true
        } else {
            false
        }
    }

    /// Save the store to disk at the given path.
    pub fn save_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<(), StoreError> {
        let snapshot = StoreSnapshot {
            config: self.config.clone(),
            index: self.index.clone(),
            entries: self.entries.clone(),
            id_to_hnsw: self.id_to_hnsw.clone(),
        };
        let json =
            serde_json::to_vec(&snapshot).map_err(|e| StoreError::PersistError(e.to_string()))?;
        fs::write(path, json).map_err(|e| StoreError::PersistError(e.to_string()))?;
        Ok(())
    }

    /// Load a store from disk.
    pub fn load_from_disk<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let bytes = fs::read(path).map_err(|e| StoreError::PersistError(e.to_string()))?;
        let snapshot: StoreSnapshot =
            serde_json::from_slice(&bytes).map_err(|e| StoreError::PersistError(e.to_string()))?;
        Ok(Self {
            config: snapshot.config,
            index: snapshot.index,
            entries: snapshot.entries,
            id_to_hnsw: snapshot.id_to_hnsw,
        })
    }

    /// Return the number of entries in the store.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the store configuration.
    pub fn config(&self) -> &VectorStoreConfig {
        &self.config
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl StoreError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            StoreError::DimensionMismatch { .. } => {
                hudhudscript_errors::ErrorCode::StoreDimensionMismatch
            }
            StoreError::InvalidConfig(..) => hudhudscript_errors::ErrorCode::StoreInvalidConfig,
            StoreError::NotFound(..) => hudhudscript_errors::ErrorCode::StoreNotFound,
            StoreError::PersistError(..) => hudhudscript_errors::ErrorCode::StorePersistError,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<StoreError> for hudhudscript_errors::Error {
    fn from(e: StoreError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
