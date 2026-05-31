//! Public API tests for tokenomics::embedding_cache

use hudhudscript_tokenomics::embedding_cache::*;

#[test]
fn test_new_cache() {
    let cache = EmbeddingCache::new("v1".to_string(), None, false, 100);
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_put_and_get() {
    let mut cache = EmbeddingCache::new("v1".to_string(), None, false, 100);
    let hash = text_hash("hello world");
    assert!(cache.get(hash).is_none());
    assert_eq!(cache.stats().misses, 1);

    cache.put(hash, vec![0.1, 0.2, 0.3]);
    let entry = cache.get(hash);
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().dimensions, 3);
    assert_eq!(cache.stats().hits, 1);
}

#[test]
fn test_version_mismatch_is_miss() {
    let mut cache = EmbeddingCache::new("v1".to_string(), None, false, 100);
    let hash = text_hash("test");
    cache.put(hash, vec![1.0, 0.0]);
    // Cannot change model_version directly from outside (private field),
    // so we test via invalidation instead
    cache.invalidate_version("v1");
    assert!(cache.get(hash).is_none());
}

#[test]
fn test_put_with_reduction() {
    let mut cache = EmbeddingCache::new("v1".to_string(), Some(3), false, 100);
    let hash = text_hash("reduce me");
    cache.put(hash, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    let entry = cache.get(hash).unwrap();
    assert_eq!(entry.dimensions, 3);
}

#[test]
fn test_put_with_quantization() {
    let mut cache = EmbeddingCache::new("v1".to_string(), None, true, 100);
    let hash = text_hash("quantize me");
    cache.put(hash, vec![0.5, -0.3, 1.0]);
    let entry = cache.get(hash).unwrap();
    assert_eq!(entry.dimensions, 3);
    // Quantized then dequantized: 1.0 -> 127/127 = 1.0
    assert_eq!(entry.embedding[2], 1.0);
}

#[test]
fn test_max_entries_eviction() {
    let mut cache = EmbeddingCache::new("v1".to_string(), None, false, 2);
    cache.put(text_hash("a"), vec![1.0]);
    cache.put(text_hash("b"), vec![2.0]);
    cache.put(text_hash("c"), vec![3.0]);
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_invalidate_version() {
    let mut cache = EmbeddingCache::new("v1".to_string(), None, false, 100);
    cache.put(text_hash("a"), vec![1.0]);
    cache.put(text_hash("b"), vec![2.0]);
    assert_eq!(cache.len(), 2);
    cache.invalidate_version("v1");
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.stats().invalidations, 2);
}

#[test]
fn test_storage_bytes_saved() {
    let mut cache = EmbeddingCache::new("v1".to_string(), Some(2), false, 100);
    cache.put(text_hash("x"), vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    // Original: 5 * 4 = 20, stored: 2 * 4 = 8, saved: 12
    assert_eq!(cache.stats().storage_bytes_saved, 12);
}

#[test]
fn test_is_empty() {
    let mut cache = EmbeddingCache::new("v1".to_string(), None, false, 100);
    assert!(cache.is_empty());
    cache.put(text_hash("something"), vec![1.0]);
    assert!(!cache.is_empty());
}

#[test]
fn test_text_hash_deterministic() {
    let h1 = text_hash("hello");
    let h2 = text_hash("hello");
    assert_eq!(h1, h2);
    let h3 = text_hash("world");
    assert_ne!(h1, h3);
}

#[test]
fn test_reduce_dimensions_basic() {
    let original = vec![0.5, 0.3, 0.1, 0.8, 0.2, 0.4, 0.6, 0.7];
    let reduced = reduce_dimensions(&original, 4);
    assert_eq!(reduced.len(), 4);
}

#[test]
fn test_reduce_dimensions_unit_norm() {
    let v = vec![3.0, 4.0, 0.0, 0.0, 0.0];
    let reduced = reduce_dimensions(&v, 2);
    let norm: f64 = reduced
        .iter()
        .map(|x| (*x as f64).powi(2))
        .sum::<f64>()
        .sqrt();
    assert!((norm - 1.0).abs() < 1e-5);
}

#[test]
fn test_reduce_dimensions_zero_vector() {
    let zeros = vec![0.0_f32; 10];
    let reduced = reduce_dimensions(&zeros, 5);
    assert_eq!(reduced.len(), 5);
    assert!(reduced.iter().all(|v| *v == 0.0));
}

#[test]
fn test_quantize_int8_roundtrip() {
    let original = vec![0.5, -0.3, 1.0, -1.0, 0.0];
    let quantized = quantize_int8(&original);
    let recovered = dequantize_int8(&quantized);
    for (o, r) in original.iter().zip(recovered.iter()) {
        assert!((o - r).abs() < 0.02);
    }
}

#[test]
fn test_quantize_int8_empty() {
    let result = quantize_int8(&[]);
    assert!(result.is_empty());
}

#[test]
fn test_quantize_int8_zero_vector() {
    let zeros = vec![0.0; 5];
    let q = quantize_int8(&zeros);
    assert!(q.iter().all(|v| *v == 0));
    let d = dequantize_int8(&q);
    assert!(d.iter().all(|v| *v == 0.0));
}

#[test]
fn test_dequantize_int8() {
    let quantized = vec![127i8, -127, 0, 64];
    let result = dequantize_int8(&quantized);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[1], -1.0);
    assert_eq!(result[2], 0.0);
    assert!((result[3] - 0.5039).abs() < 0.01);
}

#[test]
fn test_reduction_preserves_direction() {
    let original = vec![0.5, 0.3, 0.1, 0.8, 0.2, 0.4, 0.6, 0.7];
    let reduced = reduce_dimensions(&original, 4);
    assert_eq!(reduced.len(), 4);

    // The reduced vector should point in a similar direction as the first
    // 4 components of the original (just re-normalized).
    let original_prefix: Vec<f32> = original[..4].to_vec();
    let sim = cosine_similarity(&original_prefix, &reduced);
    assert!(
        sim > 0.95,
        "Cosine similarity after reduction should be > 0.95, got {}",
        sim
    );
}

#[test]
fn test_cosine_similarity_mismatched_lengths() {
    let a = vec![1.0_f32, 2.0];
    let b = vec![1.0_f32, 2.0, 3.0];
    let sim = cosine_similarity(&a, &b);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_cosine_similarity_empty() {
    let sim = cosine_similarity(&[], &[]);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_cosine_similarity_zero_denom() {
    let a = vec![0.0_f32, 0.0, 0.0];
    let b = vec![1.0_f32, 2.0, 3.0];
    let sim = cosine_similarity(&a, &b);
    assert_eq!(sim, 0.0);
}
