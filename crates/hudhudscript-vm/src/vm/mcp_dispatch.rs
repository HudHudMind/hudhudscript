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
            let cmd = command.unwrap_or("echo").to_string();
            TransportConfig::stdio(cmd, args.to_vec())
        }
        McpTransportKind::Sse => {
            let u = url.unwrap_or("http://localhost:8080").to_string();
            TransportConfig::sse(u)
        }
    };

    // Bridge async client creation into sync context (same pattern the
    // dispatcher uses in `dispatch_mcp_tool_call`).
    let client_fut = async move {
        McpClient::new(transport_config)
            .await
            .map(Arc::new)
            .map_err(|e| e.to_string())
    };
    let client = match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(client_fut)),
        Err(_) => futures::executor::block_on(client_fut),
    }
    .map_err(|e| format!("Failed to create MCP client for '{}': {}", name, e))?;

    // Best-effort initialize + start the response-handler task. Matches
    // the interpreter's `execute_mcp_server_decl` behaviour (initialize
    // can fail if the downstream server process has not booted yet; we
    // still register the client so later calls can retry).
    {
        let client_clone = client.clone();
        let init_fut = async move {
            if client_clone.initialize().await.is_ok() {
                client_clone.start_response_handler().await;
            }
        };
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(init_fut)),
            Err(_) => futures::executor::block_on(init_fut),
        }
    }

    Ok(client)
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
            let mut obj = HashMap::new();
            obj.insert("type".to_string(), Value16::string("image".to_string()));
            obj.insert("data".to_string(), Value16::string(data.clone()));
            obj.insert("mimeType".to_string(), Value16::string(mime_type.clone()));
            Ok(Value16::object(obj))
        }
        Content::Resource { resource } => {
            let mut obj = HashMap::new();
            obj.insert("type".to_string(), Value16::string("resource".to_string()));
            obj.insert("uri".to_string(), Value16::string(resource.uri.clone()));
            Ok(Value16::object(obj))
        }
    }
}
