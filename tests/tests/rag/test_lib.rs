//! Public API tests for hudhudscript-rag

use hudhudscript_rag::{
    Bm25Params, ChunkStrategy, Chunker, DistanceMetric, Document, DocumentFormat, DocumentIndex,
    EmbeddingProvider, HnswIndex, HybridSearch, HybridWeights, IndexError, MockProvider,
    SimpleEmbedding, VectorStore, VectorStoreConfig,
};
use std::collections::HashMap;

// ── HybridSearch — creation ─────────────────────────────────────────

#[test]
fn hybrid_search_new_is_empty() {
    let hs = HybridSearch::with_defaults();
    assert!(hs.is_empty());
    assert_eq!(hs.len(), 0);
}

#[test]
fn hybrid_search_custom_weights() {
    let w = HybridWeights {
        vector_weight: 0.6,
        keyword_weight: 0.4,
    };
    let hs = HybridSearch::new(w, Bm25Params::default());
    let got = hs.weights();
    assert!((got.vector_weight - 0.6).abs() < 1e-5);
    assert!((got.keyword_weight - 0.4).abs() < 1e-5);
}

// ── HybridSearch — add / remove ─────────────────────────────────────

#[test]
fn hybrid_add_document_increases_len() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello world");
    assert_eq!(hs.len(), 1);
    assert!(!hs.is_empty());
}

#[test]
fn hybrid_add_multiple_documents() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("d1", "aaa");
    hs.add_document("d2", "bbb");
    hs.add_document("d3", "ccc");
    assert_eq!(hs.len(), 3);
}

#[test]
fn hybrid_remove_document() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello world");
    hs.add_document("doc2", "goodbye world");
    hs.remove_document("doc1");
    assert_eq!(hs.len(), 1);
}

#[test]
fn hybrid_remove_nonexistent_noop() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello");
    hs.remove_document("nonexistent");
    assert_eq!(hs.len(), 1);
}

#[test]
fn hybrid_remove_all_same_id() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello");
    hs.add_document("doc1", "world");
    hs.remove_document("doc1");
    assert_eq!(hs.len(), 0);
    assert!(hs.is_empty());
}

// ── HybridSearch — BM25 ────────────────────────────────────────────

#[test]
fn bm25_matching_docs_score_positive() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "the quick brown fox");
    hs.add_document("doc2", "quantum computing algorithms");
    let results = hs.bm25_search("brown fox");
    assert!(results[0].1 > 0.0);
    assert_eq!(results[0].0, 0);
}

#[test]
fn bm25_exact_match_first() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "rust programming language");
    hs.add_document("doc2", "python scripting language");
    let results = hs.bm25_search("rust programming");
    assert_eq!(results[0].0, 0);
}

#[test]
fn bm25_empty_query_all_zero() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello world");
    for (_, score) in hs.bm25_search("") {
        assert_eq!(score, 0.0);
    }
}

#[test]
fn bm25_no_matching_terms_zero() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "alpha beta gamma");
    for (_, score) in hs.bm25_search("zzz xyz") {
        assert_eq!(score, 0.0);
    }
}

// ── HybridSearch — combined search ──────────────────────────────────

#[test]
fn hybrid_search_combines_scores() {
    let mut hs = HybridSearch::new(
        HybridWeights {
            vector_weight: 0.5,
            keyword_weight: 0.5,
        },
        Bm25Params::default(),
    );
    hs.add_document("doc1", "machine learning models");
    hs.add_document("doc2", "deep learning neural networks");
    hs.add_document("doc3", "cooking recipes pasta");
    let vector_results = vec![(1, 0.9f32), (0, 0.7), (2, 0.1)];
    let results = hs.search("learning models", &vector_results, 3);
    assert_eq!(results.len(), 3);
    assert_eq!(results[2].doc_id, "doc3");
}

#[test]
fn hybrid_search_empty_vector_results() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "rust programming language");
    let results = hs.search("rust", &[], 3);
    assert!(!results.is_empty());
    assert_eq!(results[0].doc_id, "doc1");
}

// ── DocumentIndex — add / remove / search / update ──────────────────

fn make_index() -> DocumentIndex {
    DocumentIndex::new(Box::new(MockProvider::new(32)))
}

#[test]
fn index_add_document() {
    let mut idx = make_index();
    let doc = Document::new(
        "doc1",
        "Hello world. This is a test document.",
        DocumentFormat::PlainText,
    );
    let count = idx.add_document(doc).unwrap();
    assert!(count > 0);
    assert_eq!(idx.document_count(), 1);
    assert!(idx.contains("doc1"));
}

#[test]
fn index_add_duplicate_fails() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "doc1",
        "Content one.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    let result = idx.add_document(Document::new(
        "doc1",
        "Content two.",
        DocumentFormat::PlainText,
    ));
    assert!(result.is_err());
}

#[test]
fn index_remove_document() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "doc1",
        "Hello world.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    idx.remove_document("doc1").unwrap();
    assert_eq!(idx.document_count(), 0);
    assert!(!idx.contains("doc1"));
}

#[test]
fn index_remove_nonexistent_fails() {
    let mut idx = make_index();
    assert!(idx.remove_document("nonexistent").is_err());
}

#[test]
fn index_update_document() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "doc1",
        "Original.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    let count = idx
        .update_document(Document::new(
            "doc1",
            "Updated content here.",
            DocumentFormat::PlainText,
        ))
        .unwrap();
    assert!(count > 0);
    assert_eq!(idx.document_count(), 1);
    assert!(idx
        .get_document("doc1")
        .unwrap()
        .content
        .contains("Updated"));
}

#[test]
fn index_update_nonexistent_fails() {
    let mut idx = make_index();
    assert!(idx
        .update_document(Document::new("nope", "x.", DocumentFormat::PlainText))
        .is_err());
}

#[test]
fn index_search() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "d1",
        "Rust programming language systems.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    idx.add_document(Document::new(
        "d2",
        "Cooking recipes pasta Italian.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    let results = idx.search("programming language", 3).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn index_search_empty_query() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "d1",
        "Some content.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    assert!(idx.search("", 5).unwrap().is_empty());
}

#[test]
fn index_search_after_remove() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "d1",
        "Rust programming.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    idx.remove_document("d1").unwrap();
    assert!(idx.search("programming", 5).unwrap().is_empty());
}

#[test]
fn index_chunk_count() {
    let mut idx = make_index();
    let doc = Document::new(
        "d1",
        "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.",
        DocumentFormat::Markdown,
    );
    let count = idx.add_document(doc).unwrap();
    assert_eq!(idx.chunk_count(), count);
}

#[test]
fn index_contains_after_operations() {
    let mut idx = make_index();
    assert!(!idx.contains("d1"));
    idx.add_document(Document::new("d1", "Hello.", DocumentFormat::PlainText))
        .unwrap();
    assert!(idx.contains("d1"));
    idx.remove_document("d1").unwrap();
    assert!(!idx.contains("d1"));
}

#[test]
fn index_get_document() {
    let mut idx = make_index();
    idx.add_document(Document::new(
        "d1",
        "Content here.",
        DocumentFormat::PlainText,
    ))
    .unwrap();
    assert!(idx.get_document("d1").is_some());
    assert!(idx.get_document("nope").is_none());
}

#[test]
fn index_with_config() {
    let idx = DocumentIndex::with_config(
        Box::new(MockProvider::new(32)),
        HybridWeights {
            vector_weight: 0.5,
            keyword_weight: 0.5,
        },
        Bm25Params { k1: 2.0, b: 0.8 },
        Some(ChunkStrategy::Fixed {
            size: 100,
            overlap: 0,
        }),
    );
    assert_eq!(idx.document_count(), 0);
}

#[test]
fn index_error_display() {
    assert!(format!("{}", IndexError::NotFound("d1".into())).contains("document not found: d1"));
    assert!(
        format!("{}", IndexError::DuplicateId("d1".into())).contains("duplicate document id: d1")
    );
}

// ── VectorStore ─────────────────────────────────────────────────────

fn test_config(dims: usize) -> VectorStoreConfig {
    VectorStoreConfig {
        name: "test-store".to_string(),
        dimensions: dims,
        distance_metric: DistanceMetric::Cosine,
        persist_path: None,
    }
}

#[test]
fn store_create() {
    let store = VectorStore::new(test_config(4)).unwrap();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn store_invalid_zero_dims() {
    assert!(VectorStore::new(test_config(0)).is_err());
}

#[test]
fn store_insert_and_query() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    let id1 = store
        .insert("hello", vec![1.0, 0.0, 0.0, 0.0], serde_json::json!({}))
        .unwrap();
    store
        .insert("goodbye", vec![0.0, 1.0, 0.0, 0.0], serde_json::json!({}))
        .unwrap();
    assert_eq!(store.len(), 2);
    let results = store.query(&[1.0, 0.0, 0.0, 0.0], 1).unwrap();
    assert_eq!(results[0].text, "hello");
    assert_eq!(results[0].id, id1);
}

#[test]
fn store_insert_dim_mismatch() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    assert!(store
        .insert("oops", vec![1.0], serde_json::json!({}))
        .is_err());
}

#[test]
fn store_query_dim_mismatch() {
    let store = VectorStore::new(test_config(4)).unwrap();
    assert!(store.query(&[1.0], 1).is_err());
}

#[test]
fn store_delete() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    let id = store
        .insert("del", vec![1.0, 0.0, 0.0, 0.0], serde_json::json!({}))
        .unwrap();
    assert!(store.delete(&id));
    assert!(store.is_empty());
    assert!(!store.delete(&id));
}

#[test]
fn store_query_after_delete() {
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
fn store_metadata_preserved() {
    let mut store = VectorStore::new(test_config(4)).unwrap();
    let meta = serde_json::json!({"key": "value"});
    store
        .insert("meta", vec![1.0, 0.0, 0.0, 0.0], meta.clone())
        .unwrap();
    assert_eq!(
        store.query(&[1.0, 0.0, 0.0, 0.0], 1).unwrap()[0].metadata,
        meta
    );
}

#[test]
fn store_config_accessor() {
    let store = VectorStore::new(test_config(8)).unwrap();
    assert_eq!(store.config().dimensions, 8);
    assert_eq!(store.config().name, "test-store");
}

// ── Chunking strategies ─────────────────────────────────────────────

#[test]
fn chunk_fixed_basic() {
    let chunks = Chunker::chunk(
        "abcdefghij",
        ChunkStrategy::Fixed {
            size: 4,
            overlap: 0,
        },
    );
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].text, "abcd");
    assert_eq!(chunks[1].text, "efgh");
    assert_eq!(chunks[2].text, "ij");
}

#[test]
fn chunk_fixed_with_overlap() {
    let chunks = Chunker::chunk(
        "abcdefghij",
        ChunkStrategy::Fixed {
            size: 4,
            overlap: 2,
        },
    );
    assert_eq!(chunks[0].text, "abcd");
    assert_eq!(chunks[1].text, "cdef");
    assert!(chunks.len() >= 3);
}

#[test]
fn chunk_fixed_empty() {
    assert!(Chunker::chunk(
        "",
        ChunkStrategy::Fixed {
            size: 4,
            overlap: 0
        }
    )
    .is_empty());
}

#[test]
fn chunk_fixed_zero_size() {
    assert!(Chunker::chunk(
        "hello",
        ChunkStrategy::Fixed {
            size: 0,
            overlap: 0
        }
    )
    .is_empty());
}

#[test]
fn chunk_semantic_basic() {
    let chunks = Chunker::chunk("First.\n\nSecond.\n\nThird.", ChunkStrategy::Semantic);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].text, "First.");
    assert_eq!(chunks[2].text, "Third.");
}

#[test]
fn chunk_semantic_empty() {
    assert!(Chunker::chunk("", ChunkStrategy::Semantic).is_empty());
}

#[test]
fn chunk_semantic_single() {
    let chunks = Chunker::chunk("Just one.", ChunkStrategy::Semantic);
    assert_eq!(chunks.len(), 1);
}

#[test]
fn chunk_recursive_small() {
    let chunks = Chunker::chunk("Short.", ChunkStrategy::Recursive { max_size: 100 });
    assert_eq!(chunks.len(), 1);
}

#[test]
fn chunk_recursive_splits_paragraphs() {
    let chunks = Chunker::chunk(
        "First.\n\nSecond.",
        ChunkStrategy::Recursive { max_size: 20 },
    );
    assert!(chunks.len() >= 2);
}

#[test]
fn chunk_recursive_empty() {
    assert!(Chunker::chunk("", ChunkStrategy::Recursive { max_size: 100 }).is_empty());
}

#[test]
fn chunk_recursive_zero_max() {
    assert!(Chunker::chunk("hello", ChunkStrategy::Recursive { max_size: 0 }).is_empty());
}

#[test]
fn chunk_recursive_long_falls_back_to_fixed() {
    let long = "a".repeat(200);
    let chunks = Chunker::chunk(&long, ChunkStrategy::Recursive { max_size: 50 });
    assert!(chunks.len() >= 4);
    for c in &chunks {
        assert!(c.text.len() <= 50);
    }
}

#[test]
fn chunk_offsets() {
    let chunks = Chunker::chunk(
        "Hello world",
        ChunkStrategy::Fixed {
            size: 5,
            overlap: 0,
        },
    );
    assert_eq!(chunks[0].start, 0);
    assert_eq!(chunks[0].end, 5);
}

// ── HNSW ────────────────────────────────────────────────────────────

#[test]
fn hnsw_new_empty() {
    let idx = HnswIndex::new(128, 16, 200);
    assert_eq!(idx.len(), 0);
    assert!(idx.is_empty());
    assert_eq!(idx.dimensions(), 128);
}

#[test]
fn hnsw_insert_and_len() {
    let mut idx = HnswIndex::new(4, 16, 200);
    assert_eq!(idx.insert(vec![1.0, 0.0, 0.0, 0.0]), 0);
    assert_eq!(idx.insert(vec![0.0, 1.0, 0.0, 0.0]), 1);
    assert_eq!(idx.len(), 2);
}

#[test]
fn hnsw_search_exact_match() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.insert(vec![0.0, 1.0, 0.0, 0.0]);
    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 < 1e-5);
}

#[test]
fn hnsw_search_ordering() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.insert(vec![0.9, 0.1, 0.0, 0.0]);
    idx.insert(vec![0.0, 0.0, 0.0, 1.0]);
    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 3);
    assert_eq!(results[0].0, 0);
    assert_eq!(results[2].0, 2);
}

#[test]
fn hnsw_delete() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.insert(vec![0.0, 1.0, 0.0, 0.0]);
    assert!(idx.delete(0));
    assert_eq!(idx.len(), 1);
    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 2);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 1);
}

#[test]
fn hnsw_delete_nonexistent() {
    let mut idx = HnswIndex::new(4, 16, 200);
    assert!(!idx.delete(0));
    assert!(!idx.delete(999));
}

#[test]
fn hnsw_delete_twice() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    assert!(idx.delete(0));
    assert!(!idx.delete(0));
}

#[test]
fn hnsw_search_empty() {
    let idx = HnswIndex::new(4, 16, 200);
    assert!(idx.search(&[1.0, 0.0, 0.0, 0.0], 5).is_empty());
}

#[test]
fn hnsw_get() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(idx.get(0), Some(&[1.0, 2.0, 3.0, 4.0][..]));
    assert_eq!(idx.get(1), None);
    idx.delete(0);
    assert_eq!(idx.get(0), None);
}

// ── Embedding ───────────────────────────────────────────────────────

#[test]
fn simple_embedding_creation() {
    let emb = SimpleEmbedding::new(128).unwrap();
    assert_eq!(emb.dimensions(), 128);
}

#[test]
fn simple_embedding_zero_dims_error() {
    assert!(SimpleEmbedding::new(0).is_err());
}

#[test]
fn simple_embedding_empty_input_error() {
    let emb = SimpleEmbedding::new(64).unwrap();
    assert!(emb.embed("").is_err());
    assert!(emb.embed("   ").is_err());
}

#[test]
fn simple_embedding_correct_dims() {
    let emb = SimpleEmbedding::new(64).unwrap();
    assert_eq!(emb.embed("hello world").unwrap().len(), 64);
}

#[test]
fn simple_embedding_normalized() {
    let emb = SimpleEmbedding::new(128).unwrap();
    let v = emb.embed("the quick brown fox").unwrap();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5);
}

#[test]
fn simple_embedding_deterministic() {
    let emb = SimpleEmbedding::new(64).unwrap();
    assert_eq!(emb.embed("hello").unwrap(), emb.embed("hello").unwrap());
}

#[test]
fn simple_embedding_different_texts() {
    let emb = SimpleEmbedding::new(128).unwrap();
    assert_ne!(emb.embed("hello").unwrap(), emb.embed("goodbye").unwrap());
}

// ── Document ────────────────────────────────────────────────────────

#[test]
fn document_creation() {
    let doc = Document::new("d1", "Hello world", DocumentFormat::PlainText);
    assert_eq!(doc.id, "d1");
    assert_eq!(doc.content, "Hello world");
    assert_eq!(doc.format, DocumentFormat::PlainText);
    assert!(doc.metadata.is_empty());
}

#[test]
fn document_with_metadata() {
    let mut meta = HashMap::new();
    meta.insert("author".to_string(), "test".to_string());
    let doc = Document::with_metadata("d1", "content", DocumentFormat::Markdown, meta);
    assert_eq!(doc.metadata.get("author").unwrap(), "test");
}

#[test]
fn document_chunk_plaintext() {
    let doc = Document::new(
        "d1",
        "Hello world. This is a test.",
        DocumentFormat::PlainText,
    );
    let chunks = doc.chunk(None);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].metadata.get("doc_id").unwrap(), "d1");
}

#[test]
fn document_chunk_markdown() {
    let doc = Document::new(
        "md1",
        "# Heading\n\nFirst paragraph.\n\nSecond paragraph.",
        DocumentFormat::Markdown,
    );
    assert!(doc.chunk(None).len() >= 2);
}

#[test]
fn document_chunk_code() {
    let doc = Document::new(
        "c1",
        "fn main() {\n    println!(\"hello\");\n}",
        DocumentFormat::Code,
    );
    let chunks = doc.chunk(None);
    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].metadata.get("format").unwrap(), "Code");
}

#[test]
fn document_chunk_custom_strategy() {
    let doc = Document::new(
        "d1",
        "abcdefghijklmnopqrstuvwxyz",
        DocumentFormat::PlainText,
    );
    let chunks = doc.chunk(Some(ChunkStrategy::Fixed {
        size: 10,
        overlap: 0,
    }));
    assert_eq!(chunks.len(), 3);
}

#[test]
fn document_chunk_empty() {
    let doc = Document::new("empty", "", DocumentFormat::PlainText);
    assert!(doc.chunk(None).is_empty());
}

// ── MockProvider ────────────────────────────────────────────────────

#[test]
fn mock_provider_dimensions() {
    let p = MockProvider::new(64);
    assert_eq!(p.dimensions(), 64);
}

#[test]
fn mock_provider_embed() {
    let p = MockProvider::new(32);
    assert_eq!(p.embed("hello world").unwrap().len(), 32);
}

#[test]
fn mock_provider_deterministic() {
    let p = MockProvider::new(32);
    assert_eq!(p.embed("test").unwrap(), p.embed("test").unwrap());
}

#[test]
fn mock_provider_empty_input_error() {
    let p = MockProvider::new(32);
    assert!(p.embed("").is_err());
    assert!(p.embed("   ").is_err());
}

#[test]
fn mock_provider_normalized() {
    let p = MockProvider::new(64);
    let v = p.embed("normalize me").unwrap();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-4);
}
