//! Tests for StdioTransport send/receive using `cat` as a simple echo server.
//!
//! `cat` reads from stdin and writes to stdout, making it a perfect stand-in
//! for testing the JSON line protocol without a real MCP server.

#[allow(unused_imports)]
use hudhudscript_mcp::{
    transport::{Transport, TransportConfig, TransportRecv, TransportSend},
    JsonRpcRequest, RequestId,
};

/// Test that StdioTransport can send a request and receive it back via `cat`.
#[tokio::test]
async fn test_stdio_send_receive_via_cat() {
    let config = TransportConfig::stdio("cat", vec![]);
    let mut transport = config.create_transport().await.expect("cat should spawn");

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::new_number(42),
        method: "test/echo".to_string(),
        params: None,
    };

    // Send the request -- cat will echo it back as a line.
    transport.send(request).await.expect("send should succeed");

    // Receive the echoed line -- it will be parsed as a JsonRpcResponse,
    // but since cat echoes the request JSON verbatim (which has `method`
    // and `id` but not `result`/`error`), serde will fill missing fields
    // with None due to skip_serializing_if / default behavior.
    // Actually, JsonRpcResponse requires `jsonrpc` and `id` which the
    // request JSON has, so it should deserialize (with result=None, error=None).
    let response = transport.receive().await.expect("receive should succeed");
    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, RequestId::new_number(42));

    // Clean up
    transport.close().await.expect("close should succeed");
}

/// Test that StdioTransport close kills the process.
#[tokio::test]
async fn test_stdio_close() {
    let config = TransportConfig::stdio("cat", vec![]);
    let mut transport = config.create_transport().await.unwrap();
    transport.close().await.expect("close should succeed");
    // After close, receive should fail (EOF or process killed).
    let result = transport.receive().await;
    assert!(result.is_err());
}

/// Test that StdioTransport handles multiple send/receive cycles.
#[tokio::test]
async fn test_stdio_multiple_exchanges() {
    let config = TransportConfig::stdio("cat", vec![]);
    let mut transport = config.create_transport().await.unwrap();

    for i in 1..=3 {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::new_number(i),
            method: format!("test/method_{}", i),
            params: None,
        };
        transport.send(request).await.unwrap();
        let resp = transport.receive().await.unwrap();
        assert_eq!(resp.id, RequestId::new_number(i));
    }

    transport.close().await.unwrap();
}

/// Test that receive fails on EOF when process exits.
#[tokio::test]
async fn test_stdio_receive_eof() {
    // `true` exits immediately with no output
    let config = TransportConfig::stdio("true", vec![]);
    let mut transport = config.create_transport().await.unwrap();
    let result = transport.receive().await;
    assert!(result.is_err(), "receive on closed process should fail");
}
