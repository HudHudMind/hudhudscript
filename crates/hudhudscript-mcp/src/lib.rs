//! HudHudScript MCP Client
//!
//! This crate provides the Model Context Protocol client implementation.
//!
//! # Features
//!
//! - JSON-RPC 2.0 protocol support
//! - Stdio and SSE transports
//! - Tool and resource management
//! - Async/await support with Tokio
//!
//! # Example
//!
//! ```no_run
//! use hudhudscript_mcp::{McpClient, TransportConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create client with stdio transport
//!     let config = TransportConfig::stdio("uvx", vec!["mcp-server-sqlite".to_string()]);
//!     let client = McpClient::new(config).await?;
//!     
//!     // Initialize connection
//!     let init_response = client.initialize().await?;
//!     println!("Connected to: {}", init_response.server_info.name);
//!     
//!     // Start response handler. `McpClient::new` already split the transport
//!     // and kept the receive half, so use the compat entry point; the
//!     // `start_response_handler(recv)` form is for clients built with
//!     // `from_transport`, where the caller owns that half.
//!     client.start_response_handler_compat().await;
//!     
//!     // List available tools
//!     let tools = client.list_tools(None).await?;
//!     for tool in tools.tools {
//!         println!("Tool: {}", tool.name);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod protocol;
pub mod transport;

pub use client::{ConnectionState, McpClient};
pub use protocol::*;
pub use transport::{
    Transport, TransportConfig, TransportRecv, TransportSend, TransportType,
    INITIAL_RECONNECT_DELAY_MS, MAX_RECONNECT_ATTEMPTS,
};
