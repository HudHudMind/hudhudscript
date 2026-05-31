//! Embedding provider abstraction layer.
//!
//! Defines the `EmbeddingProvider` trait (re-exported from `embedding.rs`) and
//! provides a `MockProvider` for testing and an `ApiProvider` adapter for
//! external API-backed embeddings.

use crate::embedding::{EmbeddingError, EmbeddingProvider};

/// A mock embedding provider that returns deterministic vectors based on
/// a simple hash. Useful for testing without external dependencies.
pub struct MockProvider {
    dimensions: usize,
}

impl MockProvider {
    /// Create a new `MockProvider` with the given output dimensionality.
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl EmbeddingProvider for MockProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::EmptyInput);
        }
        // Deterministic hash-based vector
        let mut vector = vec![0.0f32; self.dimensions];
        let mut hash: u64 = 5381;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        for (i, v) in vector.iter_mut().enumerate() {
            let seed = hash.wrapping_add(i as u64);
            *v = ((seed % 1000) as f32 / 1000.0) - 0.5;
        }
        // Normalize
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-10 {
            for v in vector.iter_mut() {
                *v /= norm;
            }
        }
        Ok(vector)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

/// Configuration for an API-based embedding provider.
#[derive(Debug, Clone)]
pub struct ApiProviderConfig {
    /// The API endpoint URL.
    pub endpoint: String,
    /// The API key for authentication.
    pub api_key: String,
    /// The model name to use (e.g., "text-embedding-ada-002").
    pub model: String,
    /// Output dimensionality.
    pub dimensions: usize,
}

/// An API-backed embedding provider. This struct holds configuration for
/// making HTTP calls to an external embedding service.
///
/// Note: actual HTTP calls require an async runtime and an HTTP client.
/// This provides the structural abstraction; callers integrate their own
/// HTTP layer.
pub struct ApiProvider {
    config: ApiProviderConfig,
}

impl ApiProvider {
    /// Create a new `ApiProvider` with the given configuration.
    pub fn new(config: ApiProviderConfig) -> Self {
        Self { config }
    }

    /// Return a reference to the provider configuration.
    pub fn config(&self) -> &ApiProviderConfig {
        &self.config
    }
}

impl EmbeddingProvider for ApiProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::EmptyInput);
        }

        // Real HTTP call to the embedding API (OpenAI-compatible format)
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| EmbeddingError::ProviderError(format!("HTTP client error: {}", e)))?;

        let body = serde_json::json!({
            "input": text,
            "model": self.config.model,
        });

        let mut request = client
            .post(&self.config.endpoint)
            .header("Content-Type", "application/json")
            .json(&body);

        // Add API key if present
        if !self.config.api_key.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", self.config.api_key));
        }

        let response = request.send().map_err(|e| {
            EmbeddingError::ProviderError(format!(
                "HTTP request to '{}' (model: {}) failed: {}",
                self.config.endpoint, self.config.model, e
            ))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(EmbeddingError::ProviderError(format!(
                "Embedding API at '{}' (model: {}) returned {}: {}",
                self.config.endpoint, self.config.model, status, body
            )));
        }

        // Parse OpenAI-compatible response: { data: [{ embedding: [...] }] }
        let json: serde_json::Value = response.json().map_err(|e| {
            EmbeddingError::ProviderError(format!("Failed to parse response: {}", e))
        })?;

        let embedding = json
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("embedding"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| {
                EmbeddingError::ProviderError(
                    "Response missing data[0].embedding array".to_string(),
                )
            })?;

        let floats: Vec<f32> = embedding
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        if floats.len() != self.config.dimensions && self.config.dimensions > 0 {
            return Err(EmbeddingError::ProviderError(format!(
                "Dimension mismatch: expected {}, got {}",
                self.config.dimensions,
                floats.len()
            )));
        }

        Ok(floats)
    }

    fn dimensions(&self) -> usize {
        self.config.dimensions
    }
}
