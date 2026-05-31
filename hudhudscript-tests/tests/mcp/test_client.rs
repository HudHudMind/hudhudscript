use hudhudscript_mcp::{
    make_transport_pair, ConnectionState, McpClient, ServerCapabilities, ServerInfo,
    ToolsCapability, TransportConfig,
};
use std::sync::Arc;

#[tokio::test]
async fn test_client_creation() {
    let config = TransportConfig::stdio("echo", vec![]);
    let result = McpClient::new(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_connection_state() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    assert_eq!(client.state().await, ConnectionState::Disconnected);
}

// ---- ConnectionState equality ----

#[test]
fn test_connection_state_equality() {
    assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
    assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
    assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
    assert_eq!(ConnectionState::Failed, ConnectionState::Failed);
    assert_ne!(ConnectionState::Disconnected, ConnectionState::Connected);
    assert_ne!(ConnectionState::Connecting, ConnectionState::Failed);
}

// ---- ConnectionState Debug ----

#[test]
fn test_connection_state_debug() {
    assert_eq!(
        format!("{:?}", ConnectionState::Disconnected),
        "Disconnected"
    );
    assert_eq!(format!("{:?}", ConnectionState::Connecting), "Connecting");
    assert_eq!(format!("{:?}", ConnectionState::Connected), "Connected");
    assert_eq!(format!("{:?}", ConnectionState::Failed), "Failed");
}

// ---- ConnectionState Clone + Copy ----

#[test]
fn test_connection_state_clone_copy() {
    let state = ConnectionState::Connected;
    let cloned = state.clone();
    let copied = state; // Copy
    assert_eq!(state, cloned);
    assert_eq!(state, copied);
}

// ---- server_capabilities and server_info initial state ----

#[tokio::test]
async fn test_initial_server_capabilities_none() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    assert!(client.server_capabilities().await.is_none());
}

#[tokio::test]
async fn test_initial_server_info_none() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    assert!(client.server_info().await.is_none());
}

// ---- ensure_connected error paths ----

#[tokio::test]
async fn test_ensure_connected_when_disconnected() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    // Client starts disconnected, so list_tools should fail
    let result = client.list_tools(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("not connected"));
}

// ---- ensure_connected: Connecting state ----

#[tokio::test]
async fn test_ensure_connected_when_connecting() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    // Force state to Connecting
    *client.state.write().await = ConnectionState::Connecting;
    let result = client.list_tools(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("connecting"));
}

// ---- ensure_connected: Failed state ----

#[tokio::test]
async fn test_ensure_connected_when_failed() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    // Force state to Failed
    *client.state.write().await = ConnectionState::Failed;
    let result = client.list_tools(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("failed"));
}

// ---- check_tool_execution always allows ----

#[tokio::test]
async fn test_check_tool_execution_no_sandbox() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    // No sandbox, should always succeed
    assert!(client.check_tool_execution("any_tool").is_ok());
}

// ---- request_id counter starts at 1 ----

#[tokio::test]
async fn test_request_id_starts_at_one() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    let id = client.request_id.load(std::sync::atomic::Ordering::SeqCst);
    assert_eq!(id, 1);
}

// ---- disconnect sets state back to Disconnected ----

#[tokio::test]
async fn test_disconnect_changes_state() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    // Force state to Connected
    *client.state.write().await = ConnectionState::Connected;
    assert_eq!(client.state().await, ConnectionState::Connected);
    // Disconnect (this will kill the 'echo' process)
    let _ = client.disconnect().await;
    assert_eq!(client.state().await, ConnectionState::Disconnected);
}

// ---- list_resources fails when disconnected ----

#[tokio::test]
async fn test_list_resources_when_disconnected() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    let result = client.list_resources(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("not connected"));
}

// ---- read_resource fails when disconnected ----

#[tokio::test]
async fn test_read_resource_when_disconnected() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    let result = client.read_resource("file:///test.txt".to_string()).await;
    assert!(result.is_err());
}

// ---- call_tool fails when disconnected ----

#[tokio::test]
async fn test_call_tool_when_disconnected() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    let result = client.call_tool("my_tool".to_string(), None).await;
    assert!(result.is_err());
}

// ---- pending_requests starts empty ----

#[tokio::test]
async fn test_pending_requests_starts_empty() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    let pending = client.pending_requests.read().await;
    assert!(pending.is_empty());
}

// ---- Multiple ensure_connected calls with same state ----

#[tokio::test]
async fn test_ensure_connected_when_connected() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Connected;
    // Should succeed
    let result = client.ensure_connected().await;
    assert!(result.is_ok());
}

// ---- request_id increments correctly ----

#[tokio::test]
async fn test_request_id_increments() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    let id1 = client
        .request_id
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let id2 = client
        .request_id
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

// ---- ConnectionState all four states are distinct ----

#[test]
fn test_connection_state_all_distinct() {
    let states = [
        ConnectionState::Disconnected,
        ConnectionState::Connecting,
        ConnectionState::Connected,
        ConnectionState::Failed,
    ];
    for i in 0..states.len() {
        for j in 0..states.len() {
            if i == j {
                assert_eq!(states[i], states[j]);
            } else {
                assert_ne!(states[i], states[j]);
            }
        }
    }
}

// ---- call_tool: Connecting state ----

#[tokio::test]
async fn test_call_tool_when_connecting() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Connecting;
    let result = client.call_tool("tool".to_string(), None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("connecting"));
}

// ---- call_tool: Failed state ----

#[tokio::test]
async fn test_call_tool_when_failed() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Failed;
    let result = client.call_tool("tool".to_string(), None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("failed"));
}

// ---- list_resources: Connecting state ----

#[tokio::test]
async fn test_list_resources_when_connecting() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Connecting;
    let result = client.list_resources(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("connecting"));
}

// ---- list_resources: Failed state ----

#[tokio::test]
async fn test_list_resources_when_failed() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Failed;
    let result = client.list_resources(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("failed"));
}

// ---- read_resource: Connecting state ----

#[tokio::test]
async fn test_read_resource_when_connecting() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Connecting;
    let result = client.read_resource("file:///x".to_string()).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("connecting"));
}

// ---- read_resource: Failed state ----

#[tokio::test]
async fn test_read_resource_when_failed() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Failed;
    let result = client.read_resource("file:///x".to_string()).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("failed"));
}

// ---- list_tools: Connecting state ----

#[tokio::test]
async fn test_list_tools_when_connecting() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Connecting;
    let result = client.list_tools(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("connecting"));
}

// ---- list_tools: Failed state ----

#[tokio::test]
async fn test_list_tools_when_failed() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    *client.state.write().await = ConnectionState::Failed;
    let result = client.list_tools(None).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("failed"));
}

// ---- check_tool_execution with sandbox (current impl allows all) ----

#[tokio::test]
async fn test_check_tool_execution_with_sandbox() {
    let config = TransportConfig::stdio("echo", vec![]);
    let sandbox_config = hudhudscript_sandbox::SandboxConfig::default_permissive();
    let client = McpClient::with_sandbox(config, sandbox_config)
        .await
        .unwrap();
    // Even with sandbox, current implementation allows all tools
    assert!(client.check_tool_execution("any_tool").is_ok());
    assert!(client.check_tool_execution("dangerous_tool").is_ok());
}

// ---- make_transport_pair produces two arcs pointing to same data ----

#[tokio::test]
async fn test_make_transport_pair_shared() {
    let config = TransportConfig::stdio("echo", vec![]);
    let transport = config.create_transport().await.unwrap();
    let (send, recv) = make_transport_pair(transport);
    // Both Arc's should have strong count of 2 (they share the same underlying)
    assert_eq!(Arc::strong_count(&send), 2);
    assert_eq!(Arc::strong_count(&recv), 2);
}

// ---- client initial sandbox is None ----

#[tokio::test]
async fn test_client_no_sandbox_by_default() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    assert!(client.sandbox.is_none());
}

// ---- client with_sandbox has Some sandbox ----

#[tokio::test]
async fn test_client_with_sandbox_has_sandbox() {
    let config = TransportConfig::stdio("echo", vec![]);
    let sandbox_config = hudhudscript_sandbox::SandboxConfig::default_permissive();
    let client = McpClient::with_sandbox(config, sandbox_config)
        .await
        .unwrap();
    assert!(client.sandbox.is_some());
}

// ---- state transitions: Disconnected -> Connected -> Disconnected ----

#[tokio::test]
async fn test_state_transition_full_cycle() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    assert_eq!(client.state().await, ConnectionState::Disconnected);

    *client.state.write().await = ConnectionState::Connecting;
    assert_eq!(client.state().await, ConnectionState::Connecting);

    *client.state.write().await = ConnectionState::Connected;
    assert_eq!(client.state().await, ConnectionState::Connected);

    let _ = client.disconnect().await;
    assert_eq!(client.state().await, ConnectionState::Disconnected);
}

// ---- request_id counter is shared across the client ----

#[tokio::test]
async fn test_request_id_atomic_shared() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    // Increment several times
    for expected in 1..=5 {
        let val = client
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(val, expected);
    }
    assert_eq!(
        client.request_id.load(std::sync::atomic::Ordering::SeqCst),
        6
    );
}

// ---- server_capabilities and server_info can be written and read ----

#[tokio::test]
async fn test_server_capabilities_write_read() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    assert!(client.server_capabilities().await.is_none());

    let caps = ServerCapabilities {
        experimental: None,
        logging: None,
        prompts: None,
        resources: None,
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
    };
    *client.server_capabilities.write().await = Some(caps);
    let read = client.server_capabilities().await.unwrap();
    assert!(read.tools.is_some());
}

#[tokio::test]
async fn test_server_info_write_read() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();
    assert!(client.server_info().await.is_none());

    let info = ServerInfo {
        name: "test-srv".to_string(),
        version: "1.2.3".to_string(),
    };
    *client.server_info.write().await = Some(info);
    let read = client.server_info().await.unwrap();
    assert_eq!(read.name, "test-srv");
    assert_eq!(read.version, "1.2.3");
}
