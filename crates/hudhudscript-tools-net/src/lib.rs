//! HudHudScript Tools — Network
//!
//! HTTP client, MCP bridge, and OpenAPI discovery.

pub mod http;
pub mod mcp_bridge;
pub mod openapi;

pub use http::{
    HttpAuth, HttpMethod, HttpRequest, HttpResponse, HttpTool, HttpToolError, RestResource,
};
pub use mcp_bridge::{wire_client, wire_clients};
pub use openapi::{
    build_parameters_schema, derive_tool_name, discover_tools_from_openapi, extract_param_type,
    import_openapi_tools, sanitize_name, DiscoveredTool, OpenApiDocument,
};
