use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum StorageEmbeddingErrorCode {
    /// E0075 — Embedding Input Text Is Empty
    EmbeddingEmptyInput = 75,
    /// E0076 — Embedding Vector Has Wrong Dimensionality
    EmbeddingInvalidDimensions = 76,
    /// E0077 — Embedding Provider Returned An Error
    EmbeddingProviderError = 77,
}
