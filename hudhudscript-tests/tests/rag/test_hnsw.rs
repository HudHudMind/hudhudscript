//! Tests extracted from hudhudscript-rag/src/hnsw.rs

use hudhudscript_rag::hnsw::{
    cosine_distance, dot_product_distance, euclidean_distance, HnswIndex,
};

fn random_vector(dims: usize, seed: u64) -> Vec<f32> {
    let mut v = Vec::with_capacity(dims);
    let mut state = seed;
    for _ in 0..dims {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        v.push(((state >> 33) as f32) / (u32::MAX as f32) - 0.5);
    }
    // normalize
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

#[test]
fn test_new_index() {
    let idx = HnswIndex::new(128, 16, 200);
    assert_eq!(idx.len(), 0);
    assert!(idx.is_empty());
    assert_eq!(idx.dimensions(), 128);
}

#[test]
fn test_insert_and_len() {
    let mut idx = HnswIndex::new(4, 16, 200);
    let i0 = idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    assert_eq!(i0, 0);
    assert_eq!(idx.len(), 1);

    let i1 = idx.insert(vec![0.0, 1.0, 0.0, 0.0]);
    assert_eq!(i1, 1);
    assert_eq!(idx.len(), 2);
}

#[test]
fn test_search_exact_match() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.insert(vec![0.0, 1.0, 0.0, 0.0]);
    idx.insert(vec![0.0, 0.0, 1.0, 0.0]);

    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(
        results[0].1 < 1e-5,
        "distance should be ~0, got {}",
        results[0].1
    );
}

#[test]
fn test_search_ordering() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.insert(vec![0.9, 0.1, 0.0, 0.0]); // closer to query
    idx.insert(vec![0.0, 0.0, 0.0, 1.0]); // far from query

    // normalize vec 1
    let norm = (0.9f32 * 0.9 + 0.1 * 0.1).sqrt();
    let _ = norm; // normalization happens in distance fn

    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 3);
    assert_eq!(results.len(), 3);
    // First result should be index 0 (exact match)
    assert_eq!(results[0].0, 0);
    // Second should be index 1 (close)
    assert_eq!(results[1].0, 1);
    // Third should be index 2 (far)
    assert_eq!(results[2].0, 2);
}

#[test]
fn test_delete() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.insert(vec![0.0, 1.0, 0.0, 0.0]);

    assert_eq!(idx.len(), 2);
    assert!(idx.delete(0));
    assert_eq!(idx.len(), 1);

    // Deleted node should not appear in search
    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 2);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 1);
}

#[test]
fn test_delete_nonexistent() {
    let mut idx = HnswIndex::new(4, 16, 200);
    assert!(!idx.delete(0));
    assert!(!idx.delete(999));
}

#[test]
fn test_delete_twice() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    assert!(idx.delete(0));
    assert!(!idx.delete(0)); // already deleted
}

#[test]
fn test_search_empty_index() {
    let idx = HnswIndex::new(4, 16, 200);
    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 5);
    assert!(results.is_empty());
}

#[test]
fn test_get() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 2.0, 3.0, 4.0]);
    assert_eq!(idx.get(0), Some(&[1.0, 2.0, 3.0, 4.0][..]));
    assert_eq!(idx.get(1), None);

    idx.delete(0);
    assert_eq!(idx.get(0), None);
}

#[test]
fn test_cosine_distance_identical() {
    let v = vec![1.0, 0.0, 0.0];
    assert!(cosine_distance(&v, &v) < 1e-5);
}

#[test]
fn test_cosine_distance_orthogonal() {
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    let d = cosine_distance(&a, &b);
    assert!((d - 1.0).abs() < 1e-5, "expected ~1.0, got {}", d);
}

#[test]
fn test_euclidean_distance() {
    let a = vec![0.0, 0.0];
    let b = vec![3.0, 4.0];
    let d = euclidean_distance(&a, &b);
    assert!((d - 5.0).abs() < 1e-5);
}

#[test]
fn test_dot_product_distance() {
    let a = vec![1.0, 0.0];
    let b = vec![1.0, 0.0];
    assert!((dot_product_distance(&a, &b) - (-1.0)).abs() < 1e-5);
}

#[test]
fn test_many_inserts_and_search() {
    let dims = 32;
    let mut idx = HnswIndex::new(dims, 16, 200);
    for i in 0..100 {
        idx.insert(random_vector(dims, i));
    }
    assert_eq!(idx.len(), 100);

    let query = random_vector(dims, 42);
    let results = idx.search(&query, 5);
    assert_eq!(results.len(), 5);

    // Results should be sorted by distance
    for window in results.windows(2) {
        assert!(window[0].1 <= window[1].1);
    }
}

#[test]
fn test_cosine_distance_zero_vector() {
    let a = vec![0.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0];
    let d = cosine_distance(&a, &b);
    assert!(
        (d - 1.0).abs() < 1e-5,
        "expected 1.0 for zero vector, got {}",
        d
    );
}

#[test]
fn test_cosine_distance_opposite() {
    let a = vec![1.0, 0.0];
    let b = vec![-1.0, 0.0];
    let d = cosine_distance(&a, &b);
    assert!(
        (d - 2.0).abs() < 1e-5,
        "expected ~2.0 for opposite vectors, got {}",
        d
    );
}

#[test]
fn test_euclidean_distance_same_point() {
    let a = vec![1.0, 2.0, 3.0];
    let d = euclidean_distance(&a, &a);
    assert!(d.abs() < 1e-5);
}

#[test]
fn test_dot_product_distance_orthogonal() {
    let a = vec![1.0, 0.0];
    let b = vec![0.0, 1.0];
    let d = dot_product_distance(&a, &b);
    assert!(
        d.abs() < 1e-5,
        "expected 0.0 for orthogonal vectors, got {}",
        d
    );
}

#[test]
fn test_delete_all_nodes_and_search() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.insert(vec![0.0, 1.0, 0.0, 0.0]);
    idx.delete(0);
    idx.delete(1);
    assert_eq!(idx.len(), 0);
    assert!(idx.is_empty());
    let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 5);
    assert_eq!(results.len(), 0);
}

#[test]
#[should_panic(expected = "dimension mismatch")]
fn test_insert_wrong_dimensions() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0]); // wrong dimensions
}

#[test]
#[should_panic(expected = "dimension mismatch")]
fn test_search_wrong_dimensions() {
    let mut idx = HnswIndex::new(4, 16, 200);
    idx.insert(vec![1.0, 0.0, 0.0, 0.0]);
    idx.search(&[1.0, 0.0], 1); // wrong dimensions
}
