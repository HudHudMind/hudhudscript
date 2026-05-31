//! Transport configuration — factory for concrete transports.

use anyhow::{Context, Result};

use super::{SseTransport, StdioTransport, Transport, TransportType};

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
}

impl TransportConfig {
    /// Create stdio transport config
    pub fn stdio(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            transport_type: TransportType::Stdio,
            command: Some(command.into()),
            args,
            url: None,
        }
    }

    /// Create SSE transport config
    pub fn sse(url: impl Into<String>) -> Self {
        Self {
            transport_type: TransportType::Sse,
            command: None,
            args: Vec::new(),
            url: Some(url.into()),
        }
    }

    /// Create transport from config
    pub async fn create_transport(&self) -> Result<Box<dyn Transport>> {
        match self.transport_type {
            TransportType::Stdio => {
                let command = self
                    .command
                    .as_ref()
                    .context("Command required for stdio transport")?;
                let transport = StdioTransport::new(command, &self.args)?;
                Ok(Box::new(transport))
            }
            TransportType::Sse => {
                let url = self
                    .url
                    .as_ref()
                    .context("URL required for SSE transport")?;
                let transport = SseTransport::new(url.clone()).await?;
                Ok(Box::new(transport))
            }
        }
    }
}
