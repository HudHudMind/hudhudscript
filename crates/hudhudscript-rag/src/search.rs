//! Hybrid search combining vector cosine similarity with BM25 keyword scoring.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single search result with combined score information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    /// Identifier for the document.
    pub doc_id: String,
    /// Combined hybrid score (higher = more relevant).
    pub score: f32,
    /// The chunk text that matched.
    pub chunk: String,
    /// The vector similarity component (cosine similarity, 0..1).
    pub vector_score: f32,
    /// The BM25 keyword component.
    pub bm25_score: f32,
}

/// Configuration for hybrid search weight balancing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HybridWeights {
    /// Weight for the vector similarity score (0.0..1.0).
    pub vector_weight: f32,
    /// Weight for the BM25 keyword score (0.0..1.0).
    pub keyword_weight: f32,
}

impl Default for HybridWeights {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            keyword_weight: 0.3,
        }
    }
}

/// BM25 parameters.
#[derive(Debug, Clone, Copy)]
pub struct Bm25Params {
    /// Term frequency saturation parameter (typically 1.2-2.0).
    pub k1: f32,
    /// Length normalization parameter (typically 0.75).
    pub b: f32,
}

impl Default for Bm25Params {
    fn default() -> Self {
        Self { k1: 1.5, b: 0.75 }
    }
}

/// A stored document entry for BM25 keyword search.
#[derive(Debug, Clone)]
struct BM25Document {
    doc_id: String,
    chunk: String,
    /// Term frequency map: term -> count.
    tf: HashMap<String, usize>,
    /// Total number of terms in this document.
    term_count: usize,
}

/// Hybrid search engine combining vector cosine similarity and BM25 keyword scoring.
pub struct HybridSearch {
    /// Stored documents for BM25 scoring.
    documents: Vec<BM25Document>,
    /// Document frequency: term -> number of documents containing term.
    df: HashMap<String, usize>,
    /// Average document length (in terms).
    avg_dl: f32,
    /// BM25 parameters.
    bm25_params: Bm25Params,
    /// Hybrid score weights.
    weights: HybridWeights,
}

impl HybridSearch {
    /// Create a new hybrid search engine with the given weights and BM25 parameters.
    pub fn new(weights: HybridWeights, bm25_params: Bm25Params) -> Self {
        Self {
            documents: Vec::new(),
            df: HashMap::new(),
            avg_dl: 0.0,
            bm25_params,
            weights,
        }
    }

    /// Create a hybrid search engine with default parameters.
    pub fn with_defaults() -> Self {
        Self::new(HybridWeights::default(), Bm25Params::default())
    }

    /// Add a document chunk to the BM25 index.
    pub fn add_document(&mut self, doc_id: &str, chunk: &str) {
        let terms = Self::tokenize(chunk);
        let mut tf: HashMap<String, usize> = HashMap::new();
        for term in &terms {
            *tf.entry(term.clone()).or_insert(0) += 1;
        }

        // Update document frequency
        for term in tf.keys() {
            *self.df.entry(term.clone()).or_insert(0) += 1;
        }

        let term_count = terms.len();
        self.documents.push(BM25Document {
            doc_id: doc_id.to_string(),
            chunk: chunk.to_string(),
            tf,
            term_count,
        });

        // Recompute average document length
        let total: usize = self.documents.iter().map(|d| d.term_count).sum();
        self.avg_dl = total as f32 / self.documents.len().max(1) as f32;
    }

    /// Remove all document chunks with the given doc_id.
    pub fn remove_document(&mut self, doc_id: &str) {
        // Decrement DF for terms in removed documents
        let to_remove: Vec<usize> = self
            .documents
            .iter()
            .enumerate()
            .filter(|(_, d)| d.doc_id == doc_id)
            .map(|(i, _)| i)
            .collect();

        for &idx in to_remove.iter().rev() {
            let doc = &self.documents[idx];
            for term in doc.tf.keys() {
                if let Some(count) = self.df.get_mut(term) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        self.df.remove(term);
                    }
                }
            }
            self.documents.remove(idx);
        }

        // Recompute average document length
        if self.documents.is_empty() {
            self.avg_dl = 0.0;
        } else {
            let total: usize = self.documents.iter().map(|d| d.term_count).sum();
            self.avg_dl = total as f32 / self.documents.len() as f32;
        }
    }

    /// Compute BM25 scores for a query against all stored documents.
    pub fn bm25_search(&self, query: &str) -> Vec<(usize, f32)> {
        let query_terms = Self::tokenize(query);
        let n = self.documents.len() as f32;

        let mut scores: Vec<(usize, f32)> = Vec::new();

        for (idx, doc) in self.documents.iter().enumerate() {
            let mut score = 0.0f32;
            for term in &query_terms {
                let tf = *doc.tf.get(term).unwrap_or(&0) as f32;
                let df = *self.df.get(term).unwrap_or(&0) as f32;

                if df == 0.0 || tf == 0.0 {
                    continue;
                }

                // IDF component: log((N - df + 0.5) / (df + 0.5) + 1)
                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

                // TF component with length normalization
                let dl = doc.term_count as f32;
                let k1 = self.bm25_params.k1;
                let b = self.bm25_params.b;
                let tf_norm =
                    (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * dl / self.avg_dl.max(1.0)));

                score += idf * tf_norm;
            }
            scores.push((idx, score));
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    /// Perform hybrid search combining precomputed vector scores with BM25.
    ///
    /// `vector_results` should be a list of (document index, cosine similarity score)
    /// where similarity is in [0, 1] (higher = more similar). The index corresponds
    /// to the order documents were added.
    ///
    /// Returns results sorted by combined score (descending).
    pub fn search(
        &self,
        query: &str,
        vector_results: &[(usize, f32)],
        top_k: usize,
    ) -> Vec<HybridSearchResult> {
        let bm25_results = self.bm25_search(query);

        // Normalize BM25 scores to [0, 1]
        let max_bm25 = bm25_results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
        let bm25_map: HashMap<usize, f32> = bm25_results
            .into_iter()
            .map(|(idx, s)| {
                let normalized = if max_bm25 > 0.0 { s / max_bm25 } else { 0.0 };
                (idx, normalized)
            })
            .collect();

        // Normalize vector scores to [0, 1] (they should already be similarities)
        let max_vec = vector_results
            .iter()
            .map(|(_, s)| *s)
            .fold(0.0f32, f32::max);
        let vec_map: HashMap<usize, f32> = vector_results
            .iter()
            .map(|(idx, s)| {
                let normalized = if max_vec > 0.0 { s / max_vec } else { 0.0 };
                (*idx, normalized)
            })
            .collect();

        // Gather all candidate indices
        let mut all_indices: Vec<usize> = Vec::new();
        for &(idx, _) in vector_results {
            if idx < self.documents.len() && !all_indices.contains(&idx) {
                all_indices.push(idx);
            }
        }
        for idx in bm25_map.keys() {
            if *idx < self.documents.len() && !all_indices.contains(idx) {
                all_indices.push(*idx);
            }
        }

        // Combine scores
        let mut results: Vec<HybridSearchResult> = all_indices
            .into_iter()
            .map(|idx| {
                let vec_score = *vec_map.get(&idx).unwrap_or(&0.0);
                let bm25_score = *bm25_map.get(&idx).unwrap_or(&0.0);
                let combined = self.weights.vector_weight * vec_score
                    + self.weights.keyword_weight * bm25_score;
                let doc = &self.documents[idx];
                HybridSearchResult {
                    doc_id: doc.doc_id.clone(),
                    score: combined,
                    chunk: doc.chunk.clone(),
                    vector_score: vec_score,
                    bm25_score,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(top_k);
        results
    }

    /// Simple word tokenizer: lowercase, strip punctuation.
    fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    /// Return the number of indexed document chunks.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if the search index is empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Return the configured hybrid weights.
    pub fn weights(&self) -> HybridWeights {
        self.weights
    }
}
