//! HudHudScript Resource Manager
//!
//! This crate provides resource management for MCP resources.

pub mod registry;
pub mod schema;

pub use registry::{CacheStats, ResourceCache, ResourceError, ResourceManager};
pub use schema::{CachedResource, ResourceContent, ResourceMetadata, ResourceSchema};
