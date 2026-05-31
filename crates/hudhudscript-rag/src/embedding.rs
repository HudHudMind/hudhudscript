use std::collections::HashMap;

/// Errors that can occur during embedding generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmbeddingError {
    EmptyInput,
    InvalidDimensions(usize),
    ProviderError(String),
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            EmbeddingError::EmptyInput => write!(f, "empty input text"),
            EmbeddingError::InvalidDimensions(d) => write!(f, "invalid dimensions: {}", d),
            EmbeddingError::ProviderError(s) => write!(f, "provider error: {}", s),
        }
    }
}

impl std::error::Error for EmbeddingError {}

/// Trait for embedding providers that convert text into vector representations.
pub trait EmbeddingProvider: Send + Sync {
    /// Embed a text string into a fixed-size vector.
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Return the dimensionality of the output vectors.
    fn dimensions(&self) -> usize;
}

/// A simple bag-of-words / hash-based embedding provider.
///
/// This provides a rough but functional embedding without any external API
/// calls. Words are tokenized, hashed to dimension buckets, counted (TF),
/// and the resulting vector is L2-normalized.
pub struct SimpleEmbedding {
    dimensions: usize,
}

impl SimpleEmbedding {
    /// Create a new `SimpleEmbedding` with the given output dimensionality.
    pub fn new(dimensions: usize) -> Result<Self, EmbeddingError> {
        if dimensions == 0 {
            return Err(EmbeddingError::InvalidDimensions(0));
        }
        Ok(Self { dimensions })
    }

    /// Tokenize text into lowercased words, stripping punctuation.
    pub fn tokenize(text: &str) -> Vec<String> {
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

    /// Simple hash function that maps a word to a dimension bucket.
    fn hash_word(word: &str, dimensions: usize) -> usize {
        let mut hash: u64 = 5381;
        for byte in word.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        (hash % dimensions as u64) as usize
    }

    /// L2-normalize a vector in place.
    fn normalize(vec: &mut [f32]) {
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-10 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }
    }
}

impl EmbeddingProvider for SimpleEmbedding {
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::EmptyInput);
        }

        let tokens = Self::tokenize(text);
        if tokens.is_empty() {
            return Err(EmbeddingError::EmptyInput);
        }

        // Count term frequencies per bucket
        let mut tf: HashMap<usize, f32> = HashMap::new();
        for token in &tokens {
            let bucket = Self::hash_word(token, self.dimensions);
            *tf.entry(bucket).or_insert(0.0) += 1.0;
        }

        // Build the vector
        let mut vector = vec![0.0f32; self.dimensions];
        for (bucket, count) in tf {
            // Use log-scaled TF: 1 + log(count)
            vector[bucket] = 1.0 + count.ln();
        }

        Self::normalize(&mut vector);
        Ok(vector)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl EmbeddingError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            EmbeddingError::EmptyInput => hudhudscript_errors::ErrorCode::EmbeddingEmptyInput,
            EmbeddingError::InvalidDimensions(..) => {
                hudhudscript_errors::ErrorCode::EmbeddingInvalidDimensions
            }
            EmbeddingError::ProviderError(..) => {
                hudhudscript_errors::ErrorCode::EmbeddingProviderError
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<EmbeddingError> for hudhudscript_errors::Error {
    fn from(e: EmbeddingError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
