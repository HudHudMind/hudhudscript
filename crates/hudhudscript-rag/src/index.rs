//! Document index with incremental add/update/delete and hybrid search.

use std::collections::HashMap;

use crate::chunking::ChunkStrategy;
use crate::document::Document;
use crate::embedding::{EmbeddingError, EmbeddingProvider};
use crate::hnsw::HnswIndex;
use crate::search::{Bm25Params, HybridSearch, HybridSearchResult, HybridWeights};

/// Errors that can occur in the document index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexError {
    NotFound(String),
    Embedding(EmbeddingError),
    DuplicateId(String),
}

impl std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            IndexError::NotFound(s) => write!(f, "document not found: {}", s),
            IndexError::Embedding(e) => write!(f, "embedding error: {}", e),
            IndexError::DuplicateId(s) => write!(f, "duplicate document id: {}", s),
        }
    }
}

impl std::error::Error for IndexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IndexError::Embedding(e) => Some(e),
            _ => None,
        }
    }
}

impl From<EmbeddingError> for IndexError {
    fn from(e: EmbeddingError) -> Self {
        IndexError::Embedding(e)
    }
}

/// Internal record for a chunk stored in the index.
#[derive(Debug, Clone)]
struct IndexedChunk {
    #[allow(dead_code)]
    doc_id: String,
    #[allow(dead_code)]
    chunk_text: String,
    hnsw_idx: usize,
}

/// A document index that supports incremental add/update/delete operations
/// and hybrid search (vector + BM25).
pub struct DocumentIndex {
    /// The HNSW vector index.
    hnsw: HnswIndex,
    /// Hybrid search engine for BM25.
    hybrid: HybridSearch,
    /// Mapping from HNSW index to chunk metadata.
    chunks: Vec<IndexedChunk>,
    /// Mapping from doc_id to list of chunk indices (positions in `chunks`).
    doc_chunks: HashMap<String, Vec<usize>>,
    /// Stored documents (for update support).
    documents: HashMap<String, Document>,
    /// The embedding provider.
    provider: Box<dyn EmbeddingProvider>,
    /// Optional default chunk strategy override.
    chunk_strategy: Option<ChunkStrategy>,
}

impl DocumentIndex {
    /// Create a new document index with the given embedding provider.
    pub fn new(provider: Box<dyn EmbeddingProvider>) -> Self {
        let dims = provider.dimensions();
        Self {
            hnsw: HnswIndex::new(dims, 16, 200),
            hybrid: HybridSearch::with_defaults(),
            chunks: Vec::new(),
            doc_chunks: HashMap::new(),
            documents: HashMap::new(),
            provider,
            chunk_strategy: None,
        }
    }

    /// Create a new document index with custom hybrid search weights.
    pub fn with_config(
        provider: Box<dyn EmbeddingProvider>,
        weights: HybridWeights,
        bm25_params: Bm25Params,
        chunk_strategy: Option<ChunkStrategy>,
    ) -> Self {
        let dims = provider.dimensions();
        Self {
            hnsw: HnswIndex::new(dims, 16, 200),
            hybrid: HybridSearch::new(weights, bm25_params),
            chunks: Vec::new(),
            doc_chunks: HashMap::new(),
            documents: HashMap::new(),
            provider,
            chunk_strategy,
        }
    }

    /// Add a document to the index. Returns the number of chunks indexed.
    pub fn add_document(&mut self, document: Document) -> Result<usize, IndexError> {
        if self.documents.contains_key(&document.id) {
            return Err(IndexError::DuplicateId(document.id.clone()));
        }

        let doc_id = document.id.clone();
        let chunks = document.chunk(self.chunk_strategy);
        let mut chunk_indices = Vec::new();

        for chunk in &chunks {
            let embedding = self.provider.embed(&chunk.text)?;
            let hnsw_idx = self.hnsw.insert(embedding);

            let chunk_idx = self.chunks.len();
            self.chunks.push(IndexedChunk {
                doc_id: doc_id.clone(),
                chunk_text: chunk.text.clone(),
                hnsw_idx,
            });

            self.hybrid.add_document(&doc_id, &chunk.text);
            chunk_indices.push(chunk_idx);
        }

        let num_chunks = chunk_indices.len();
        self.doc_chunks.insert(doc_id.clone(), chunk_indices);
        self.documents.insert(doc_id, document);

        Ok(num_chunks)
    }

    /// Update a document by replacing it with a new version.
    /// Removes old chunks and re-indexes the new content.
    pub fn update_document(&mut self, document: Document) -> Result<usize, IndexError> {
        let doc_id = document.id.clone();
        if !self.documents.contains_key(&doc_id) {
            return Err(IndexError::NotFound(doc_id));
        }

        // Remove old data
        self.remove_chunks_for_doc(&doc_id);
        self.documents.remove(&doc_id);

        // Re-add
        self.add_document(document)
    }

    /// Remove a document and all its chunks from the index.
    pub fn remove_document(&mut self, doc_id: &str) -> Result<(), IndexError> {
        if !self.documents.contains_key(doc_id) {
            return Err(IndexError::NotFound(doc_id.to_string()));
        }

        self.remove_chunks_for_doc(doc_id);
        self.documents.remove(doc_id);
        Ok(())
    }

    /// Internal: remove all chunks belonging to a document.
    fn remove_chunks_for_doc(&mut self, doc_id: &str) {
        if let Some(indices) = self.doc_chunks.remove(doc_id) {
            for &chunk_idx in &indices {
                if chunk_idx < self.chunks.len() {
                    let hnsw_idx = self.chunks[chunk_idx].hnsw_idx;
                    self.hnsw.delete(hnsw_idx);
                }
            }
        }
        self.hybrid.remove_document(doc_id);
    }

    /// Search the index using hybrid vector + keyword search.
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<HybridSearchResult>, IndexError> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Get vector results
        let query_embedding = self.provider.embed(query)?;
        let vector_raw = self.hnsw.search(&query_embedding, top_k * 3);

        // Map HNSW indices to hybrid search document indices
        // The hybrid search uses sequential indices matching the order documents
        // were added to it. We need to map HNSW results to those indices.
        let mut vector_results: Vec<(usize, f32)> = Vec::new();
        for (hnsw_idx, distance) in &vector_raw {
            // Find which chunk this HNSW index belongs to
            for (chunk_idx, chunk) in self.chunks.iter().enumerate() {
                if chunk.hnsw_idx == *hnsw_idx {
                    // Convert cosine distance to similarity
                    let similarity = 1.0 - distance;
                    vector_results.push((chunk_idx, similarity));
                    break;
                }
            }
        }

        let results = self.hybrid.search(query, &vector_results, top_k);
        Ok(results)
    }

    /// Return the number of documents in the index.
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Return the total number of indexed chunks.
    pub fn chunk_count(&self) -> usize {
        self.doc_chunks.values().map(|v| v.len()).sum()
    }

    /// Check if a document with the given id exists in the index.
    pub fn contains(&self, doc_id: &str) -> bool {
        self.documents.contains_key(doc_id)
    }

    /// Get a reference to a stored document by id.
    pub fn get_document(&self, doc_id: &str) -> Option<&Document> {
        self.documents.get(doc_id)
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl IndexError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            IndexError::DuplicateId(..) => hudhudscript_errors::ErrorCode::IndexDuplicateId,
            IndexError::Embedding(..) => hudhudscript_errors::ErrorCode::IndexEmbedding,
            IndexError::NotFound(..) => hudhudscript_errors::ErrorCode::IndexNotFound,
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

impl From<IndexError> for hudhudscript_errors::Error {
    fn from(e: IndexError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
