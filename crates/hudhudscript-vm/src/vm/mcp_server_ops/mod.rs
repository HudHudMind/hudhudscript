//! Shared MCP Server mode builtins (Issue #600, #861).
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).
//! Uses a process-global `McpServerRecord` registry keyed by server name,
//! so tool/resource registrations persist across the runtime.

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

mod protocol;
mod server;
mod tools;

pub use protocol::{
    default_input_schema, evict_if_full, resource_to_value, server_to_value, state, tool_to_value,
    McpState, ResourceRecord, ServerRecord, ToolRecord, MAX_SERVERS, PROTOCOL_VERSION,
};
pub use server::{mcp_server_create, mcp_server_start, mcp_server_status, mcp_server_stop};
pub use tools::{
    mcp_server_add_resource, mcp_server_add_tool, mcp_server_resources, mcp_server_tools,
};

pub fn call_mcp_server_method(method: &str, args: &[Value16]) -> SharedResult<Value16> {
    match method {
        "create" => server::mcp_server_create(args),
        "add_tool" => tools::mcp_server_add_tool(args),
        "add_resource" => tools::mcp_server_add_resource(args),
        "start" => server::mcp_server_start(args),
        "stop" => server::mcp_server_stop(args),
        "tools" => tools::mcp_server_tools(args),
        "resources" => tools::mcp_server_resources(args),
        "status" => server::mcp_server_status(args),
        _ => Err(runtime_error(format!(
            "Unknown McpServer method: {}",
            method
        ))),
    }
}
