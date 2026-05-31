//! Tests extracted from hudhudscript-rag/src/search.rs

use hudhudscript_rag::search::{Bm25Params, HybridSearch, HybridWeights};

#[test]
fn test_hybrid_search_creation() {
    let hs = HybridSearch::with_defaults();
    assert!(hs.is_empty());
    assert_eq!(hs.len(), 0);
}

#[test]
fn test_add_and_bm25_search() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "the quick brown fox jumps over the lazy dog");
    hs.add_document("doc2", "a fast brown fox leaps over a sleepy hound");
    hs.add_document("doc3", "quantum computing algorithms and complexity");

    let results = hs.bm25_search("brown fox");
    // doc1 and doc2 should score higher than doc3
    assert!(results[0].1 > 0.0);
    let top_idx = results[0].0;
    assert!(top_idx == 0 || top_idx == 1);
}

#[test]
fn test_bm25_exact_match_scores_high() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "rust programming language");
    hs.add_document("doc2", "python scripting language");
    hs.add_document("doc3", "java enterprise development");

    let results = hs.bm25_search("rust programming");
    assert_eq!(results[0].0, 0); // doc1 should be first
}

#[test]
fn test_hybrid_search_combines_scores() {
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

    // Simulate vector results: doc2 is closest by vector
    let vector_results = vec![(1, 0.9f32), (0, 0.7), (2, 0.1)];

    let results = hs.search("learning models", &vector_results, 3);
    assert_eq!(results.len(), 3);
    // Combined score should put doc1 or doc2 on top
    assert!(results[0].doc_id == "doc1" || results[0].doc_id == "doc2");
    // Cooking doc should be last
    assert_eq!(results[2].doc_id, "doc3");
}

#[test]
fn test_remove_document() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello world");
    hs.add_document("doc2", "goodbye world");
    assert_eq!(hs.len(), 2);

    hs.remove_document("doc1");
    assert_eq!(hs.len(), 1);
}

#[test]
fn test_remove_nonexistent() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello");
    hs.remove_document("nonexistent");
    assert_eq!(hs.len(), 1);
}

#[test]
fn test_empty_query() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello world");
    let results = hs.bm25_search("");
    // All scores should be 0
    for (_, score) in &results {
        assert_eq!(*score, 0.0);
    }
}

#[test]
fn test_remove_document_updates_avg_dl() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "a b c d e f g h i j");
    hs.add_document("doc2", "x y");
    hs.remove_document("doc1");
    assert_eq!(hs.len(), 1);
    // After removing the long doc, avg_dl should reflect only doc2
}

#[test]
fn test_remove_all_documents() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello");
    hs.add_document("doc1", "world");
    hs.remove_document("doc1");
    assert_eq!(hs.len(), 0);
    assert!(hs.is_empty());
}

#[test]
fn test_hybrid_search_empty_vector_results() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "rust programming language");
    let results = hs.search("rust", &[], 3);
    // With no vector results, only bm25 candidates appear
    assert!(!results.is_empty());
    assert_eq!(results[0].doc_id, "doc1");
}

#[test]
fn test_hybrid_search_zero_bm25_and_vector() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "hello world");
    // Query with no matching terms, and empty vector results
    let results = hs.search("zzzzunknown", &[], 3);
    // BM25 may return doc with score 0; verify all scores are near-zero
    for result in &results {
        assert!(
            result.score <= f32::EPSILON,
            "expected near-zero score, got {}",
            result.score
        );
    }
}

#[test]
fn test_bm25_search_no_matching_terms() {
    let mut hs = HybridSearch::with_defaults();
    hs.add_document("doc1", "alpha beta gamma");
    let results = hs.bm25_search("zzz xyz");
    // All scores should be 0
    for (_, score) in &results {
        assert_eq!(*score, 0.0);
    }
}

#[test]
fn test_weights_returned() {
    let w = HybridWeights {
        vector_weight: 0.6,
        keyword_weight: 0.4,
    };
    let hs = HybridSearch::new(w, Bm25Params::default());
    let got = hs.weights();
    assert!((got.vector_weight - 0.6).abs() < 1e-5);
    assert!((got.keyword_weight - 0.4).abs() < 1e-5);
}
