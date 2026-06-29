//! Stdio Transport — spawn subprocess, JSON-RPC over stdin/stdout.
//! Provides independent send (stdin) and receive (stdout) halves.

use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

use super::{Transport, TransportRecv, TransportSend};

/// Stdio Transport — owns the child process.
pub struct StdioTransport {
    process: Child,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
}

impl StdioTransport {
    pub fn new(command: &str, args: &[String]) -> Result<Self> {
        let mut process = tokio::process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn MCP server")?;

        let stdin = process.stdin.take().context("stdin")?;
        let stdout = process.stdout.take().context("stdout")?;
        let stdout_reader = BufReader::new(stdout);

        Ok(Self { process, stdin, stdout_reader })
    }
}

#[async_trait::async_trait]
impl Transport for StdioTransport {
    async fn close(&mut self) -> Result<()> {
        let _ = self.process.start_kill();
        let _ = self.process.wait().await;
        Ok(())
    }

    fn split(self: Box<Self>) -> (super::TransportSendHalf, super::TransportRecvHalf) {
        let send: super::TransportSendHalf = Box::new(StdioSendHalf {
            stdin: self.stdin,
        });
        let recv: super::TransportRecvHalf = Box::new(StdioRecvHalf {
            stdout_reader: self.stdout_reader,
            _process: self.process,
        });
        (send, recv)
    }
}

/// Send half — owns stdin writer.
pub struct StdioSendHalf {
    stdin: ChildStdin,
}

#[async_trait::async_trait]
impl TransportSend for StdioSendHalf {
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()> {
        let mut json = serde_json::to_string(&request)?;
        json.push('\n');
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }
}

/// Receive half — owns stdout reader + child process handle.
pub struct StdioRecvHalf {
    stdout_reader: BufReader<ChildStdout>,
    _process: Child,
}

#[async_trait::async_trait]
impl TransportRecv for StdioRecvHalf {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        let mut line = String::new();
        let n = self.stdout_reader.read_line(&mut line).await?;
        if n == 0 { anyhow::bail!("EOF"); }
        serde_json::from_str(&line).context("parse response")
    }
}

// Backward compat: StdioTransport itself implements send+recv
#[async_trait::async_trait]
impl super::TransportSend for StdioTransport {
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()> {
        let mut json = serde_json::to_string(&request)?;
        json.push('\n');
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl super::TransportRecv for StdioTransport {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        let mut line = String::new();
        let n = self.stdout_reader.read_line(&mut line).await?;
        if n == 0 { anyhow::bail!("EOF"); }
        serde_json::from_str(&line).context("parse response")
    }
}
