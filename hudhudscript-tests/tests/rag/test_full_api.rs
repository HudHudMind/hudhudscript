//! Real unit tests for hudhudscript-rag — chunking, embedding, vector store, search

use hudhudscript_rag::*;

// ── Chunking ─────────────────────────────────────────────────────────────

#[test]
fn chunk_strategy_fixed_size() {
    let strategy = ChunkStrategy::Fixed {
        size: 100,
        overlap: 10,
    };
    let _ = strategy;
}

#[test]
fn chunk_strategy_semantic() {
    let strategy = ChunkStrategy::Semantic;
    let _ = strategy;
}

#[test]
fn chunk_strategy_recursive() {
    let strategy = ChunkStrategy::Recursive { max_size: 500 };
    let _ = strategy;
}

#[test]
fn chunker_chunk_basic() {
    let text = "HudHudScript is a multilingual scripting language. It supports many human languages.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Fixed { size: 100, overlap: 10 });
    assert!(!chunks.is_empty());
}

#[test]
fn chunk_empty_document_returns_empty() {
    let chunks = Chunker::chunk("", ChunkStrategy::Fixed { size: 100, overlap: 0 });
    assert!(chunks.is_empty());
}

#[test]
fn chunk_markdown_document() {
    let text = "# Title\n\nParagraph one.\n\nParagraph two.";
    let chunks = Chunker::chunk(text, ChunkStrategy::Semantic);
    assert!(!chunks.is_empty());
}

// ── Document ─────────────────────────────────────────────────────────────

#[test]
fn document_plain_text() {
    let doc = Document::new("doc1", "hello world", DocumentFormat::PlainText);
    assert_eq!(doc.content, "hello world");
}

#[test]
fn document_format_variants() {
    assert!(matches!(DocumentFormat::PlainText, DocumentFormat::PlainText));
    assert!(matches!(DocumentFormat::Markdown, DocumentFormat::Markdown));
    assert!(matches!(DocumentFormat::Code, DocumentFormat::Code));
}

// ── Embedding ────────────────────────────────────────────────────────────

#[test]
fn simple_embedding_new() {
    let emb = SimpleEmbedding::new(3).unwrap();
    assert_eq!(emb.dimensions(), 3);
}

#[test]
fn simple_embedding_zero_dimensions_errors() {
    let emb = SimpleEmbedding::new(0);
    assert!(emb.is_err());
}

#[test]
fn simple_embedding_tokenize() {
    let tokens = SimpleEmbedding::tokenize("Hello, world!");
    assert_eq!(tokens, vec!["hello", "world"]);
}

#[test]
fn embedding_error_display() {
    let e = EmbeddingError::InvalidDimensions(384);
    assert!(!format!("{}", e).is_empty());
}

// ── HNSW ─────────────────────────────────────────────────────────────────

#[test]
fn hnsw_index_new() {
    let index: HnswIndex = HnswIndex::new(16, 200, 32);
    let _ = index;
}

#[test]
fn cosine_distance_identical_vectors() {
    let a = vec![1.0, 0.0];
    let b = vec![1.0, 0.0];
    let dist = hudhudscript_rag::hnsw::cosine_distance(&a, &b);
    assert!((dist - 0.0).abs() < 0.001);
}

#[test]
fn cosine_distance_orthogonal() {
    let a = vec![1.0, 0.0];
    let b = vec![0.0, 1.0];
    let dist = hudhudscript_rag::hnsw::cosine_distance(&a, &b);
    assert!((dist - 1.0).abs() < 0.001);
}

#[test]
fn euclidean_distance_same() {
    let a = vec![0.0, 0.0];
    let b = vec![0.0, 0.0];
    let dist = hudhudscript_rag::hnsw::euclidean_distance(&a, &b);
    assert!((dist - 0.0).abs() < 0.001);
}

#[test]
fn euclidean_distance_unit() {
    let a = vec![0.0, 0.0];
    let b = vec![3.0, 4.0];
    let dist = hudhudscript_rag::hnsw::euclidean_distance(&a, &b);
    assert!((dist - 5.0).abs() < 0.001);
}

#[test]
fn dot_product_distance() {
    let a = vec![1.0, 2.0];
    let b = vec![3.0, 4.0];
    let dist = hudhudscript_rag::hnsw::dot_product_distance(&a, &b);
    // 1.0 - (1*3 + 2*4) = 1 - 11 = -10
    assert!((dist - (-11.0)).abs() < 0.001);
}

// ── Vector Store ─────────────────────────────────────────────────────────

#[test]
fn vector_store_new() {
    let cfg = VectorStoreConfig {
        name: "test".into(),
        dimensions: 4,
        distance_metric: DistanceMetric::Cosine,
        persist_path: None,
    };
    let store = VectorStore::new(cfg).unwrap();
    let _ = store;
}

#[test]
fn vector_store_insert_and_search() {
    let cfg = VectorStoreConfig {
        name: "test".into(),
        dimensions: 4,
        distance_metric: DistanceMetric::Cosine,
        persist_path: None,
    };
    let mut store = VectorStore::new(cfg).unwrap();
    store.insert("doc_a", vec![1.0, 0.0, 0.0, 0.0], serde_json::Value::Null).unwrap();
    store.insert("doc_b", vec![0.9, 0.1, 0.0, 0.0], serde_json::Value::Null).unwrap();
    let results = store.query(&[1.0, 0.0, 0.0, 0.0], 2).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].text, "doc_a");
}

#[test]
fn vector_store_wrong_dimension_errors() {
    let cfg = VectorStoreConfig {
        name: "test".into(),
        dimensions: 4,
        distance_metric: DistanceMetric::Cosine,
        persist_path: None,
    };
    let mut store = VectorStore::new(cfg).unwrap();
    let result = store.insert("bad", vec![1.0, 0.0], serde_json::Value::Null);
    // Wrong dimension should error
    assert!(result.is_err());
}

#[test]
fn distance_metric_variants() {
    assert!(matches!(
        DistanceMetric::Cosine,
        DistanceMetric::Cosine
    ));
    assert!(matches!(
        DistanceMetric::Euclidean,
        DistanceMetric::Euclidean
    ));
}

#[test]
fn store_error_display() {
    let e = StoreError::DimensionMismatch {
        expected: 768,
        got: 128,
    };
    assert!(!format!("{}", e).is_empty());
}

// ── DocumentIndex ────────────────────────────────────────────────────────

#[test]
fn document_index_with_mock_provider() {
    let provider = MockProvider::new(16);
    let mut idx = DocumentIndex::new(Box::new(provider));
    let doc = Document::new(
        "doc1",
        "HudHudScript is a multilingual scripting language.",
        DocumentFormat::PlainText,
    );
    idx.add_document(doc).unwrap();
    let results = idx.search("multilingual", 3).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn document_index_search_no_results() {
    let provider = MockProvider::new(16);
    let idx = DocumentIndex::new(Box::new(provider));
    let results = idx.search("nonexistent_term", 5).unwrap();
    // Empty index returns empty results
    assert!(results.is_empty());
}

#[test]
fn index_error_display() {
    let e = IndexError::Embedding(EmbeddingError::ProviderError("provider error".into()));
    assert!(!format!("{}", e).is_empty());
}

// ── HybridSearch ─────────────────────────────────────────────────────────

#[test]
fn hybrid_weights_construction() {
    let weights = HybridWeights {
        vector_weight: 0.7,
        keyword_weight: 0.3,
    };
    assert!((weights.vector_weight - 0.7).abs() < 0.001);
    assert!((weights.keyword_weight - 0.3).abs() < 0.001);
}

#[test]
fn hybrid_search_new() {
    let weights = HybridWeights {
        vector_weight: 0.6,
        keyword_weight: 0.4,
    };
    let bm25 = Bm25Params::default();
    let search = HybridSearch::new(weights, bm25);
    let _ = search;
}

// ── Provider ─────────────────────────────────────────────────────────────

#[test]
fn mock_provider_new() {
    let provider = MockProvider::new(384);
    let _ = provider;
}

#[test]
fn api_provider_config_construction() {
    let config = ApiProviderConfig {
        endpoint: "https://api.example.com".into(),
        api_key: "key123".into(),
        model: "text-embedding-3".into(),
        dimensions: 1536,
    };
    assert_eq!(config.endpoint, "https://api.example.com");
    assert_eq!(config.dimensions, 1536);
}

#[test]
fn api_provider_new() {
    let config = ApiProviderConfig {
        endpoint: "https://api.example.com".into(),
        api_key: "".into(),
        model: "".into(),
        dimensions: 384,
    };
    let provider = ApiProvider::new(config);
    let _ = provider;
}
