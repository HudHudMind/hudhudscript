//! HudHudScript Tools — Schema Validation and Tool Registry
//!
//! Provides JSON Schema validation and a tool registry for MCP tools.

pub mod registry;
pub mod schema;

pub use registry::{CacheStats, RegistryError, ToolCache, ToolRegistry};
pub use schema::{
    validate_property_type, value_type_name, JsonSchema, JsonSchemaProperty, ToolMetadata,
    ToolSchema, ValidationError,
};
