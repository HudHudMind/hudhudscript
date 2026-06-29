//! MCP Client — main entry point for MCP server connections.
//!
//! Uses independent send/receive transport halves (no shared lock).
//! The response handler runs a background task reading from the receive half
//! and dispatching to pending requests.

use crate::protocol::*;
use crate::transport::config::TransportConfig;
use crate::transport::{TransportSendHalf, TransportRecvHalf};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex, RwLock};
use tracing::warn;

/// MCP-43: Maximum size of a JSON-RPC request body (256 KiB).
pub const MAX_REQUEST_SIZE: usize = 256 * 1024;
/// MCP-43: Maximum size of a JSON-RPC response body (1 MiB).
pub const MAX_RESPONSE_SIZE: usize = 1024 * 1024;
/// MCP-42: Maximum number of tools a server may advertise.
pub const MAX_TOOLS_PER_SERVER: usize = 256;
/// MCP-43: Maximum number of concurrent pending requests.
pub const MAX_PENDING_REQUESTS: usize = 32;
/// MCP-43: Default per-request timeout.
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

type PendingResult = Result<serde_json::Value, String>;
type SharedPending = Arc<Mutex<HashMap<RequestId, oneshot::Sender<PendingResult>>>>;

/// Shared transport for backward compat — single Arc<Mutex<Box<dyn Transport>>>
/// used by `from_transport` and `start_response_handler_compat`.
type SharedTransport = Arc<tokio::sync::Mutex<Box<dyn crate::transport::Transport>>>;

/// MCP Client
pub struct McpClient {
    transport_send: Mutex<TransportSendHalf>,
    state: Arc<RwLock<ConnectionState>>,
    request_id: Arc<AtomicU64>,
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    server_info: Arc<RwLock<Option<ServerInfo>>>,
    pending_requests: SharedPending,
    /// Abort handle for the response handler task (set when spawned).
    response_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Shared transport for backward compat (from_transport + start_response_handler_compat).
    shared_transport: Option<SharedTransport>,
    /// MCP-42: Cached tool definitions from tools/list.
    tool_cache: RwLock<Option<Vec<Tool>>>,
    /// MCP-50: Request counter (success + failure).
    request_count: AtomicU64,
    /// MCP-50: Timeout counter.
    timeout_count: AtomicU64,
    /// MCP-50: Last error message.
    last_error: RwLock<Option<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed(String),
}

impl McpClient {
    pub async fn new(config: TransportConfig) -> Result<Self> {
        let transport = config.create_transport().await?;
        let (send, recv) = transport.split();
        let client = Self {
            transport_send: Mutex::new(send),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            request_id: Arc::new(AtomicU64::new(1)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            response_handle: Mutex::new(None),
            shared_transport: None,
            tool_cache: RwLock::new(None),
            request_count: AtomicU64::new(0),
            timeout_count: AtomicU64::new(0),
            last_error: RwLock::new(None),
        };
        client.spawn_response_handler(recv);
        Ok(client)
    }

    /// Spawn a background response handler using a TransportRecvHalf.
    fn spawn_response_handler(&self, recv: TransportRecvHalf) {
        let pending = self.pending_requests.clone();
        let handle = tokio::spawn(async move {
            Self::response_loop(recv, pending).await;
        });
        // Store handle for cleanup — best-effort since we can't block in &self.
        if let Ok(mut guard) = self.response_handle.try_lock() {
            *guard = Some(handle);
        }
    }

    /// Start the background response handler from a TransportRecvHalf.
    /// Public API for cases where the caller has a receive half.
    pub fn start_response_handler(&self, transport_recv: TransportRecvHalf) {
        self.spawn_response_handler(transport_recv);
    }

    /// Backward compat: spawn a response handler that reads from the shared
    /// transport (the one passed to `from_transport`).
    pub async fn start_response_handler_compat(&self) {
        if let Some(shared) = &self.shared_transport {
            let recv: TransportRecvHalf = Box::new(SharedTransportRecv { inner: shared.clone() });
            self.spawn_response_handler(recv);
        }
    }

    // ── Public accessors ──────────────────────────────────────────

    /// Current connection state.
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Server capabilities (populated after initialize).
    pub async fn server_capabilities(&self) -> Option<ServerCapabilities> {
        self.server_capabilities.read().await.clone()
    }

    /// Server info (populated after initialize).
    pub async fn server_info(&self) -> Option<ServerInfo> {
        self.server_info.read().await.clone()
    }

    /// Current request id counter value.
    pub fn request_id_counter(&self) -> u64 {
        self.request_id.load(Ordering::SeqCst)
    }

    // ── Lifecycle ─────────────────────────────────────────────────

    /// Disconnect and clean up.
    pub async fn disconnect(&self) -> Result<()> {
        *self.state.write().await = ConnectionState::Disconnected;
        if let Some(handle) = self.response_handle.lock().await.take() { handle.abort(); }
        let mut map = self.pending_requests.lock().await;
        for (_, tx) in map.drain() { let _ = tx.send(Err("client disconnected".to_string())); }
        Ok(())
    }

    // ── MCP-50: State report ────────────────────────────────────

    /// Return a structured status report for observability.
    pub async fn status_report(&self) -> Value {
        let state = self.state().await;
        let pending = self.pending_requests.lock().await.len();
        let tool_count = self.tool_cache.read().await.as_ref().map(|t| t.len()).unwrap_or(0);
        let info = self.server_info().await;
        let caps = self.server_capabilities().await;
        serde_json::json!({
            "state": format!("{:?}", state), "pending_requests": pending,
            "request_count": self.request_count.load(Ordering::SeqCst),
            "timeout_count": self.timeout_count.load(Ordering::SeqCst),
            "cached_tools": tool_count,
            "last_error": self.last_error.read().await.clone(),
            "server_info": info.map(|i| serde_json::json!({"name": i.name, "version": i.version})),
            "capabilities": caps.map(|c| serde_json::json!({
                "has_tools": c.tools.is_some(), "has_resources": c.resources.is_some(),
                "has_prompts": c.prompts.is_some()
            }))
        })
    }

    // ── MCP-51: Cleanup on drop ─────────────────────────────────

    /// Graceful shutdown: disconnect and abort the response handler.
    /// Safe to call multiple times; idempotent.
    pub async fn shutdown(&self) {
        let _ = self.disconnect().await;
    }

    // ── MCP-11: Single-source lifecycle helper ───────────────────

    /// Create transport, spawn client, initialize, and return Arc<McpClient>.
    /// This is the single source of truth for MCP client lifecycle (Kural 7).
    /// Initialize fail → state Failed + disconnect + Err (no partial client leak).
    pub async fn connect_initialized(
        config: TransportConfig,
        initialize_timeout: std::time::Duration,
    ) -> Result<Arc<McpClient>> {
        let client = Arc::new(McpClient::new(config).await?);
        match tokio::time::timeout(initialize_timeout, client.initialize()).await {
            Ok(Ok(_)) => Ok(client),
            Ok(Err(e)) => {
                let msg = e.to_string();
                *client.state.write().await = ConnectionState::Failed(msg.clone());
                *client.last_error.write().await = Some(msg);
                let _ = client.disconnect().await;
                Err(e)
            }
            Err(_) => {
                let msg = format!("initialize timeout after {:?}", initialize_timeout);
                *client.state.write().await = ConnectionState::Failed(msg.clone());
                *client.last_error.write().await = Some(msg.clone());
                let _ = client.disconnect().await;
                anyhow::bail!("{}", msg)
            }
        }
    }

    async fn response_loop(
        mut recv: TransportRecvHalf,
        pending: SharedPending,
    ) {
        loop {
            let response = match recv.receive().await {
                Ok(r) => r,
                Err(e) => {
                    warn!("Response handler error: {}", e);
                    // Wake all pending with error
                    let mut map = pending.lock().await;
                    for (_, tx) in map.drain() {
                        let _ = tx.send(Err(format!("transport error: {}", e)));
                    }
                    return;
                }
            };

            let mut map = pending.lock().await;
            if let Some(tx) = map.remove(&response.id) {
                let payload = match (&response.result, &response.error) {
                    (Some(r), _) => Ok(r.clone()),
                    (None, Some(e)) => Err(format!("{} (code: {})", e.message, e.code)),
                    (None, None) => Err("response has no result and no error".to_string()),
                };
                let _ = tx.send(payload);
            }
        }
    }

    pub async fn initialize(&self) -> Result<InitializeResponse> {
        *self.state.write().await = ConnectionState::Connecting;
        let request = InitializeRequest {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities { experimental: None, sampling: None },
            client_info: ClientInfo {
                name: "HudHudScript".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        let response = self.send_request("initialize", Some(serde_json::to_value(request)?)).await?;
        let init: InitializeResponse = serde_json::from_value(response)
            .context("parse initialize response")?;

        *self.server_capabilities.write().await = Some(init.capabilities.clone());
        *self.server_info.write().await = Some(init.server_info.clone());
        *self.state.write().await = ConnectionState::Connected;
        Ok(init)
    }

    pub async fn list_tools(&self, cursor: Option<String>) -> Result<ToolsListResponse> {
        let params = cursor.map(|c| serde_json::json!({"cursor": c}));
        let response = self.send_request("tools/list", params).await?;
        let tools_resp: ToolsListResponse = serde_json::from_value(response).context("parse tools/list")?;
        // MCP-42: Validate tool count and schemas.
        Self::validate_tools(&tools_resp.tools)?;
        // MCP-42: Cache tools for pre-call validation.
        *self.tool_cache.write().await = Some(tools_resp.tools.clone());
        Ok(tools_resp)
    }

    /// MCP-42: Look up a cached tool by name. Returns None if not cached.
    pub async fn get_cached_tool(&self, name: &str) -> Option<Tool> {
        let cache = self.tool_cache.read().await;
        cache.as_ref()?.iter().find(|t| t.name == name).cloned()
    }

    /// MCP-42: Validate tool definitions returned by a server.
    fn validate_tools(tools: &[Tool]) -> Result<()> {
        if tools.len() > MAX_TOOLS_PER_SERVER {
            anyhow::bail!("Server returned {} tools, max {}", tools.len(), MAX_TOOLS_PER_SERVER);
        }
        for t in tools {
            if t.name.is_empty() { anyhow::bail!("Tool has empty name"); }
            if t.name.len() > 128 { anyhow::bail!("Tool name too long: '{}'", t.name); }
            if !t.input_schema.is_null() && !t.input_schema.is_object() {
                anyhow::bail!("Tool '{}' has invalid inputSchema", t.name);
            }
        }
        Ok(())
    }

    pub async fn call_tool(&self, name: String, arguments: Option<serde_json::Value>) -> Result<ToolCallResponse> {
        let params = serde_json::json!({"name": name, "arguments": arguments.unwrap_or(serde_json::json!({}))});
        let response = self.send_request("tools/call", Some(params)).await?;
        serde_json::from_value(response).context("parse tools/call")
    }

    pub async fn list_resources(&self, cursor: Option<String>) -> Result<ResourcesListResponse> {
        let request = ResourcesListRequest { cursor };
        let response = self.send_request("resources/list", Some(serde_json::to_value(request)?)).await?;
        serde_json::from_value(response).context("parse resources/list")
    }

    pub async fn read_resource(&self, uri: String) -> Result<ResourceReadResponse> {
        let request = ResourceReadRequest { uri };
        let response = self.send_request("resources/read", Some(serde_json::to_value(request)?)).await?;
        serde_json::from_value(response).context("parse resources/read")
    }

    async fn send_request(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        self.request_count.fetch_add(1, Ordering::SeqCst);
        let id = RequestId::new_number(self.request_id.fetch_add(1, Ordering::SeqCst) as i64);
        let request = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: id.clone(), method: method.to_string(), params };
        let (tx, rx) = oneshot::channel::<PendingResult>();
        {
            let mut map = self.pending_requests.lock().await;
            if map.len() >= MAX_PENDING_REQUESTS {
                anyhow::bail!("Too many pending MCP requests ({}/{})", map.len(), MAX_PENDING_REQUESTS);
            }
            map.insert(id.clone(), tx);
        }

        // MCP-43: Enforce request size limit.
        {
            let json = serde_json::to_string(&request)?;
            if json.len() > MAX_REQUEST_SIZE {
                self.pending_requests.lock().await.remove(&id);
                anyhow::bail!("MCP request too large: {} bytes (max {})", json.len(), MAX_REQUEST_SIZE);
            }
        }

        {
            let mut send = self.transport_send.lock().await;
            send.send(request).await?;
        }

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            rx,
        )
            .await
            .map_err(|_| {
                self.timeout_count.fetch_add(1, Ordering::SeqCst);
                anyhow::anyhow!("Request timeout after {}s", DEFAULT_REQUEST_TIMEOUT_SECS)
            })?
            .context("Response channel closed")?
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(result)
    }
}

/// SharedTransportAdapter wraps SharedTransport to implement TransportSend.
struct SharedTransportAdapter {
    inner: SharedTransport,
}

#[async_trait::async_trait]
impl crate::transport::TransportSend for SharedTransportAdapter {
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()> {
        self.inner.lock().await.send(request).await
    }
}

/// SharedTransportRecv wraps SharedTransport to implement TransportRecv.
struct SharedTransportRecv {
    inner: SharedTransport,
}

#[async_trait::async_trait]
impl crate::transport::TransportRecv for SharedTransportRecv {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        self.inner.lock().await.receive().await
    }
}

impl McpClient {
    /// Create from a pre-built transport (for mock/testing).
    /// Uses shared-mutex model for backward compatibility.
    /// Call `start_response_handler_compat().await` to start the
    /// response handler before sending requests.
    pub fn from_transport(transport: Box<dyn crate::transport::Transport>) -> Self {
        let shared: SharedTransport = Arc::new(tokio::sync::Mutex::new(transport));
        Self {
            transport_send: tokio::sync::Mutex::new(Box::new(SharedTransportAdapter { inner: shared.clone() })),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            request_id: Arc::new(AtomicU64::new(1)),
            server_capabilities: Arc::new(RwLock::new(None)),
            server_info: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            response_handle: Mutex::new(None),
            shared_transport: Some(shared.clone()),
            tool_cache: RwLock::new(None),
            request_count: AtomicU64::new(0),
            timeout_count: AtomicU64::new(0),
            last_error: RwLock::new(None),
        }
    }
}
