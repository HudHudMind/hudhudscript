//! G6: MCP WebSocket transport (pending tokio-tungstenite dependency).

/// WebSocket transport config.
pub struct WebSocketTransport {
    pub url: String,
}

impl WebSocketTransport {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}
