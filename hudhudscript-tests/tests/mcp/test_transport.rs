use hudhudscript_mcp::{
    TransportConfig, TransportType, INITIAL_RECONNECT_DELAY_MS, MAX_RECONNECT_ATTEMPTS,
};

#[test]
fn test_transport_config_stdio() {
    let config = TransportConfig::stdio("uvx", vec!["mcp-server-sqlite".to_string()]);
    assert_eq!(config.transport_type, TransportType::Stdio);
    assert_eq!(config.command.unwrap(), "uvx");
    assert_eq!(config.args.len(), 1);
}

#[test]
fn test_transport_config_sse() {
    let config = TransportConfig::sse("http://localhost:3000/sse");
    assert_eq!(config.transport_type, TransportType::Sse);
    assert_eq!(config.url.unwrap(), "http://localhost:3000/sse");
}

// ---- TransportType equality ----

#[test]
fn test_transport_type_equality() {
    assert_eq!(TransportType::Stdio, TransportType::Stdio);
    assert_eq!(TransportType::Sse, TransportType::Sse);
    assert_ne!(TransportType::Stdio, TransportType::Sse);
}

// ---- TransportConfig construction details ----

#[test]
fn test_transport_config_stdio_fields() {
    let config = TransportConfig::stdio("my-cmd", vec!["arg1".to_string(), "arg2".to_string()]);
    assert_eq!(config.transport_type, TransportType::Stdio);
    assert_eq!(config.command.as_deref().unwrap(), "my-cmd");
    assert_eq!(config.args, vec!["arg1".to_string(), "arg2".to_string()]);
    assert!(config.url.is_none());
}

#[test]
fn test_transport_config_sse_fields() {
    let config = TransportConfig::sse("https://example.com/events");
    assert_eq!(config.transport_type, TransportType::Sse);
    assert!(config.command.is_none());
    assert!(config.args.is_empty());
    assert_eq!(config.url.as_deref().unwrap(), "https://example.com/events");
}

// ---- TransportConfig stdio with no args ----

#[test]
fn test_transport_config_stdio_no_args() {
    let config = TransportConfig::stdio("echo", vec![]);
    assert_eq!(config.command.as_deref().unwrap(), "echo");
    assert!(config.args.is_empty());
}

// ---- TransportType Debug ----

#[test]
fn test_transport_type_debug() {
    let stdio_debug = format!("{:?}", TransportType::Stdio);
    assert_eq!(stdio_debug, "Stdio");
    let sse_debug = format!("{:?}", TransportType::Sse);
    assert_eq!(sse_debug, "Sse");
}

// ---- TransportConfig Debug ----

#[test]
fn test_transport_config_debug() {
    let config = TransportConfig::stdio("test", vec![]);
    let debug = format!("{:?}", config);
    assert!(debug.contains("Stdio"));
    assert!(debug.contains("test"));
}

// ---- TransportType Clone ----

#[test]
fn test_transport_type_clone() {
    let original = TransportType::Sse;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ---- TransportConfig clone ----

#[test]
fn test_transport_config_clone() {
    let config = TransportConfig::stdio("cmd", vec!["arg1".to_string()]);
    let cloned = config.clone();
    assert_eq!(cloned.transport_type, TransportType::Stdio);
    assert_eq!(cloned.command.as_deref().unwrap(), "cmd");
    assert_eq!(cloned.args, vec!["arg1".to_string()]);
}

// ---- TransportConfig SSE with Into<String> ----

#[test]
fn test_transport_config_sse_string_type() {
    let url = String::from("http://localhost:8080/events");
    let config = TransportConfig::sse(url.clone());
    assert_eq!(config.url.unwrap(), url);
}

// ---- create_transport: stdio missing command error ----

#[tokio::test]
async fn test_create_transport_stdio_missing_command() {
    let config = TransportConfig {
        transport_type: TransportType::Stdio,
        command: None,
        args: vec![],
        url: None,
    };
    let result = config.create_transport().await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.err().unwrap());
    assert!(err_msg.contains("Command required"));
}

// ---- create_transport: SSE missing URL error ----

#[tokio::test]
async fn test_create_transport_sse_missing_url() {
    let config = TransportConfig {
        transport_type: TransportType::Sse,
        command: None,
        args: vec![],
        url: None,
    };
    let result = config.create_transport().await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.err().unwrap());
    assert!(err_msg.contains("URL required"));
}

// ---- create_transport: stdio with valid command succeeds ----

#[tokio::test]
async fn test_create_transport_stdio_valid() {
    let config = TransportConfig::stdio("echo", vec![]);
    let result = config.create_transport().await;
    assert!(result.is_ok());
}

// ---- MAX_RECONNECT_ATTEMPTS & INITIAL_RECONNECT_DELAY_MS ----

#[test]
fn test_reconnect_constants() {
    assert_eq!(MAX_RECONNECT_ATTEMPTS, 5);
    assert_eq!(INITIAL_RECONNECT_DELAY_MS, 500);
}

// ---- TransportConfig: stdio with Into<String> ----

#[test]
fn test_transport_config_stdio_string_type() {
    let cmd = String::from("my-binary");
    let config = TransportConfig::stdio(cmd.clone(), vec![]);
    assert_eq!(config.command.unwrap(), cmd);
}

// ---- TransportConfig: stdio with multiple args ----

#[test]
fn test_transport_config_stdio_multiple_args() {
    let config = TransportConfig::stdio(
        "node",
        vec![
            "--experimental".to_string(),
            "server.js".to_string(),
            "--port".to_string(),
            "3000".to_string(),
        ],
    );
    assert_eq!(config.args.len(), 4);
    assert_eq!(config.args[0], "--experimental");
    assert_eq!(config.args[3], "3000");
}

// ---- create_transport: stdio with invalid command produces an error on send/recv, not on creation ----

#[tokio::test]
async fn test_create_transport_stdio_nonexistent_command() {
    let config = TransportConfig::stdio("definitely_not_a_real_binary_xyz_123", vec![]);
    let result = config.create_transport().await;
    // The command might fail to spawn, producing an error
    assert!(result.is_err());
}

// ---- TransportType: all variants have distinct debug strings ----

#[test]
fn test_transport_type_all_debug_distinct() {
    let stdio = format!("{:?}", TransportType::Stdio);
    let sse = format!("{:?}", TransportType::Sse);
    assert_ne!(stdio, sse);
}

// ---- TransportConfig: manual construction with all None ----

#[test]
fn test_transport_config_manual_all_none() {
    let config = TransportConfig {
        transport_type: TransportType::Stdio,
        command: None,
        args: vec![],
        url: None,
    };
    assert!(config.command.is_none());
    assert!(config.url.is_none());
    assert!(config.args.is_empty());
}
