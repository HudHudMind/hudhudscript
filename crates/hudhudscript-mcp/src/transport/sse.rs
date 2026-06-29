//! SSE Transport — HTTP streaming with background event listener.
//! Provides independent send (POST) and receive (channel) halves.

use anyhow::{Context, Result};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

use super::{Transport, TransportRecv, TransportSend};

pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;
pub const INITIAL_RECONNECT_DELAY_MS: u64 = 500;

fn sse_retry_config() -> hudhudscript_utils::RetryConfig {
    hudhudscript_utils::RetryConfig {
        max_retries: MAX_RECONNECT_ATTEMPTS,
        base_delay: std::time::Duration::from_millis(INITIAL_RECONNECT_DELAY_MS),
        max_delay: std::time::Duration::from_secs(30),
        multiplier: 2.0,
        jitter: false,
    }
}

pub struct SseTransport {
    url: String,
    client: reqwest::Client,
    response_rx: mpsc::UnboundedReceiver<JsonRpcResponse>,
    _handle: tokio::task::JoinHandle<()>,
    close_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl SseTransport {
    pub async fn new(url: String) -> Result<Self> {
        // MCP-41: Disable automatic redirect following for SSRF protection.
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .context("Failed to create SSE HTTP client")?;
        let (response_tx, response_rx) = mpsc::unbounded_channel();
        let (close_tx, mut close_rx) = tokio::sync::oneshot::channel::<()>();

        let url_c = url.clone();
        let client_c = client.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = Self::event_loop(client_c, &url_c, response_tx) => {},
                _ = (&mut close_rx) => {},
            }
        });

        Ok(Self { url, client, response_rx, _handle: handle, close_tx: Some(close_tx) })
    }

    async fn event_loop(
        client: reqwest::Client, url: &str,
        response_tx: mpsc::UnboundedSender<JsonRpcResponse>,
    ) {
        let retry = sse_retry_config();
        let mut attempts: u32 = 0;
        let mut last_id: Option<String> = None;

        loop {
            match Self::stream(&client, url, &response_tx, &mut last_id).await {
                Ok(()) => break,
                Err(_) => {
                    attempts += 1;
                    if attempts > MAX_RECONNECT_ATTEMPTS { break; }
                    tokio::time::sleep(retry.delay_for_attempt(attempts - 1)).await;
                }
            }
        }
    }

    async fn stream(
        client: &reqwest::Client, url: &str,
        tx: &mpsc::UnboundedSender<JsonRpcResponse>,
        last_id: &mut Option<String>,
    ) -> Result<()> {
        let mut req = client.get(url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache");
        if let Some(ref id) = last_id { req = req.header("Last-Event-ID", id.as_str()); }

        let resp = req.send().await?;
        if !resp.status().is_success() { anyhow::bail!("SSE status {}", resp.status()); }

        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        let mut data = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(nl) = buf.find('\n') {
                let line = buf[..nl].trim_end_matches('\r').to_string();
                buf = buf[nl + 1..].to_string();
                if line.is_empty() {
                    if !data.is_empty() {
                        let json = data.trim_end_matches('\n');
                        if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(json) {
                            if tx.send(resp).is_err() { return Ok(()); }
                        }
                        data.clear();
                    }
                } else if let Some(v) = line.strip_prefix("data:") {
                    data.push_str(v.strip_prefix(' ').unwrap_or(v));
                    data.push('\n');
                } else if let Some(v) = line.strip_prefix("id:") {
                    let id = v.strip_prefix(' ').unwrap_or(v);
                    if !id.contains('\0') { *last_id = Some(id.to_string()); }
                }
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Transport for SseTransport {
    fn split(mut self: Box<Self>) -> (super::TransportSendHalf, super::TransportRecvHalf) {
        let close_tx = self.close_tx.take();
        let send: super::TransportSendHalf = Box::new(SseSendHalf {
            url: self.url.clone(),
            client: self.client.clone(),
        });
        let recv: super::TransportRecvHalf = Box::new(SseRecvHalf {
            rx: self.response_rx,
            _close_tx: close_tx,
            _handle: self._handle,
        });
        (send, recv)
    }
}

pub struct SseSendHalf { url: String, client: reqwest::Client }

#[async_trait::async_trait]
impl TransportSend for SseSendHalf {
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()> {
        let resp = self.client.post(&self.url).json(&request).send().await?;
        if !resp.status().is_success() { anyhow::bail!("SSE POST {}", resp.status()); }
        Ok(())
    }
}

pub struct SseRecvHalf {
    rx: mpsc::UnboundedReceiver<JsonRpcResponse>,
    _close_tx: Option<tokio::sync::oneshot::Sender<()>>,
    _handle: tokio::task::JoinHandle<()>,
}

#[async_trait::async_trait]
impl TransportRecv for SseRecvHalf {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        self.rx.recv().await.context("SSE channel closed")
    }
}

// Backward compat: SseTransport itself implements send+recv
#[async_trait::async_trait]
impl TransportSend for SseTransport {
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()> {
        let resp = self.client.post(&self.url).json(&request).send().await?;
        if !resp.status().is_success() { anyhow::bail!("SSE POST {}", resp.status()); }
        Ok(())
    }
}

#[async_trait::async_trait]
impl TransportRecv for SseTransport {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        self.response_rx.recv().await.context("SSE channel closed")
    }
}
