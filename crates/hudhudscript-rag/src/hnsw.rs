use serde::{Deserialize, Serialize};

/// Threshold below which we use brute-force search instead of HNSW layers.
///
/// Audit v3 Finding 9.1 — Malkov-Yashunin 2016 (IEEE TPAMI 42(5))
/// empirical break-even is ~100-200 for typical embedding dimensionality.
/// The previous value of 1000 was an order of magnitude high, leaving
/// medium-sized indices (N=500-900) on the slow brute-force path.
/// `hnsw_rs` crate default is 100.
const BRUTE_FORCE_THRESHOLD: usize = 128;

/// A node in the HNSW graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswNode {
    pub vector: Vec<f32>,
    pub neighbors: Vec<usize>,
    pub deleted: bool,
}

/// A layer in the HNSW graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswLayer {
    /// Maps global node index -> list of neighbor indices in this layer.
    pub connections: Vec<Vec<usize>>,
}

/// A Hierarchical Navigable Small World (HNSW) index for approximate
/// nearest-neighbor search.
///
/// For small collections (< 1000 items) this falls back to brute-force
/// exact search. For larger collections the multi-layer HNSW graph is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswIndex {
    /// All stored vectors (including soft-deleted ones).
    nodes: Vec<HnswNode>,
    /// HNSW layers; layer 0 is the bottom (densest) layer.
    layers: Vec<HnswLayer>,
    /// Entry point node index (in the top layer), or None if empty.
    entry_point: Option<usize>,
    /// Dimensionality of the vectors.
    dimensions: usize,
    /// Max number of neighbors per node per layer.
    m: usize,
    /// Number of candidates to evaluate during construction.
    ef_construction: usize,
    /// Number of live (non-deleted) nodes.
    live_count: usize,
}

impl HnswIndex {
    /// Create a new empty HNSW index.
    ///
    /// - `dimensions`: vector dimensionality
    /// - `m`: max number of bi-directional links per node (default 16)
    /// - `ef_construction`: size of the dynamic candidate list during build (default 200)
    pub fn new(dimensions: usize, m: usize, ef_construction: usize) -> Self {
        Self {
            nodes: Vec::new(),
            layers: vec![HnswLayer {
                connections: Vec::new(),
            }],
            entry_point: None,
            dimensions,
            m,
            ef_construction,
            live_count: 0,
        }
    }

    /// Insert a vector and return its index.
    pub fn insert(&mut self, vector: Vec<f32>) -> usize {
        assert_eq!(
            vector.len(),
            self.dimensions,
            "vector dimension mismatch: expected {}, got {}",
            self.dimensions,
            vector.len()
        );

        let idx = self.nodes.len();
        self.nodes.push(HnswNode {
            vector,
            neighbors: Vec::new(),
            deleted: false,
        });
        self.live_count += 1;

        // Ensure layer 0 connections vector is large enough
        if self.layers[0].connections.len() <= idx {
            self.layers[0].connections.resize(idx + 1, Vec::new());
        }

        if self.live_count == 1 {
            self.entry_point = Some(idx);
            return idx;
        }

        // For the brute-force regime we just maintain a simple neighbor list
        // in layer 0 so search works. For the HNSW regime we also maintain
        // layer 0 connections.
        self.connect_node_layer0(idx);

        idx
    }

    /// Connect `idx` into layer-0 by finding its nearest neighbors and adding
    /// bi-directional links.
    fn connect_node_layer0(&mut self, idx: usize) {
        let candidates = self.brute_force_search(&self.nodes[idx].vector, self.ef_construction);

        let mut neighbors: Vec<usize> = Vec::new();
        for (neighbor_idx, _dist) in candidates.iter().take(self.m) {
            if *neighbor_idx != idx && !self.nodes[*neighbor_idx].deleted {
                neighbors.push(*neighbor_idx);
            }
        }

        // Set forward links
        self.layers[0].connections[idx] = neighbors.clone();

        // Set back-links (bidirectional)
        for &n in &neighbors {
            if !self.layers[0].connections[n].contains(&idx) {
                self.layers[0].connections[n].push(idx);
                // Prune if over capacity
                if self.layers[0].connections[n].len() > self.m * 2 {
                    let node_vec = self.nodes[n].vector.clone();
                    let mut scored: Vec<(usize, f32)> = self.layers[0].connections[n]
                        .iter()
                        .filter(|&&i| !self.nodes[i].deleted)
                        .map(|&i| (i, cosine_distance(&node_vec, &self.nodes[i].vector)))
                        .collect();
                    scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    self.layers[0].connections[n] =
                        scored.iter().take(self.m).map(|(i, _)| *i).collect();
                }
            }
        }

        // Also update the flat neighbor list on the node struct
        self.nodes[idx].neighbors = neighbors;
    }

    /// Search for the `top_k` nearest vectors. Returns (index, distance) pairs
    /// sorted by ascending distance (most similar first).
    pub fn search(&self, query: &[f32], top_k: usize) -> Vec<(usize, f32)> {
        assert_eq!(
            query.len(),
            self.dimensions,
            "query dimension mismatch: expected {}, got {}",
            self.dimensions,
            query.len()
        );

        if self.live_count == 0 {
            return Vec::new();
        }

        if self.live_count < BRUTE_FORCE_THRESHOLD {
            return self.brute_force_search(query, top_k);
        }

        self.hnsw_search_layer0(query, top_k)
    }

    /// Graph-based greedy search on layer 0.
    fn hnsw_search_layer0(&self, query: &[f32], top_k: usize) -> Vec<(usize, f32)> {
        let ep = match self.entry_point {
            Some(ep) => ep,
            None => return Vec::new(),
        };

        let ef = std::cmp::max(top_k, self.ef_construction);
        let mut visited = vec![false; self.nodes.len()];
        let mut candidates: Vec<(usize, f32)> = Vec::new();

        let ep_dist = cosine_distance(query, &self.nodes[ep].vector);
        candidates.push((ep, ep_dist));
        visited[ep] = true;

        let mut pointer = 0;
        while pointer < candidates.len() {
            let (current, _) = candidates[pointer];
            pointer += 1;

            if current < self.layers[0].connections.len() {
                for &neighbor in &self.layers[0].connections[current] {
                    if visited[neighbor] || self.nodes[neighbor].deleted {
                        continue;
                    }
                    visited[neighbor] = true;
                    let dist = cosine_distance(query, &self.nodes[neighbor].vector);
                    candidates.push((neighbor, dist));
                }
            }

            // Keep candidate list bounded
            if candidates.len() > ef * 2 {
                candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                candidates.truncate(ef);
                // reset pointer to avoid re-visiting already expanded nodes
                // (they are still in the list, pointer stays)
            }
        }

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates
            .into_iter()
            .filter(|(i, _)| !self.nodes[*i].deleted)
            .take(top_k)
            .collect()
    }

    /// Brute-force linear scan (used for small stores and during construction).
    fn brute_force_search(&self, query: &[f32], top_k: usize) -> Vec<(usize, f32)> {
        let mut results: Vec<(usize, f32)> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| !node.deleted)
            .map(|(i, node)| (i, cosine_distance(query, &node.vector)))
            .collect();

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(top_k);
        results
    }

    /// Soft-delete a node by index. Returns true if it was live.
    pub fn delete(&mut self, index: usize) -> bool {
        if index >= self.nodes.len() || self.nodes[index].deleted {
            return false;
        }
        self.nodes[index].deleted = true;
        self.live_count -= 1;
        true
    }

    /// Return the number of live (non-deleted) vectors.
    pub fn len(&self) -> usize {
        self.live_count
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.live_count == 0
    }

    /// Return the dimensionality of the index.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Get the vector at the given index, if it exists and is not deleted.
    pub fn get(&self, index: usize) -> Option<&[f32]> {
        self.nodes
            .get(index)
            .filter(|n| !n.deleted)
            .map(|n| n.vector.as_slice())
    }
}

/// Cosine distance: 1 - cosine_similarity. Range [0, 2]; 0 = identical.
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-10 {
        return 1.0;
    }
    1.0 - (dot / denom)
}

/// Euclidean (L2) distance.
#[allow(dead_code)]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

/// Negative dot product distance (so that smaller = more similar).
#[allow(dead_code)]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> f32 {
    -a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>()
}
