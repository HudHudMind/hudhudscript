//! Resource Registry and Manager

use crate::schema::{CachedResource, ResourceContent, ResourceMetadata, ResourceSchema};
use hudhudscript_mcp::client::McpClient;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Resource cache with TTL support
#[derive(Debug, Clone)]
pub struct ResourceCache {
    /// Cached resources
    resources: Arc<RwLock<HashMap<String, CachedResource>>>,

    /// Default TTL for cached resources
    default_ttl: Duration,
}

impl ResourceCache {
    /// Create new resource cache
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Get cached resource
    pub fn get(&self, uri: &str) -> Option<CachedResource> {
        let cache = self.resources.read().ok()?;
        let cached = cache.get(uri)?;

        // Check if cache is still valid
        if cached.is_valid(self.default_ttl) {
            Some(cached.clone())
        } else {
            None
        }
    }

    /// Put resource in cache
    pub fn put(&self, uri: String, resource: CachedResource) {
        if let Ok(mut cache) = self.resources.write() {
            cache.insert(uri, resource);
        }
    }

    /// Remove resource from cache
    pub fn remove(&self, uri: &str) {
        if let Ok(mut cache) = self.resources.write() {
            cache.remove(uri);
        }
    }

    /// Clear all cached resources
    pub fn clear(&self) {
        if let Ok(mut cache) = self.resources.write() {
            cache.clear();
        }
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.resources.read().map(|c| c.len()).unwrap_or(0)
    }
}

/// Resource Manager for MCP resources
#[derive(Clone)]
pub struct ResourceManager {
    /// MCP client for resource operations
    mcp_client: Arc<McpClient>,

    /// Resource metadata registry
    resources: Arc<RwLock<HashMap<String, ResourceMetadata>>>,

    /// Resource cache
    cache: ResourceCache,
}

impl ResourceManager {
    /// Create new resource manager
    pub fn new(mcp_client: Arc<McpClient>, cache_ttl: Duration) -> Self {
        Self {
            mcp_client,
            resources: Arc::new(RwLock::new(HashMap::new())),
            cache: ResourceCache::new(cache_ttl),
        }
    }

    /// Discover resources from MCP servers
    pub async fn discover_resources(
        &self,
        server_name: &str,
    ) -> Result<Vec<ResourceSchema>, ResourceError> {
        // Get list of resources from MCP client
        let response = self
            .mcp_client
            .list_resources(None)
            .await
            .map_err(|e| ResourceError::DiscoveryFailed(e.to_string()))?;

        let mut schemas = Vec::new();

        // Store metadata
        if let Ok(mut registry) = self.resources.write() {
            for resource in response.resources {
                let schema = ResourceSchema {
                    uri: resource.uri.clone(),
                    name: resource.name.clone(),
                    description: resource.description.clone(),
                    mime_type: resource.mime_type.clone(),
                    server: server_name.to_string(),
                };

                let metadata = ResourceMetadata::new(
                    resource.uri.clone(),
                    resource.name.clone(),
                    server_name.to_string(),
                    resource.description.clone(),
                    resource.mime_type.clone(),
                );

                registry.insert(resource.uri.clone(), metadata);
                schemas.push(schema);
            }
        }

        Ok(schemas)
    }

    /// Get resource metadata
    pub fn get_metadata(&self, uri: &str) -> Option<ResourceMetadata> {
        self.resources.read().ok()?.get(uri).cloned()
    }

    /// List all resources
    pub fn list_resources(&self) -> Vec<ResourceMetadata> {
        self.resources
            .read()
            .map(|r| r.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Search resources by name pattern
    pub fn search_resources(&self, pattern: &str) -> Vec<ResourceMetadata> {
        self.resources
            .read()
            .map(|r| {
                r.values()
                    .filter(|m| m.name.contains(pattern))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Read resource content
    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContent, ResourceError> {
        // Check cache first
        if let Some(cached) = self.cache.get(uri) {
            // Update access statistics
            if let Ok(mut registry) = self.resources.write() {
                if let Some(metadata) = registry.get_mut(uri) {
                    metadata.record_access();
                }
            }
            return Ok(cached.content);
        }

        // Get metadata
        let mut metadata = self
            .get_metadata(uri)
            .ok_or_else(|| ResourceError::NotFound(uri.to_string()))?;

        // Read from MCP server
        let response = self
            .mcp_client
            .read_resource(uri.to_string())
            .await
            .map_err(|e| ResourceError::ReadFailed(e.to_string()))?;

        // Convert to ResourceContent
        let content = if let Some(text) = response.contents.first().and_then(|c| c.text.as_ref()) {
            ResourceContent::Text(text.clone())
        } else if let Some(blob) = response.contents.first().and_then(|c| c.blob.as_ref()) {
            ResourceContent::Binary(blob.clone())
        } else {
            return Err(ResourceError::ReadFailed(
                "No content in response".to_string(),
            ));
        };

        // Update metadata
        metadata.record_access();
        if let Ok(mut registry) = self.resources.write() {
            registry.insert(uri.to_string(), metadata.clone());
        }

        // Cache the resource
        let cached = CachedResource::new(metadata, content.clone(), None);
        self.cache.put(uri.to_string(), cached);

        Ok(content)
    }

    /// Invalidate cached resource
    pub fn invalidate_cache(&self, uri: &str) {
        self.cache.remove(uri);
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.size(),
            ttl: self.cache.default_ttl,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached resources
    pub size: usize,

    /// Default TTL
    pub ttl: Duration,
}

/// Resource errors
#[derive(Debug)]
pub enum ResourceError {
    DiscoveryFailed(String),
    NotFound(String),
    ReadFailed(String),
    InvalidUri(String),
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            ResourceError::DiscoveryFailed(s) => write!(f, "Resource discovery failed: {}", s),
            ResourceError::NotFound(s) => write!(f, "Resource not found: {}", s),
            ResourceError::ReadFailed(s) => write!(f, "Resource read failed: {}", s),
            ResourceError::InvalidUri(s) => write!(f, "Invalid resource URI {}", s),
        }
    }
}

impl std::error::Error for ResourceError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl ResourceError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            ResourceError::DiscoveryFailed(..) => {
                hudhudscript_errors::ErrorCode::ResourceDiscoveryFailed
            }
            ResourceError::InvalidUri(..) => hudhudscript_errors::ErrorCode::ResourceInvalidUri,
            ResourceError::NotFound(..) => hudhudscript_errors::ErrorCode::ResourceNotFound,
            ResourceError::ReadFailed(..) => hudhudscript_errors::ErrorCode::ResourceReadFailed,
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

impl From<ResourceError> for hudhudscript_errors::Error {
    fn from(e: ResourceError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
