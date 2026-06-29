//! Real stdio MCP fixture tests — uses `mcp_fixture_server` binary.

use hudhudscript_mcp::client::McpClient;
use hudhudscript_mcp::transport::TransportConfig;
use std::time::Duration;
use tokio::time::timeout;

fn fixture_config() -> TransportConfig {
    let bin = env!("CARGO_BIN_EXE_mcp_fixture_server").to_string();
    TransportConfig::stdio(bin, vec![])
}

async fn fixture_client() -> McpClient {
    McpClient::new(fixture_config())
        .await
        .expect("spawn fixture")
}

#[tokio::test]
async fn test_real_stdio_fixture_initialize() {
    let client = fixture_client().await;
    let result = timeout(Duration::from_secs(5), client.initialize())
        .await
        .expect("timeout")
        .expect("initialize failed");
    assert_eq!(result.protocol_version, "2024-11-05");
    assert_eq!(result.server_info.name, "hudhud-mcp-fixture");
    assert!(result.capabilities.tools.is_some());
}

#[tokio::test]
async fn test_real_stdio_fixture_list_tools() {
    let client = fixture_client().await;
    client.initialize().await.expect("initialize");
    let result = timeout(Duration::from_secs(5), client.list_tools(None))
        .await
        .expect("timeout")
        .expect("list_tools");
    assert_eq!(result.tools.len(), 2);
    let names: Vec<&str> = result.tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"echo"));
    assert!(names.contains(&"add"));
}

#[tokio::test]
async fn test_real_stdio_fixture_call_tool_echo() {
    let client = fixture_client().await;
    client.initialize().await.expect("initialize");
    let args = serde_json::json!({"text": "hello fixture"});
    let result = timeout(
        Duration::from_secs(5),
        client.call_tool("echo".to_string(), Some(args)),
    )
    .await
    .expect("timeout")
    .expect("call_tool echo");
    assert!(!result.content.is_empty());
    if let hudhudscript_mcp::protocol::Content::Text { ref text } = result.content[0] {
        assert_eq!(text, "hello fixture");
    } else {
        panic!("expected text content");
    }
}

#[tokio::test]
async fn test_real_stdio_fixture_call_tool_add() {
    let client = fixture_client().await;
    client.initialize().await.expect("initialize");
    let args = serde_json::json!({"a": 7, "b": 3});
    let result = timeout(
        Duration::from_secs(5),
        client.call_tool("add".to_string(), Some(args)),
    )
    .await
    .expect("timeout")
    .expect("call_tool add");
    assert!(!result.content.is_empty());
    if let hudhudscript_mcp::protocol::Content::Text { ref text } = result.content[0] {
        assert_eq!(text, "10");
    } else {
        panic!("expected text content");
    }
}

#[tokio::test]
async fn test_real_stdio_fixture_unknown_method_error() {
    let client = fixture_client().await;
    client.initialize().await.expect("initialize");
    let result = timeout(
        Duration::from_secs(5),
        client.call_tool("nonexistent".to_string(), Some(serde_json::json!({}))),
    )
    .await
    .expect("timeout");
    assert!(result.is_err());
}
