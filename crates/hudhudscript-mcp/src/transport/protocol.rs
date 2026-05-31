//! MCP Transport Protocol — traits and core types.

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;

/// Transport type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportType {
    /// Standard input/output
    Stdio,
    /// Server-Sent Events (HTTP)
    Sse,
}

/// Transport trait for sending/receiving messages
///
/// Implementations must be safe to call `send` and `receive` concurrently
/// from separate tasks without holding a shared lock across both operations.
#[async_trait::async_trait]
pub trait Transport: TransportSend + TransportRecv + Send + Sync {
    /// Close the transport
    async fn close(&mut self) -> Result<()>;
}

/// Send half of a transport — used exclusively by request-sending code.
#[async_trait::async_trait]
pub trait TransportSend: Send + Sync {
    /// Send a JSON-RPC request
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()>;
}

/// Receive half of a transport — used exclusively by the response-handler task.
#[async_trait::async_trait]
pub trait TransportRecv: Send + Sync {
    /// Receive a JSON-RPC response
    async fn receive(&mut self) -> Result<JsonRpcResponse>;
}
