//! HudHudScript RAG: Built-in vector memory for Subject-Oriented Programming
//!
//! Provides HNSW-based vector indexing for subject memory, relation-scoped
//! shared memory, and recall/remember/forget operations.
//!
//! Production-ready features:
//! - **Chunking**: Fixed-size, semantic (paragraph), and recursive strategies
//! - **Embedding providers**: Trait abstraction with mock and API providers
//! - **Hybrid search**: Combined vector cosine similarity + BM25 keyword scoring
//! - **Document model**: Format-aware processing (plain text, markdown, code)
//! - **Document index**: Incremental add/update/delete with hybrid search

pub mod chunking;
pub mod document;
pub mod embedding;
pub mod hnsw;
pub mod index;
pub mod provider;
pub mod search;
pub mod store;

pub use chunking::{Chunk, ChunkStrategy, Chunker};
pub use document::{Document, DocumentFormat};
pub use embedding::{EmbeddingError, EmbeddingProvider, SimpleEmbedding};
pub use hnsw::HnswIndex;
pub use index::{DocumentIndex, IndexError};
pub use provider::{ApiProvider, ApiProviderConfig, MockProvider};
pub use search::{Bm25Params, HybridSearch, HybridSearchResult, HybridWeights};
pub use store::{DistanceMetric, SearchResult, StoreError, VectorStore, VectorStoreConfig};
