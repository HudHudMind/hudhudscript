//! Cache serialization with metadata support
//!
//! This module provides dedicated serialization functionality for the command cache,
//! including metadata (version, timestamp, item count) and validation.

use crate::cache::{CacheError, CommandCache};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Serialization format version
pub const SERIALIZATION_VERSION: u32 = 1;

/// Cache serialization wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSerializationFormat {
    /// Format version for compatibility checking
    pub version: u32,

    /// Timestamp when serialization was created
    pub timestamp: DateTime<Utc>,

    /// Total number of items in cache
    pub item_count: usize,

    /// The actual cache data
    pub cache: CommandCache,
}

impl CacheSerializationFormat {
    /// Create a new serialization format from a cache
    pub fn new(cache: CommandCache) -> Self {
        let item_count = cache.constitutions.len() + cache.laws.len() + cache.rules.len();

        Self {
            version: SERIALIZATION_VERSION,
            timestamp: Utc::now(),
            item_count,
            cache,
        }
    }

    /// Validate the serialization format
    pub fn validate(&self) -> Result<(), CacheError> {
        // Check version compatibility
        if self.version > SERIALIZATION_VERSION {
            return Err(CacheError::DeserializationError(format!(
                "Unsupported serialization version: {} (current: {})",
                self.version, SERIALIZATION_VERSION
            )));
        }

        // Validate item count matches actual cache contents
        let actual_count =
            self.cache.constitutions.len() + self.cache.laws.len() + self.cache.rules.len();

        if self.item_count != actual_count {
            return Err(CacheError::DeserializationError(format!(
                "Item count mismatch: metadata says {} but cache contains {}",
                self.item_count, actual_count
            )));
        }

        Ok(())
    }
}

/// Serialize cache definitions with metadata
///
/// Produces compact JSON format with UTF-8 encoding.
/// Includes metadata: version, timestamp, item count.
///
/// # Arguments
/// * `cache` - The command cache to serialize
///
/// # Returns
/// * `Ok(String)` - Compact JSON string with UTF-8 encoding
/// * `Err(CacheError)` - Serialization error
///
/// # Requirements
/// Validates: Requirements 5.5, 5.6, 16.1, 16.2, 28.1, 28.2
pub fn serialize_definitions(cache: &CommandCache) -> Result<String, CacheError> {
    let format = CacheSerializationFormat::new(cache.clone());

    // Use compact JSON format (no pretty printing)
    serde_json::to_string(&format).map_err(|e| CacheError::SerializationError(e.to_string()))
}

/// Deserialize cache definitions with validation
///
/// Validates the serialization format version and item count before
/// restoring the cache.
///
/// # Arguments
/// * `data` - JSON string with UTF-8 encoding
///
/// # Returns
/// * `Ok(CommandCache)` - Restored cache with validated data
/// * `Err(CacheError)` - Deserialization or validation error
///
/// # Requirements
/// Validates: Requirements 5.6, 16.2, 16.3, 28.1, 28.4
pub fn deserialize_definitions(data: &str) -> Result<CommandCache, CacheError> {
    // Deserialize the format wrapper
    let format: CacheSerializationFormat =
        serde_json::from_str(data).map_err(|e| CacheError::DeserializationError(e.to_string()))?;

    // Validate before returning
    format.validate()?;

    // Return the validated cache
    Ok(format.cache)
}
