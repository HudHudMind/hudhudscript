//! Tool Registry Implementation

use crate::schema::{JsonSchema, ToolMetadata, ToolSchema, ValidationError};
use hudhudscript_mcp::client::McpClient;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tracing::warn;

/// Tool Registry for managing MCP tools
pub struct ToolRegistry {
    /// Map of tool name to tool schema
    tools: Arc<RwLock<HashMap<String, ToolSchema>>>,

    /// Map of tool name to metadata
    metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,

    /// Tool cache
    cache: Arc<RwLock<ToolCache>>,

    /// MCP clients by server name
    clients: Arc<RwLock<HashMap<String, Arc<McpClient>>>>,
}

/// Tool cache with TTL support
pub struct ToolCache {
    /// Cached tool schemas
    schemas: HashMap<String, CachedSchema>,

    /// Cache TTL (time-to-live)
    ttl: Duration,
}

/// Cached schema with timestamp
struct CachedSchema {
    schema: ToolSchema,
    cached_at: SystemTime,
}

impl ToolRegistry {
    /// Create new tool registry
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(ToolCache::new(Duration::from_secs(300)))),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an MCP client for a server
    pub fn register_client(&self, server_name: String, client: Arc<McpClient>) {
        let mut clients = self.clients.write().unwrap();
        clients.insert(server_name, client);
    }

    /// Discover tools from all registered MCP servers
    pub async fn discover_tools(&self) -> Result<usize, RegistryError> {
        let clients = self.clients.read().unwrap().clone();
        let mut discovered_count = 0;

        for (server_name, client) in clients.iter() {
            match self.discover_tools_from_server(server_name, client).await {
                Ok(count) => discovered_count += count,
                Err(e) => {
                    warn!("Failed to discover tools from {}: {}", server_name, e);
                }
            }
        }

        Ok(discovered_count)
    }

    /// Discover tools from a specific MCP server
    async fn discover_tools_from_server(
        &self,
        server_name: &str,
        client: &Arc<McpClient>,
    ) -> Result<usize, RegistryError> {
        let tools_response = client
            .list_tools(None)
            .await
            .map_err(|e| RegistryError::DiscoveryFailed(e.to_string()))?;

        let mut count = 0;
        for tool in tools_response.tools {
            // Convert Value to JsonSchema
            let input_schema: JsonSchema = serde_json::from_value(tool.input_schema)
                .map_err(|e| RegistryError::DiscoveryFailed(format!("Invalid schema: {}", e)))?;

            let schema = ToolSchema {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema,
                server: server_name.to_string(),
            };

            let metadata =
                ToolMetadata::new(tool.name.clone(), server_name.to_string(), tool.description);

            self.register_tool(schema, metadata)?;
            count += 1;
        }

        Ok(count)
    }

    /// Register a tool with its schema and metadata
    pub fn register_tool(
        &self,
        schema: ToolSchema,
        metadata: ToolMetadata,
    ) -> Result<(), RegistryError> {
        let tool_name = schema.name.clone();

        // Store in registry
        {
            let mut tools = self.tools.write().unwrap();
            tools.insert(tool_name.clone(), schema.clone());
        }

        {
            let mut meta = self.metadata.write().unwrap();
            meta.insert(tool_name.clone(), metadata);
        }

        // Store in cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.put(schema);
        }

        Ok(())
    }

    /// Get tool schema by name
    pub fn get_tool(&self, name: &str) -> Option<ToolSchema> {
        // Try cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(schema) = cache.get(name) {
                return Some(schema);
            }
        }

        // Fall back to registry
        let tools = self.tools.read().unwrap();
        tools.get(name).cloned()
    }

    /// Get tool metadata by name
    pub fn get_metadata(&self, name: &str) -> Option<ToolMetadata> {
        let metadata = self.metadata.read().unwrap();
        metadata.get(name).cloned()
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().unwrap();
        tools.keys().cloned().collect()
    }

    /// Validate tool arguments against schema
    pub fn validate_arguments(
        &self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<(), ValidationError> {
        let schema = self.get_tool(tool_name).ok_or_else(|| {
            ValidationError::UnknownType(format!("Tool not found: {}", tool_name))
        })?;

        schema.input_schema.validate(arguments)
    }

    /// Call a tool with arguments
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, RegistryError> {
        // Validate arguments
        self.validate_arguments(tool_name, &arguments)
            .map_err(RegistryError::ValidationFailed)?;

        // Get tool schema to find server
        let schema = self
            .get_tool(tool_name)
            .ok_or_else(|| RegistryError::ToolNotFound(tool_name.to_string()))?;

        // Get MCP client for server
        let client = {
            let clients = self.clients.read().unwrap();
            clients
                .get(&schema.server)
                .cloned()
                .ok_or_else(|| RegistryError::ServerNotFound(schema.server.clone()))?
        };

        // Call tool via MCP client
        let result = client
            .call_tool(tool_name.to_string(), Some(arguments))
            .await
            .map_err(|e| RegistryError::CallFailed(e.to_string()))?;

        // Update metadata
        {
            let mut metadata = self.metadata.write().unwrap();
            if let Some(meta) = metadata.get_mut(tool_name) {
                meta.record_usage();
            }
        }

        // Convert content to JSON value
        serde_json::to_value(result.content)
            .map_err(|e| RegistryError::CallFailed(format!("Failed to serialize result: {}", e)))
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        CacheStats {
            size: cache.size(),
            ttl: cache.ttl,
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCache {
    /// Create new tool cache with TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            schemas: HashMap::new(),
            ttl,
        }
    }

    /// Put schema in cache
    pub fn put(&mut self, schema: ToolSchema) {
        let cached = CachedSchema {
            schema,
            cached_at: SystemTime::now(),
        };
        self.schemas.insert(cached.schema.name.clone(), cached);
    }

    /// Get schema from cache if not expired
    pub fn get(&self, name: &str) -> Option<ToolSchema> {
        if let Some(cached) = self.schemas.get(name) {
            if let Ok(elapsed) = cached.cached_at.elapsed() {
                if elapsed < self.ttl {
                    return Some(cached.schema.clone());
                }
            }
        }
        None
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.schemas.clear();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.schemas.len()
    }

    /// Remove expired entries
    pub fn cleanup(&mut self) {
        self.schemas.retain(|_, cached| {
            if let Ok(elapsed) = cached.cached_at.elapsed() {
                elapsed < self.ttl
            } else {
                false
            }
        });
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub ttl: Duration,
}

/// Registry error types
#[derive(Debug, Clone)]
pub enum RegistryError {
    ToolNotFound(String),
    ServerNotFound(String),
    DiscoveryFailed(String),
    CallFailed(String),
    DuplicateTool(String),
    ValidationFailed(ValidationError),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            RegistryError::ToolNotFound(s) => write!(f, "Tool not found: {}", s),
            RegistryError::ServerNotFound(s) => write!(f, "Server not found: {}", s),
            RegistryError::DiscoveryFailed(s) => write!(f, "Tool discovery failed: {}", s),
            RegistryError::CallFailed(s) => write!(f, "Tool call failed: {}", s),
            RegistryError::DuplicateTool(s) => write!(f, "Duplicate tool: {}", s),
            RegistryError::ValidationFailed(e) => write!(f, "Validation failed: {}", e),
        }
    }
}

impl std::error::Error for RegistryError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl RegistryError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            RegistryError::CallFailed(..) => hudhudscript_errors::ErrorCode::RegistryCallFailed,
            RegistryError::DiscoveryFailed(..) => {
                hudhudscript_errors::ErrorCode::RegistryDiscoveryFailed
            }
            RegistryError::DuplicateTool(..) => {
                hudhudscript_errors::ErrorCode::RegistryDuplicateTool
            }
            RegistryError::ServerNotFound(..) => {
                hudhudscript_errors::ErrorCode::RegistryServerNotFound
            }
            RegistryError::ToolNotFound(..) => hudhudscript_errors::ErrorCode::RegistryToolNotFound,
            RegistryError::ValidationFailed(..) => {
                hudhudscript_errors::ErrorCode::RegistryValidationFailed
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

impl From<RegistryError> for hudhudscript_errors::Error {
    fn from(e: RegistryError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
