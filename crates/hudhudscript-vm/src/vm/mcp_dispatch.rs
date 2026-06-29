//! Shared MCP tool-call dispatch (Kural 7).
//!
//! Both the interpreter and the VM now route `mcp.ServerName.toolName(args)`
//! and `mcp.call(server, tool, args)` through the same function here. Each
//! runtime implements the [`McpContext`] trait to supply its own
//! permission / constitution / sandbox checks and its own MCP client lookup.
//! The actual async call to [`hudhudscript_mcp::McpClient::call_tool`] and
//! the [`hudhudscript_mcp::protocol::ToolCallResponse`] → `Value` conversion
//! live here — **a single source of truth** — so the two runtimes cannot drift.

use std::collections::HashMap;
use std::sync::Arc;

use hudhudscript_mcp::protocol::{Content, ToolCallResponse};
use hudhudscript_mcp::{McpClient, TransportConfig};

use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

/// Transport kind for a script-level `mcp server` declaration.
///
/// Mirrors `hudhudscript_ast::TransportType` but is decoupled so the shared
/// crate does not take a dependency on the AST crate. Callers translate their
/// typed enum into this before invoking [`create_mcp_client_from_config`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTransportKind {
    /// Spawn a child process and speak JSON-RPC over stdio.
    Stdio,
    /// Connect to an HTTP endpoint and speak JSON-RPC over SSE.
    Sse,
}

/// MCP-41: Validate SSE URL for SSRF protection.
/// - Only http/https schemes allowed.
/// - http only with explicit allowlist or localhost.
/// - Blocks private/metadata IPs.
pub fn validate_sse_url(
    server_name: &str,
    url: &str,
    allow_insecure_http: bool,
) -> Result<(), String> {
    let lower = url.to_lowercase();
    let is_https = lower.starts_with("https://");
    let is_http = lower.starts_with("http://");
    if !is_https && !is_http {
        return Err(format!(
            "MCP server '{}': SSE URL must use http:// or https://, got: {}",
            server_name, url
        ));
    }
    // MCP-41: http only allowed with explicit opt-in (localhost or allowlist).
    if is_http && !allow_insecure_http {
        return Err(format!(
            "MCP server '{}': http:// URLs require explicit allow_insecure_http (use https:// or enable insecure HTTP)",
            server_name
        ));
    }
    // Block common SSRF targets.
    let blocked_hosts = [
        "localhost",
        "127.0.0.1",
        "0.0.0.0",
        "[::1]",
        "10.",
        "172.16.",
        "192.168.",                 // private IPv4
        "169.254.",                 // link-local
        "metadata.google.internal", // GCP
        "169.254.169.254",          // AWS
    ];
    let host_part = if is_https {
        lower.strip_prefix("https://").unwrap()
    } else {
        lower.strip_prefix("http://").unwrap()
    };
    let host = host_part.split('/').next().unwrap_or("");
    let host = host.split(':').next().unwrap_or(host);
    for blocked in &blocked_hosts {
        if host == *blocked || host.starts_with(blocked) {
            return Err(format!(
                "MCP server '{}': SSE URL host '{}' is blocked for SSRF protection",
                server_name, host
            ));
        }
    }
    Ok(())
}

/// Create, initialize, and wire up an `McpClient` from a script-level
/// `mcp server` declaration (Kural 7).
///
/// Both the interpreter and the VM need to honour an `mcp server "Name" { ... }`
/// block by spawning a real MCP client (not just recording a declaration
/// object). This function is the single source of truth for that lifecycle:
///
/// 1. Build the transport config (stdio → command+args, sse → url)
/// 2. Construct the `McpClient` asynchronously, bridged into the current
///    (or a fresh) tokio runtime so callers from both sync and async
///    contexts can use it
/// 3. Send the `initialize` handshake
/// 4. Start the response-handler task so subsequent `call_tool` awaits
///    actually resolve
///
/// Returns the live `Arc<McpClient>` ready to be registered via the
/// runtime's `register_mcp_client` (or equivalent). Returns an error string
/// on transport / handshake failure so callers can fold it into their own
/// error type.
pub fn create_mcp_client_from_config(
    name: &str,
    transport: McpTransportKind,
    command: Option<&str>,
    args: &[String],
    url: Option<&str>,
) -> Result<Arc<McpClient>, String> {
    let transport_config = match transport {
        McpTransportKind::Stdio => {
            let cmd = command.ok_or_else(|| {
                format!(
                    "MCP server '{}': stdio transport requires 'command' field",
                    name
                )
            })?;
            TransportConfig::stdio(cmd.to_string(), args.to_vec())
        }
        McpTransportKind::Sse => {
            let u = url.ok_or_else(|| {
                format!("MCP server '{}': SSE transport requires 'url' field", name)
            })?;
            // MCP-41: SSRF protection — validate SSE URL (http only with allowlist).
            validate_sse_url(name, u, false)?;
            TransportConfig::sse(u.to_string())
        }
    };

    // MCP-11: Single-source lifecycle — use connect_initialized helper.
    // Default 5s initialize timeout; bridges async into sync context.
    let connect_fut = async move {
        McpClient::connect_initialized(transport_config, std::time::Duration::from_secs(5))
            .await
            .map_err(|e| e.to_string())
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(connect_fut)),
        Err(_) => futures::executor::block_on(connect_fut),
    }
    .map_err(|e| format!("Failed to connect MCP client '{}': {}", name, e))
}

/// Runtime-provided hooks needed to execute an MCP tool call.
///
/// Each runtime (interpreter, VM, future frontends) implements this trait so
/// the shared dispatcher can enforce security invariants and look up the
/// correct client without knowing which runtime it is running inside.
pub trait McpContext {
    /// Agent permission enforcement (Issue #449).
    ///
    /// Implementations inspect the calling agent's `permission` object for
    /// `deny` / `dangerous` / `allow` lists, or a shorthand string
    /// (`"all"` / `"none"`), and return an error if the call is forbidden.
    fn mcp_check_permission(&self, server: &str, tool: &str) -> SharedResult<()>;

    /// Constitution compliance (active governance policy).
    ///
    /// Called with the server and tool so implementations can add those to an
    /// `EvaluationContext` and run [`hudhudscript_governance::enforcement`]
    /// against the active constitution. Should be a no-op if no constitution
    /// is active.
    fn mcp_check_constitution(&self, server: &str, tool: &str) -> SharedResult<()>;

    /// Sandbox access check (Issue #33).
    ///
    /// MCP calls are IPC / network traffic and must go through the sandbox's
    /// network policy. Implementations typically defer to a
    /// `Sandbox::check_network_access("mcp.{server}", 0)` call.
    fn mcp_check_sandbox(&self, server: &str) -> SharedResult<()>;

    /// Look up a live MCP client by registered server name.
    ///
    /// Returns `None` if the server has not been registered — the dispatcher
    /// then raises a "MCP server not found" runtime error naming the caller
    /// site.
    fn mcp_get_client(&self, server: &str) -> Option<Arc<McpClient>>;
}

/// Execute a full MCP tool call end-to-end.
///
/// The dispatcher runs, in order:
/// 1. `mcp_check_permission` (agent allow/deny lists)
/// 2. `mcp_check_constitution` (active governance policy)
/// 3. `mcp_check_sandbox` (network / IPC policy)
/// 4. `mcp_get_client` (resolve the live client)
/// 5. Value → `serde_json::Value` conversion
/// 6. `client.call_tool(...)` via a tokio runtime bridge that works both
///    inside and outside an existing runtime context
/// 7. `ToolCallResponse` → `V` conversion
///
/// Any step that fails short-circuits with the appropriate error. This is
/// the *only* path both runtimes use — the VM and the interpreter must not
/// carry their own copies of this logic (Kural 7).
pub fn dispatch_mcp_tool_call<C>(
    context: &C,
    server_name: &str,
    tool_name: &str,
    arguments: &Value16,
) -> SharedResult<Value16>
where
    C: McpContext,
{
    context.mcp_check_permission(server_name, tool_name)?;
    context.mcp_check_constitution(server_name, tool_name)?;
    context.mcp_check_sandbox(server_name)?;

    let client = context.mcp_get_client(server_name).ok_or_else(|| {
        runtime_error(format!(
            "MCP server '{}' not found. Did you register it with register_mcp_client?",
            server_name
        ))
    })?;

    // Value → JSON. We go through the shared `value_to_json_string` to keep
    // number formatting and escaping identical across runtimes.
    let json_args = {
        let json_str = crate::json::value_to_json_string(arguments);
        serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null)
    };

    // Async → sync bridge. Inside a tokio context we use `block_in_place`
    // to avoid blocking a worker thread; outside any runtime we fall back
    // to the lightweight futures executor.
    let tool_name_owned = tool_name.to_string();
    let tool_fut = async move { client.call_tool(tool_name_owned, Some(json_args)).await };
    let result = match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(tool_fut)),
        Err(_) => futures::executor::block_on(tool_fut),
    };

    match result {
        Ok(response) => mcp_response_to_value(&response),
        Err(e) => Err(runtime_error(format!(
            "MCP call to '{}.{}' failed: {}",
            server_name, tool_name, e
        ))),
    }
}

/// Convert an [`McpClient`] response into a runtime `Value`.
///
/// - Empty content → `null`
/// - Single content item → the item directly (text string, or an
///   `{ type, data, mimeType }` / `{ type, uri }` object)
/// - Multiple content items → an array of the above shapes
pub fn mcp_response_to_value(response: &ToolCallResponse) -> SharedResult<Value16> {
    if response.content.is_empty() {
        return Ok(Value16::null());
    }
    if response.content.len() == 1 {
        return content_to_value(&response.content[0]);
    }
    let items: SharedResult<Vec<Value16>> = response.content.iter().map(content_to_value).collect();
    Ok(Value16::array(items?))
}

fn content_to_value(content: &Content) -> SharedResult<Value16> {
    match content {
        Content::Text { text } => Ok(Value16::string(text.clone())),
        Content::Image { data, mime_type } => {
            let mut obj = hudhudscript_bytecode::ObjMap::default();
            obj.insert("type".to_string(), Value16::string("image".to_string()));
            obj.insert("data".to_string(), Value16::string(data.clone()));
            obj.insert("mimeType".to_string(), Value16::string(mime_type.clone()));
            Ok(Value16::object(obj))
        }
        Content::Resource { resource } => {
            let mut obj = hudhudscript_bytecode::ObjMap::default();
            obj.insert("type".to_string(), Value16::string("resource".to_string()));
            obj.insert("uri".to_string(), Value16::string(resource.uri.clone()));
            Ok(Value16::object(obj))
        }
    }
}
