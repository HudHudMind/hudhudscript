//! MCP integration tests

use hudhudscript_mcp::{McpClient, TransportConfig, TransportType};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_client_creation() {
    let config = TransportConfig::stdio("echo", vec![]);
    let result = McpClient::new(config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_client_state_transitions() {
    use hudhudscript_mcp::ConnectionState;

    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();

    // Should start disconnected
    assert_eq!(client.state().await, ConnectionState::Disconnected);
}

#[tokio::test]
async fn test_multiple_clients() {
    let config1 = TransportConfig::stdio("echo", vec![]);
    let config2 = TransportConfig::stdio("cat", vec![]);

    let client1 = McpClient::new(config1).await;
    let client2 = McpClient::new(config2).await;

    assert!(client1.is_ok());
    assert!(client2.is_ok());
}

#[cfg(feature = "integration-tests")]
#[tokio::test]
async fn test_client_with_timeout() {
    let config = TransportConfig::stdio("sleep", vec!["10".to_string()]);

    // Try to create client with timeout
    // This may or may not timeout depending on system
    let result = timeout(Duration::from_millis(100), McpClient::new(config)).await;

    // Either timeout or error is acceptable
    assert!(result.is_err() || result.unwrap().is_err());
}

#[tokio::test]
async fn test_transport_config_stdio() {
    let config = TransportConfig::stdio("test", vec!["arg1".to_string(), "arg2".to_string()]);

    // Config should be created successfully
    assert_eq!(config.transport_type, TransportType::Stdio);
    assert_eq!(config.command, Some("test".to_string()));
    assert_eq!(config.args.len(), 2);
}

#[tokio::test]
async fn test_transport_config_sse() {
    let config = TransportConfig::sse("http://localhost:8080");

    // Config should be created successfully
    assert_eq!(config.transport_type, TransportType::Sse);
    assert_eq!(config.url, Some("http://localhost:8080".to_string()));
}

#[tokio::test]
async fn test_client_disconnect() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();

    // Disconnect should not fail
    let result = client.disconnect().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_client_creation() {
    use tokio::task::JoinSet;

    let mut set = JoinSet::new();

    for i in 0..5 {
        set.spawn(async move {
            let config = TransportConfig::stdio("echo", vec![format!("client_{}", i)]);
            McpClient::new(config).await
        });
    }

    let mut success_count = 0;
    while let Some(result) = set.join_next().await {
        if let Ok(Ok(_)) = result {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 5);
}

#[tokio::test]
async fn test_client_capabilities() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();

    // Should have no capabilities before initialization
    assert!(client.server_capabilities().await.is_none());
}

#[tokio::test]
async fn test_client_server_info() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();

    // Should have no server info before initialization
    assert!(client.server_info().await.is_none());
}

#[tokio::test]
async fn test_error_handling_invalid_command() {
    let config = TransportConfig::stdio("nonexistent_command_12345", vec![]);
    let result = McpClient::new(config).await;

    // Should fail with invalid command
    assert!(result.is_err());
}

#[tokio::test]
async fn test_client_with_sandbox() {
    use hudhudscript_sandbox::SandboxConfig;

    let config = TransportConfig::stdio("echo", vec![]);
    let sandbox_config = SandboxConfig::default_permissive();

    let result = McpClient::with_sandbox(config, sandbox_config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_client_with_restrictive_sandbox() {
    use hudhudscript_sandbox::SandboxConfig;

    let config = TransportConfig::stdio("echo", vec![]);
    let sandbox_config = SandboxConfig::default_restrictive();

    let result = McpClient::with_sandbox(config, sandbox_config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_disconnect_calls() {
    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();

    // First disconnect
    let result1 = client.disconnect().await;
    assert!(result1.is_ok());

    // Second disconnect should also succeed (idempotent)
    let result2 = client.disconnect().await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_client_state_after_disconnect() {
    use hudhudscript_mcp::ConnectionState;

    let config = TransportConfig::stdio("echo", vec![]);
    let client = McpClient::new(config).await.unwrap();

    client.disconnect().await.unwrap();

    // Should be disconnected
    assert_eq!(client.state().await, ConnectionState::Disconnected);
}
