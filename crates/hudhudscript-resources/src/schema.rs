//! Resource Schema Definitions

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Resource schema with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSchema {
    /// Resource URI
    pub uri: String,

    /// Resource name
    pub name: String,

    /// Resource description
    pub description: Option<String>,

    /// MIME type
    pub mime_type: Option<String>,

    /// Server that provides this resource
    pub server: String,
}

/// Resource metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// Resource URI
    pub uri: String,

    /// Resource name
    pub name: String,

    /// Resource description
    pub description: Option<String>,

    /// MIME type
    pub mime_type: Option<String>,

    /// Server name
    pub server: String,

    /// When was this resource discovered
    pub discovered_at: SystemTime,

    /// Last accessed timestamp
    pub last_accessed: Option<SystemTime>,

    /// Access count
    pub access_count: u64,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// ETag for cache validation
    pub etag: Option<String>,
}

impl ResourceMetadata {
    /// Create new resource metadata
    pub fn new(
        uri: String,
        name: String,
        server: String,
        description: Option<String>,
        mime_type: Option<String>,
    ) -> Self {
        Self {
            uri,
            name,
            description,
            mime_type,
            server,
            discovered_at: SystemTime::now(),
            last_accessed: None,
            access_count: 0,
            tags: Vec::new(),
            etag: None,
        }
    }

    /// Record resource access
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Some(SystemTime::now());
    }

    /// Add tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Update ETag
    pub fn update_etag(&mut self, etag: String) {
        self.etag = Some(etag);
    }
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceContent {
    /// Text content
    Text(String),

    /// Binary content (base64 encoded)
    Binary(String),
}

impl ResourceContent {
    /// Get content as text
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ResourceContent::Text(text) => Some(text),
            ResourceContent::Binary(_) => None,
        }
    }

    /// Get content as binary
    pub fn as_binary(&self) -> Option<&str> {
        match self {
            ResourceContent::Text(_) => None,
            ResourceContent::Binary(data) => Some(data),
        }
    }

    /// Check if content is text
    pub fn is_text(&self) -> bool {
        matches!(self, ResourceContent::Text(_))
    }

    /// Check if content is binary
    pub fn is_binary(&self) -> bool {
        matches!(self, ResourceContent::Binary(_))
    }
}

/// Cached resource with content
#[derive(Debug, Clone)]
pub struct CachedResource {
    /// Resource metadata
    pub metadata: ResourceMetadata,

    /// Resource content
    pub content: ResourceContent,

    /// When was this cached
    pub cached_at: SystemTime,

    /// ETag for validation
    pub etag: Option<String>,
}

impl CachedResource {
    /// Create new cached resource
    pub fn new(metadata: ResourceMetadata, content: ResourceContent, etag: Option<String>) -> Self {
        Self {
            metadata,
            content,
            cached_at: SystemTime::now(),
            etag,
        }
    }

    /// Check if cache is still valid
    pub fn is_valid(&self, ttl: std::time::Duration) -> bool {
        if let Ok(elapsed) = self.cached_at.elapsed() {
            elapsed < ttl
        } else {
            false
        }
    }
}
