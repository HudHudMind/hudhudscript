//! HudHudScript Tool Registry
//!
//! This crate provides tool registry and management for MCP tools.
#![allow(clippy::type_complexity, clippy::manual_range_contains)]
//!
//! As of Issue #677, the tool implementations are split into sub-crates:
//! - `hudhudscript-tools-schema` — schema validation, tool registry
//! - `hudhudscript-tools-io`     — standard tools, database
//! - `hudhudscript-tools-net`    — HTTP, MCP bridge, OpenAPI
//! - `hudhudscript-tools-vcs`    — git operations
//! - `hudhudscript-tools-ai`     — memory/RAG, context
//! - `hudhudscript-tools-ops`    — approval, retry, telemetry
//!
//! This crate re-exports everything for backward compatibility.

// ── Original modules (kept for backward compatibility) ──────────────
pub mod database;
pub mod git;
pub mod http;
pub mod mcp_bridge;
pub mod memory;
pub mod registry;
pub mod schema;
pub mod standard;

pub mod approval;
pub mod context;
pub mod openapi;
pub mod retry;
pub mod telemetry;

// ── Re-exports from original modules ────────────────────────────────
pub use approval::{ApprovalError, ApprovalGate, ApprovalRegistry, ApprovalRequest, ApprovalState};
pub use context::{estimate_tokens, ContextWindow, OutputLimiterConfig, ToolOutputLimiter};
#[cfg(feature = "db")]
pub use database::DatabaseService;
pub use database::{
    register_database_tools, ColumnInfo, DatabaseBackend, DatabaseConfig, DatabaseConnection,
    DatabaseError, DatabaseTool, ExecuteOptions, Migration, MigrationReport, PoolStatus,
    QueryResult, TransactionOptions,
};
pub use git::{register_git_tools, GitError, GitOutput, GitTool};
pub use http::{
    HttpAuth, HttpMethod, HttpRequest, HttpResponse, HttpTool, HttpToolError, RestResource,
};
pub use memory::{InMemoryBackend, MemoryBackend, MemoryEntry, MemoryError, MemoryStore};
pub use openapi::{
    discover_tools_from_openapi, import_openapi_tools, DiscoveredTool, OpenApiDocument,
};
pub use registry::{CacheStats, RegistryError, ToolCache, ToolRegistry};
pub use retry::{RetryPolicy, ToolCallExecutor, ToolCallOutcome};
pub use schema::{JsonSchema, JsonSchemaProperty, ToolMetadata, ToolSchema, ValidationError};
pub use standard::{register_standard_tools, CustomTool, StandardTool, ToolError};
pub use telemetry::{
    record_tool_telemetry, ExecutionStatus, InstrumentedToolExecutor, TelemetryCollector,
    ToolStats, ToolTelemetryRecord,
};

// ── Sub-crate re-exports (Issue #677) ───────────────────────────────
// Consumers can depend on sub-crates directly for a smaller dependency footprint,
// or use this umbrella crate for convenience.
pub mod sub {
    pub use hudhudscript_tools_ai as ai;
    pub use hudhudscript_tools_io as io;
    pub use hudhudscript_tools_net as net;
    pub use hudhudscript_tools_ops as ops;
    pub use hudhudscript_tools_schema as schema;
    pub use hudhudscript_tools_vcs as vcs;
}
