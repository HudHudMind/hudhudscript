//! Tests for McpClient struct — public API surface.
//! Does NOT test actual MCP interaction (see test_mock_client, test_real_stdio_fixture).

use hudhudscript_mcp::{
    ConnectionState, McpClient, ServerCapabilities, ServerInfo, ToolsCapability,
};
// use hudhudscript_mcp::TransportConfig; // used indirectly via McpClient::new

// ── ConnectionState equality ──────────────────────────────────────────

#[test]
fn test_connection_state_equality() {
    assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
    assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
    assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
    assert_eq!(
        ConnectionState::Failed("a".into()),
        ConnectionState::Failed("a".into())
    );
    assert_ne!(ConnectionState::Disconnected, ConnectionState::Connected);
    assert_ne!(
        ConnectionState::Connecting,
        ConnectionState::Failed("x".into())
    );
}

// ── ConnectionState Debug ─────────────────────────────────────────────

#[test]
fn test_connection_state_debug() {
    assert_eq!(
        format!("{:?}", ConnectionState::Disconnected),
        "Disconnected"
    );
    assert_eq!(format!("{:?}", ConnectionState::Connecting), "Connecting");
    assert_eq!(format!("{:?}", ConnectionState::Connected), "Connected");
    let s = format!("{:?}", ConnectionState::Failed("boom".into()));
    assert!(s.contains("Failed"), "Got: {}", s);
}

// ── ConnectionState Clone ─────────────────────────────────────────────

#[test]
fn test_connection_state_clone() {
    let state = ConnectionState::Connected;
    let cloned = state.clone();
    assert_eq!(state, cloned);
}

// ── ConnectionState all distinct ──────────────────────────────────────

#[test]
fn test_connection_state_all_distinct() {
    let states = [
        ConnectionState::Disconnected,
        ConnectionState::Connecting,
        ConnectionState::Connected,
        ConnectionState::Failed("err".into()),
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

// ── Public accessors ──────────────────────────────────────────────────

// These tests verify the public API shape.
// Real interaction tests are in test_mock_client.rs.
// We skip tests that would spawn real processes (echo).

#[test]
fn test_connection_state_public_api_exists() {
    // Just verify the type compiles and can be used.
    let _ = ConnectionState::Disconnected;
    let _ = ConnectionState::Connected;
    let _ = ConnectionState::Failed("e".into());
}

#[test]
fn test_server_capabilities_struct() {
    let caps = ServerCapabilities {
        experimental: None,
        logging: None,
        prompts: None,
        resources: None,
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
    };
    assert!(caps.tools.is_some());
    assert_eq!(caps.tools.unwrap().list_changed, Some(true));
}

#[test]
fn test_server_info_struct() {
    let info = ServerInfo {
        name: "test-srv".to_string(),
        version: "1.2.3".to_string(),
    };
    assert_eq!(info.name, "test-srv");
    assert_eq!(info.version, "1.2.3");
}
