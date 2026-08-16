//! MCP Transport Protocol — traits and core types.

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Transport type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportType {
    /// Standard input/output
    Stdio,
    /// Server-Sent Events (HTTP)
    Sse,
}

/// Owned send half of a transport.
pub type TransportSendHalf = Box<dyn TransportSend>;

/// Owned receive half of a transport.
pub type TransportRecvHalf = Box<dyn TransportRecv>;

/// Transport trait — full send+receive+close.
///
/// Implementations provide a `split()` method for production use,
/// and also implement `TransportSend` + `TransportRecv` directly
/// for mock/testing or backward compat.
#[async_trait::async_trait]
pub trait Transport: TransportSend + TransportRecv + Send + Sync {
    /// Split into independent send and receive halves.
    fn split(self: Box<Self>) -> (TransportSendHalf, TransportRecvHalf);
    /// Close the transport and cleanup resources.
    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
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
