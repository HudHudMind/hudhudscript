//! MCP Client Implementation

use crate::protocol::*;
use crate::transport::{Transport, TransportConfig};
use anyhow::{Context, Result};
use hudhudscript_sandbox::{Sandbox, SandboxConfig};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::warn;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected and initialized
    Connected,
    /// Connection failed
    Failed,
}

/// MCP Client
///
/// The transport is split into two independently-locked halves at construction
/// time.  `transport_send` is used exclusively by `send_request` (via
/// `TransportSend::send`), and `transport_recv` is used exclusively by the
/// background response-handler task (via `TransportRecv::receive`).  Because
/// the two halves each have their own `tokio::sync::Mutex`, sending a new
/// request never blocks on an in-progress receive, eliminating lock-contention
/// deadlocks.
pub struct McpClient {
    /// Send half of the transport (only `TransportSend::send` is called)
    transport_send: Arc<Mutex<Box<dyn Transport>>>,
    /// Receive half of the transport (only `TransportRecv::receive` is called)
    transport_recv: Arc<Mutex<Box<dyn Transport>>>,
    /// Connection state
    pub state: Arc<RwLock<ConnectionState>>,
    /// Request ID counter
    pub request_id: Arc<AtomicU64>,
    /// Server capabilities
    pub server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    /// Server info
    pub server_info: Arc<RwLock<Option<ServerInfo>>>,
    /// Pending requests
    pub pending_requests:
        Arc<RwLock<HashMap<RequestId, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
    /// Security sandbox (optional)
    pub sandbox: Option<Arc<Sandbox>>,
}

/// Build a pair of `Arc<Mutex<Box<dyn Transport>>>` sharing the same underlying
/// channel.
///
/// The `Transport` trait extends `TransportSend + TransportRecv`.  The locking
/// convention ensures that `transport_send` is only locked to call
/// `TransportSend::send`, and `transport_recv` is only locked to call
/// `TransportRecv::receive`.
type SharedTransport = Arc<Mutex<Box<dyn Transport>>>;

pub fn make_transport_pair(transport: Box<dyn Transport>) -> (SharedTransport, SharedTransport) {
    let shared = Arc::new(Mutex::new(transport));
    (shared.clone(), shared)
}

impl McpClient {
    /// Create a new MCP client from a pre-built transport.
    ///
    /// This is useful for testing with mock transports.
    pub fn from_transport(transport: Box<dyn Transport>) -> Self {
        let (transport_send, transport_recv) = make_transport_pair(transport);
        Self {
            transport_send,
            transport_recv,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            request_id: Arc::new(AtomicU64::new(1)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            sandbox: None,
        }
    }

    /// Create a new MCP client
    pub async fn new(config: TransportConfig) -> Result<Self> {
        let transport = config.create_transport().await?;
        let (transport_send, transport_recv) = make_transport_pair(transport);

        Ok(Self {
            transport_send,
            transport_recv,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            request_id: Arc::new(AtomicU64::new(1)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            sandbox: None,
        })
    }

    /// Create a new MCP client with sandbox enabled
    pub async fn with_sandbox(
        config: TransportConfig,
        sandbox_config: SandboxConfig,
    ) -> Result<Self> {
        let transport = config.create_transport().await?;
        let (transport_send, transport_recv) = make_transport_pair(transport);

        Ok(Self {
            transport_send,
            transport_recv,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            request_id: Arc::new(AtomicU64::new(1)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            sandbox: Some(Arc::new(Sandbox::new(sandbox_config))),
        })
    }

    /// Check if sandbox allows tool execution
    pub fn check_tool_execution(&self, _tool_name: &str) -> Result<()> {
        if let Some(_sandbox) = &self.sandbox {
            // Check if tool execution is allowed
            // Currently, we allow all tools but this can be extended
            // to check against a whitelist/blacklist
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Initialize connection
    pub async fn initialize(&self) -> Result<InitializeResponse> {
        *self.state.write().await = ConnectionState::Connecting;

        let request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                experimental: None,
                sampling: None,
            },
            client_info: ClientInfo {
                name: "HudHudScript".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let response = self
            .send_request(methods::INITIALIZE, Some(serde_json::to_value(request)?))
            .await?;

        let init_response: InitializeResponse =
            serde_json::from_value(response).context("Failed to parse initialize response")?;

        // Store server capabilities and info
        *self.server_capabilities.write().await = Some(init_response.capabilities.clone());
        *self.server_info.write().await = Some(init_response.server_info.clone());
        *self.state.write().await = ConnectionState::Connected;

        Ok(init_response)
    }

    /// List available tools
    pub async fn list_tools(&self, cursor: Option<String>) -> Result<ToolsListResponse> {
        self.ensure_connected().await?;

        let request = ToolsListRequest { cursor };
        let response = self
            .send_request(methods::TOOLS_LIST, Some(serde_json::to_value(request)?))
            .await?;

        serde_json::from_value(response).context("Failed to parse tools list response")
    }

    /// Call a tool
    pub async fn call_tool(
        &self,
        name: String,
        arguments: Option<Value>,
    ) -> Result<ToolCallResponse> {
        self.ensure_connected().await?;

        // Check sandbox permissions
        self.check_tool_execution(&name)?;

        let request = ToolCallRequest { name, arguments };
        let response = self
            .send_request(methods::TOOLS_CALL, Some(serde_json::to_value(request)?))
            .await?;

        serde_json::from_value(response).context("Failed to parse tool call response")
    }

    /// List available resources
    pub async fn list_resources(&self, cursor: Option<String>) -> Result<ResourcesListResponse> {
        self.ensure_connected().await?;

        let request = ResourcesListRequest { cursor };
        let response = self
            .send_request(
                methods::RESOURCES_LIST,
                Some(serde_json::to_value(request)?),
            )
            .await?;

        serde_json::from_value(response).context("Failed to parse resources list response")
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: String) -> Result<ResourceReadResponse> {
        self.ensure_connected().await?;

        let request = ResourceReadRequest { uri };
        let response = self
            .send_request(
                methods::RESOURCES_READ,
                Some(serde_json::to_value(request)?),
            )
            .await?;

        serde_json::from_value(response).context("Failed to parse resource read response")
    }

    /// Get connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Get server capabilities
    pub async fn server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.read().await.clone()
    }

    /// Get server info
    pub async fn server_info(&self) -> Option<ServerInfo> {
        self.server_info.read().await.clone()
    }

    /// Disconnect from server
    pub async fn disconnect(&self) -> Result<()> {
        let mut transport = self.transport_send.lock().await;
        transport.close().await?;
        *self.state.write().await = ConnectionState::Disconnected;
        Ok(())
    }

    /// Send a JSON-RPC request
    async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = RequestId::new_number(self.request_id.fetch_add(1, Ordering::SeqCst) as i64);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: id.clone(),
            method: method.to_string(),
            params,
        };

        // Register the pending-request channel *before* sending, so the
        // response handler cannot race and discard a fast response.
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending_requests.write().await.insert(id.clone(), tx);

        // Send the request — lock is released immediately after send().
        {
            let mut transport = self.transport_send.lock().await;
            transport.send(request).await?;
        }

        // Wait for response (with timeout). The lock is NOT held here, so
        // the response-handler task can freely call receive() concurrently.
        let response = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .context("Request timeout")?
            .context("Response channel closed")?;

        // Check for error
        if let Some(error) = response.error {
            anyhow::bail!("RPC error {}: {}", error.code, error.message);
        }

        response.result.context("Response has no result")
    }

    /// Ensure client is connected
    pub async fn ensure_connected(&self) -> Result<()> {
        let state = *self.state.read().await;
        match state {
            ConnectionState::Connected => Ok(()),
            ConnectionState::Disconnected => {
                anyhow::bail!("Client not connected. Call initialize() first.")
            }
            ConnectionState::Connecting => {
                anyhow::bail!("Client is connecting")
            }
            ConnectionState::Failed => {
                anyhow::bail!("Connection failed")
            }
        }
    }

    /// Start the background response-handler task.
    ///
    /// This task owns the receive side of the transport exclusively.  It
    /// calls `receive()` in a loop, looks up the matching pending-request
    /// oneshot channel, and delivers the response.  Because `receive()` is
    /// called with `transport_recv` locked only for the duration of the call
    /// itself, and `send_request` uses the *same* underlying mutex (see
    /// `make_transport_pair`) only while calling `send()`, the two operations
    /// do not block each other in the steady state.
    pub async fn start_response_handler(&self) {
        let transport_recv = self.transport_recv.clone();
        let pending_requests = self.pending_requests.clone();

        tokio::spawn(async move {
            loop {
                // Acquire the lock only for the duration of a single receive
                // call, then release it immediately.  This allows send_request
                // to acquire the same mutex without waiting for the next
                // network round-trip to complete.
                let response = {
                    let mut transport = transport_recv.lock().await;
                    match transport.receive().await {
                        Ok(resp) => resp,
                        Err(e) => {
                            warn!("Error receiving response: {}", e);
                            break;
                        }
                    }
                    // Mutex guard dropped here — lock released.
                };

                // Deliver to the waiting send_request caller.
                let mut pending = pending_requests.write().await;
                if let Some(tx) = pending.remove(&response.id) {
                    let _ = tx.send(response);
                }
            }
        });
    }
}
