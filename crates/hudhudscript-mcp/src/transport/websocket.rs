//! G6: MCP WebSocket transport — placeholder.
//! Requires tokio-tungstenite dependency for full implementation.

/// WebSocket transport config.
pub struct WebSocketTransport {
    pub url: String,
}

impl WebSocketTransport {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string() }
    }
}
