//! SSE Transport (Server-Sent Events) — streaming HTTP transport with auto-reconnect.

use anyhow::{Context, Result};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

use super::{Transport, TransportRecv, TransportSend};

/// Maximum number of consecutive reconnect attempts before giving up.
pub const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Initial delay between reconnect attempts (doubles on each retry).
pub const INITIAL_RECONNECT_DELAY_MS: u64 = 500;

/// SSE Transport (Server-Sent Events)
///
/// Opens a streaming GET connection to the server endpoint and listens for
/// Server-Sent Events.  Each SSE `data:` line is parsed as a JSON-RPC
/// response and forwarded to the internal channel consumed by
/// [`Transport::receive`].
///
/// The background listener reconnects automatically on transient errors
/// (up to a configurable limit) so callers see a seamless stream of
/// responses.
pub struct SseTransport {
    url: String,
    client: reqwest::Client,
    response_rx: mpsc::UnboundedReceiver<JsonRpcResponse>,
    _event_source_handle: tokio::task::JoinHandle<()>,
}

/// Shared retry configuration for SSE reconnection (Issue #692).
///
/// Uses `RetryConfig` from `hudhudscript-utils` to keep backoff logic in one
/// place across the entire project.
fn sse_retry_config() -> hudhudscript_utils::RetryConfig {
    hudhudscript_utils::RetryConfig {
        max_retries: MAX_RECONNECT_ATTEMPTS,
        base_delay: std::time::Duration::from_millis(INITIAL_RECONNECT_DELAY_MS),
        max_delay: std::time::Duration::from_secs(30),
        multiplier: 2.0,
        jitter: false,
    }
}

impl SseTransport {
    /// Create a new SSE transport and start the background event-listener.
    ///
    /// The listener opens a streaming GET request to `url`, parses incoming
    /// SSE frames, and pushes parsed JSON-RPC responses through an
    /// unbounded channel.  Transient connection errors trigger automatic
    /// reconnection with exponential back-off.
    pub async fn new(url: String) -> Result<Self> {
        let client = reqwest::Client::new();
        let (response_tx, response_rx) = mpsc::unbounded_channel::<JsonRpcResponse>();

        let url_clone = url.clone();
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            Self::sse_event_loop(client_clone, &url_clone, response_tx).await;
        });

        Ok(Self {
            url,
            client,
            response_rx,
            _event_source_handle: handle,
        })
    }

    /// Core SSE event-listening loop with automatic reconnection.
    ///
    /// Reconnection delay is computed via the shared [`RetryConfig`] returned by
    /// [`sse_retry_config`] (Issue #692).
    async fn sse_event_loop(
        client: reqwest::Client,
        url: &str,
        response_tx: mpsc::UnboundedSender<JsonRpcResponse>,
    ) {
        let retry_config = sse_retry_config();
        let mut reconnect_attempts: u32 = 0;
        let mut last_event_id: Option<String> = None;

        loop {
            match Self::connect_and_listen(&client, url, &response_tx, &mut last_event_id).await {
                Ok(()) => {
                    // Stream ended cleanly (server closed connection).
                    tracing::info!(url = %url, "SSE stream ended; closing listener");
                    break;
                }
                Err(e) => {
                    reconnect_attempts += 1;
                    if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
                        tracing::error!(
                            url = %url,
                            error = %e,
                            attempts = reconnect_attempts,
                            "SSE listener exceeded max reconnect attempts; giving up"
                        );
                        break;
                    }

                    let delay = retry_config.delay_for_attempt(reconnect_attempts - 1);
                    tracing::warn!(
                        url = %url,
                        error = %e,
                        attempt = reconnect_attempts,
                        delay_ms = delay.as_millis(),
                        "SSE connection error; reconnecting"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    /// Open a single SSE connection and stream events until it ends or errors.
    async fn connect_and_listen(
        client: &reqwest::Client,
        url: &str,
        response_tx: &mpsc::UnboundedSender<JsonRpcResponse>,
        last_event_id: &mut Option<String>,
    ) -> Result<()> {
        let mut request = client
            .get(url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache");

        // If we have a Last-Event-ID from a previous connection, send it so
        // the server can resume from where we left off (per the SSE spec).
        if let Some(ref id) = last_event_id {
            request = request.header("Last-Event-ID", id.as_str());
        }

        let response = request
            .send()
            .await
            .context("Failed to open SSE connection")?;

        if !response.status().is_success() {
            anyhow::bail!("SSE connection returned status {}", response.status());
        }

        // Stream the response body as a byte stream and parse SSE frames.
        let mut byte_stream = response.bytes_stream();
        let mut buffer = String::new();
        // Accumulated fields for the current SSE event.
        let mut event_data = String::new();
        let mut event_type = String::new();
        let mut event_id: Option<String> = None;

        while let Some(chunk_result) = byte_stream.next().await {
            let chunk = chunk_result.context("Error reading SSE stream chunk")?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process all complete lines in the buffer.
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].trim_end_matches('\r').to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    // Empty line = end of an SSE event.  Dispatch if we have
                    // accumulated data.
                    if !event_data.is_empty() {
                        // Update last-event-id for reconnection.
                        if let Some(ref id) = event_id {
                            *last_event_id = Some(id.clone());
                        }

                        // Only process "message" events (the default) or
                        // explicit "message" type.  Ignore other event types.
                        let should_process = event_type.is_empty() || event_type == "message";

                        if should_process {
                            // Remove trailing newline from concatenated data
                            // lines (per SSE spec, each data: appends '\n').
                            let data = event_data.trim_end_matches('\n');

                            match serde_json::from_str::<JsonRpcResponse>(data) {
                                Ok(resp) => {
                                    if response_tx.send(resp).is_err() {
                                        // Receiver dropped -- stop listening.
                                        tracing::debug!(
                                            "SSE response channel closed; stopping listener"
                                        );
                                        return Ok(());
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        data = %data,
                                        error = %e,
                                        "Failed to parse SSE data as JSON-RPC response; skipping"
                                    );
                                }
                            }
                        }

                        // Reset for next event.
                        event_data.clear();
                        event_type.clear();
                        event_id = None;
                    }
                } else if let Some(rest) = line.strip_prefix("data:") {
                    // `data:` field -- may appear multiple times per event;
                    // values are concatenated with '\n' separators.
                    let value = rest.strip_prefix(' ').unwrap_or(rest);
                    event_data.push_str(value);
                    event_data.push('\n');
                } else if let Some(rest) = line.strip_prefix("event:") {
                    event_type = rest.strip_prefix(' ').unwrap_or(rest).to_string();
                } else if let Some(rest) = line.strip_prefix("id:") {
                    let id_value = rest.strip_prefix(' ').unwrap_or(rest);
                    // Per spec, id fields containing NUL must be ignored.
                    if !id_value.contains('\0') {
                        event_id = Some(id_value.to_string());
                    }
                } else if line.starts_with(':') {
                    // Comment line -- ignore (used as keep-alive by servers).
                } else if line.starts_with("retry:") {
                    // `retry:` field -- could update reconnect delay.
                    // Acknowledged but not currently acted upon.
                    tracing::trace!(line = %line, "SSE retry field received (not applied)");
                }
                // Unknown field names are ignored per the SSE specification.
            }
        }

        // Stream exhausted normally.
        Ok(())
    }
}

#[async_trait::async_trait]
impl TransportSend for SseTransport {
    async fn send(&mut self, request: JsonRpcRequest) -> Result<()> {
        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .context("Failed to send SSE request")?;

        if !response.status().is_success() {
            anyhow::bail!("SSE request failed with status: {}", response.status());
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl TransportRecv for SseTransport {
    async fn receive(&mut self) -> Result<JsonRpcResponse> {
        self.response_rx
            .recv()
            .await
            .context("SSE transport: stream closed or event-listener stopped")
    }
}

#[async_trait::async_trait]
impl Transport for SseTransport {
    async fn close(&mut self) -> Result<()> {
        // Abort the background event-listener task and drop the channel.
        self._event_source_handle.abort();
        self.response_rx.close();
        Ok(())
    }
}
