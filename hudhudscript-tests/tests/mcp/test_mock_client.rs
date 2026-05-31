//! Tests for McpClient using a mock transport to exercise initialize(),
//! send_request(), start_response_handler(), and the full request/response
//! pipeline without any real network or subprocess connections.
//!
//! Because `make_transport_pair` shares a single Mutex for both send and
//! receive, the mock transport uses a tokio channel so that `receive()` yields
//! the executor while waiting, allowing `send()` to acquire the lock.

use anyhow::Result;
use hudhudscript_mcp::{
    transport::{Transport, TransportRecv, TransportSend},
    ConnectionState, JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpClient, RequestId,
};
use serde_json::json;
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Channel-based Mock Transport
// ---------------------------------------------------------------------------

/// A mock transport backed by a tokio mpsc channel.
///
/// Responses are fed through the channel so that `receive()` properly yields
/// to the executor, allowing `send()` to interleave on the shared mutex.
struct ChannelMockTransport {
    rx: mpsc::UnboundedReceiver<JsonRpcResponse>,
}

impl ChannelMockTransport {
    /// Create a new mock transport and return (transport, sender).
    /// Push responses into the sender to feed them to the client.
    fn new() -> (Self, mpsc::UnboundedSender<JsonRpcResponse>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { rx }, tx)
    }
}

#[async_trait::async_trait]
impl TransportSend for ChannelMockTransport {
    async fn send(&mut self, _request: JsonRpcRequest) -> Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl TransportRecv for ChannelMockTransport {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("channel closed"))
    }
}

#[async_trait::async_trait]
impl Transport for ChannelMockTransport {
    async fn close(&mut self) -> Result<()> {
        self.rx.close();
        Ok(())
    }
}

/// Build a successful JSON-RPC response for a given request id and result value.
fn ok_response(id: RequestId, result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

/// Build an error JSON-RPC response.
fn err_response(id: RequestId, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
            data: None,
        }),
    }
}

fn init_ok() -> serde_json::Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "serverInfo": { "name": "mock", "version": "1.0" }
    })
}

/// Helper: create client + sender, start response handler, send init response.
async fn setup_connected_client() -> (McpClient, mpsc::UnboundedSender<JsonRpcResponse>) {
    let (transport, tx) = ChannelMockTransport::new();
    let client = McpClient::from_transport(Box::new(transport));
    client.start_response_handler().await;

    // Feed the init response before calling initialize
    tx.send(ok_response(RequestId::new_number(1), init_ok()))
        .unwrap();
    let _ = client.initialize().await.unwrap();
    (client, tx)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Test that initialize() transitions state and stores server info.
#[tokio::test]
async fn test_initialize_success_via_mock() {
    let (transport, tx) = ChannelMockTransport::new();
    let client = McpClient::from_transport(Box::new(transport));
    client.start_response_handler().await;

    let init_result = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": { "listChanged": true }
        },
        "serverInfo": {
            "name": "mock-server",
            "version": "0.1.0"
        }
    });
    tx.send(ok_response(RequestId::new_number(1), init_result))
        .unwrap();

    let resp = client
        .initialize()
        .await
        .expect("initialize should succeed");
    assert_eq!(resp.server_info.name, "mock-server");
    assert_eq!(resp.server_info.version, "0.1.0");
    assert_eq!(resp.protocol_version, "2024-11-05");
    assert_eq!(client.state().await, ConnectionState::Connected);

    let info = client
        .server_info()
        .await
        .expect("server_info should be Some");
    assert_eq!(info.name, "mock-server");
    let caps = client
        .server_capabilities()
        .await
        .expect("caps should be Some");
    assert!(caps.tools.is_some());
}

/// Test list_tools() after initialization.
#[tokio::test]
async fn test_list_tools_via_mock() {
    let (client, tx) = setup_connected_client().await;

    tx.send(ok_response(
        RequestId::new_number(2),
        json!({
            "tools": [
                { "name": "read_file", "description": "Read a file", "inputSchema": { "type": "object" } },
                { "name": "write_file", "inputSchema": { "type": "object" } }
            ]
        }),
    ))
    .unwrap();

    let resp = client.list_tools(None).await.unwrap();
    assert_eq!(resp.tools.len(), 2);
    assert_eq!(resp.tools[0].name, "read_file");
    assert!(resp.tools[0].description.is_some());
    assert!(resp.tools[1].description.is_none());
    assert!(resp.next_cursor.is_none());
}

/// Test call_tool() after initialization.
#[tokio::test]
async fn test_call_tool_via_mock() {
    let (client, tx) = setup_connected_client().await;

    tx.send(ok_response(
        RequestId::new_number(2),
        json!({
            "content": [{ "type": "text", "text": "Hello from tool" }]
        }),
    ))
    .unwrap();

    let resp = client
        .call_tool("greet".to_string(), Some(json!({"name": "world"})))
        .await
        .unwrap();
    assert_eq!(resp.content.len(), 1);
    assert!(resp.is_error.is_none());
}

/// Test list_resources() via mock.
#[tokio::test]
async fn test_list_resources_via_mock() {
    let (client, tx) = setup_connected_client().await;

    tx.send(ok_response(
        RequestId::new_number(2),
        json!({
            "resources": [{
                "uri": "file:///tmp/test.txt",
                "name": "test.txt",
                "description": "A test file",
                "mimeType": "text/plain"
            }]
        }),
    ))
    .unwrap();

    let res = client.list_resources(None).await.unwrap();
    assert_eq!(res.resources.len(), 1);
    assert_eq!(res.resources[0].uri, "file:///tmp/test.txt");
    assert_eq!(res.resources[0].description.as_deref(), Some("A test file"));
    assert_eq!(res.resources[0].mime_type.as_deref(), Some("text/plain"));
}

/// Test read_resource() via mock.
#[tokio::test]
async fn test_read_resource_via_mock() {
    let (client, tx) = setup_connected_client().await;

    tx.send(ok_response(
        RequestId::new_number(2),
        json!({
            "contents": [{
                "uri": "file:///tmp/test.txt",
                "mimeType": "text/plain",
                "text": "file contents here"
            }]
        }),
    ))
    .unwrap();

    let res = client
        .read_resource("file:///tmp/test.txt".to_string())
        .await
        .unwrap();
    assert_eq!(res.contents.len(), 1);
    assert_eq!(res.contents[0].text.as_deref(), Some("file contents here"));
}

/// Test that an RPC error response propagates as Err.
#[tokio::test]
async fn test_rpc_error_propagation() {
    let (transport, tx) = ChannelMockTransport::new();
    let client = McpClient::from_transport(Box::new(transport));
    client.start_response_handler().await;

    tx.send(err_response(
        RequestId::new_number(1),
        -32601,
        "Method not found",
    ))
    .unwrap();

    let result = client.initialize().await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Method not found") || err_msg.contains("-32601"),
        "Got: {}",
        err_msg
    );
}

/// Test response with neither result nor error.
#[tokio::test]
async fn test_response_with_no_result_no_error() {
    let (transport, tx) = ChannelMockTransport::new();
    let client = McpClient::from_transport(Box::new(transport));
    client.start_response_handler().await;

    tx.send(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(1),
        result: None,
        error: None,
    })
    .unwrap();

    let result = client.initialize().await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("no result"), "Got: {}", err_msg);
}

/// Test request IDs increment across multiple requests.
#[tokio::test]
async fn test_request_id_increments_across_requests() {
    let (client, tx) = setup_connected_client().await;

    tx.send(ok_response(
        RequestId::new_number(2),
        json!({ "tools": [] }),
    ))
    .unwrap();
    tx.send(ok_response(
        RequestId::new_number(3),
        json!({ "resources": [] }),
    ))
    .unwrap();

    let _ = client.list_tools(None).await.unwrap();
    let _ = client.list_resources(None).await.unwrap();

    let counter = client.request_id.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(
        counter, 4,
        "After 3 requests (init + 2), counter should be 4"
    );
}

/// Test disconnect after connected via mock.
/// We drop the sender to unblock the response handler before calling disconnect.
#[tokio::test]
async fn test_disconnect_via_mock() {
    let (client, tx) = setup_connected_client().await;
    assert_eq!(client.state().await, ConnectionState::Connected);

    // Drop sender so the response handler's receive() returns an error and
    // exits the loop, releasing the shared mutex for disconnect().
    drop(tx);
    // Give the handler task a moment to notice the closed channel.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    client
        .disconnect()
        .await
        .expect("disconnect should succeed");
    assert_eq!(client.state().await, ConnectionState::Disconnected);

    let result = client.list_tools(None).await;
    assert!(result.is_err());
}

/// Test list_tools with pagination cursor.
#[tokio::test]
async fn test_list_tools_with_cursor_via_mock() {
    let (client, tx) = setup_connected_client().await;

    tx.send(ok_response(
        RequestId::new_number(2),
        json!({
            "tools": [{ "name": "tool_a", "inputSchema": {} }],
            "nextCursor": "page2"
        }),
    ))
    .unwrap();

    let page1 = client.list_tools(None).await.unwrap();
    assert_eq!(page1.tools[0].name, "tool_a");
    assert_eq!(page1.next_cursor.as_deref(), Some("page2"));

    tx.send(ok_response(
        RequestId::new_number(3),
        json!({ "tools": [{ "name": "tool_b", "inputSchema": {} }] }),
    ))
    .unwrap();

    let page2 = client.list_tools(Some("page2".to_string())).await.unwrap();
    assert_eq!(page2.tools[0].name, "tool_b");
    assert!(page2.next_cursor.is_none());
}

/// Test from_transport creates client in Disconnected state with no sandbox.
#[tokio::test]
async fn test_from_transport_initial_state() {
    let (transport, _tx) = ChannelMockTransport::new();
    let client = McpClient::from_transport(Box::new(transport));

    assert_eq!(client.state().await, ConnectionState::Disconnected);
    assert!(client.sandbox.is_none());
    assert!(client.server_info().await.is_none());
    assert!(client.server_capabilities().await.is_none());
    let id = client.request_id.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(id, 1);
}
