//! Tests extracted from hudhudscript-rag/src/embedding.rs

use hudhudscript_rag::embedding::SimpleEmbedding;
use hudhudscript_rag::EmbeddingProvider;

#[test]
fn test_simple_embedding_creation() {
    let emb = SimpleEmbedding::new(128).unwrap();
    assert_eq!(emb.dimensions(), 128);
}

#[test]
fn test_zero_dimensions_error() {
    let result = SimpleEmbedding::new(0);
    assert!(result.is_err());
}

#[test]
fn test_empty_input_error() {
    let emb = SimpleEmbedding::new(64).unwrap();
    assert!(emb.embed("").is_err());
    assert!(emb.embed("   ").is_err());
    assert!(emb.embed("!!!").is_err());
}

#[test]
fn test_embed_produces_correct_dimensions() {
    let emb = SimpleEmbedding::new(64).unwrap();
    let vec = emb.embed("hello world").unwrap();
    assert_eq!(vec.len(), 64);
}

#[test]
fn test_embed_is_normalized() {
    let emb = SimpleEmbedding::new(128).unwrap();
    let vec = emb
        .embed("the quick brown fox jumps over the lazy dog")
        .unwrap();
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-5, "norm was {}", norm);
}

#[test]
fn test_same_text_same_embedding() {
    let emb = SimpleEmbedding::new(64).unwrap();
    let v1 = emb.embed("hello world").unwrap();
    let v2 = emb.embed("hello world").unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn test_different_text_different_embedding() {
    let emb = SimpleEmbedding::new(128).unwrap();
    let v1 = emb.embed("hello world").unwrap();
    let v2 = emb.embed("goodbye universe").unwrap();
    assert_ne!(v1, v2);
}

#[test]
fn test_similar_texts_higher_similarity() {
    let emb = SimpleEmbedding::new(128).unwrap();
    let v_base = emb.embed("the cat sat on the mat").unwrap();
    let v_similar = emb.embed("the cat sat on the rug").unwrap();
    let v_different = emb.embed("quantum computing algorithms").unwrap();

    let sim_similar: f32 = v_base.iter().zip(&v_similar).map(|(a, b)| a * b).sum();
    let sim_different: f32 = v_base.iter().zip(&v_different).map(|(a, b)| a * b).sum();

    assert!(
        sim_similar > sim_different,
        "similar={}, different={}",
        sim_similar,
        sim_different
    );
}

#[test]
fn test_tokenize() {
    let tokens = SimpleEmbedding::tokenize("Hello, World! 123");
    assert_eq!(tokens, vec!["hello", "world", "123"]);
}
