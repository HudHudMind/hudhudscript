use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

mod cache;
pub use cache::*;
mod embedding;
pub use embedding::*;
mod index;
pub use index::*;
mod persistence;
pub use persistence::*;
mod store;
pub use store::*;

pub static TABLE: &[ExceptionEntry] = &[
    CACHE_CONSTITUTION_NOT_FOUND,
    CACHE_DESERIALIZATION_ERROR,
    CACHE_DUPLICATE_CONTENT,
    CACHE_ID_COLLISION,
    CACHE_LAW_NOT_FOUND,
    CACHE_QUOTA_EXCEEDED,
    CACHE_RULE_NOT_FOUND,
    CACHE_SERIALIZATION_ERROR,
    EMBEDDING_EMPTY_INPUT,
    EMBEDDING_INVALID_DIMENSIONS,
    EMBEDDING_PROVIDER_ERROR,
    INDEX_DUPLICATE_ID,
    INDEX_EMBEDDING,
    INDEX_NOT_FOUND,
    PERSISTENCE_IO,
    PERSISTENCE_NOT_FOUND,
    PERSISTENCE_SERIALIZATION,
    STORE_DIMENSION_MISMATCH,
    STORE_INVALID_CONFIG,
    STORE_NOT_FOUND,
    STORE_PERSIST_ERROR,
];
