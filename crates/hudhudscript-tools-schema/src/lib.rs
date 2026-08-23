//! HudHudScript Tools — Schema Validation and Tool Registry
//!
//! Provides JSON Schema validation and a tool registry for MCP tools.

pub mod registry;
mod registry_error;
pub mod schema;

pub use registry::{CacheStats, NativeToolHandler, ToolCache, ToolRegistry};
pub use registry_error::RegistryError;
pub use schema::{
    validate_property_type, value_type_name, JsonSchema, JsonSchemaProperty, ToolMetadata,
    ToolSchema, ValidationError,
};
