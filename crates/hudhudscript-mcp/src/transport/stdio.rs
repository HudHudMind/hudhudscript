//! Stdio Transport — spawn a subprocess and speak JSON-RPC over its stdin/stdout.

use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

use super::{Transport, TransportRecv, TransportSend};

/// Stdio Transport
pub struct StdioTransport {
    process: tokio::process::Child,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new(command: &str, args: &[String]) -> Result<Self> {
        let mut process = tokio::process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn MCP server process")?;

        let stdin = process.stdin.take().context("Failed to get stdin")?;

        let stdout = process.stdout.take().context("Failed to get stdout")?;

        let stdout_reader = BufReader::new(stdout);

        Ok(Self {
            process,
            stdin,
            stdout_reader,
        })
    }
}

#[async_trait::async_trait]
impl TransportSend for StdioTransport {
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()> {
        let mut json = serde_json::to_string(&request).context("Failed to serialize request")?;
        json.push('\n');

        self.stdin
            .write_all(json.as_bytes())
            .await
            .context("Failed to write to stdin")?;

        self.stdin.flush().await.context("Failed to flush stdin")?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl TransportRecv for StdioTransport {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        let mut line = String::new();
        let n = self
            .stdout_reader
            .read_line(&mut line)
            .await
            .context("Failed to read from stdout")?;

        if n == 0 {
            anyhow::bail!("EOF reached");
        }

        let response: JsonRpcResponse =
            serde_json::from_str(&line).context("Failed to parse response")?;

        Ok(response)
    }
}

#[async_trait::async_trait]
impl Transport for StdioTransport {
    async fn close(&mut self) -> Result<()> {
        self.process
            .kill()
            .await
            .context("Failed to kill process")?;
        Ok(())
    }
}
