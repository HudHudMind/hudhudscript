//! Text-stream universality for agent communication.
//!
//! # Issue #105 — Doug McIlroy Review: Enforce Text-Stream Universality between Agents
//!
//! Doug McIlroy's foundational Unix philosophy dictates that programs should:
//! 1. Write to standard output.
//! 2. Read from standard input.
//! 3. Communicate through text streams.
//!
//! Applied to HudHudScript agent orchestration, every agent must expose a
//! uniform text-stream interface for its inputs and outputs.  This enables:
//!
//! - **Pipeline composability** — agents can be chained like Unix pipes:
//!   `agent_a | agent_b | agent_c`
//! - **Universal interoperability** — any agent can communicate with any
//!   other without bespoke adapters.
//! - **Testability** — streams can be captured, replayed, and inspected.
//! - **Language independence** — agents implemented in Python, Rust, or any
//!   other language all speak the same line-delimited JSON text protocol.
//!
//! ## Protocol
//!
//! All inter-agent messages are encoded as **newline-delimited JSON** (NDJSON):
//!
//! ```text
//! {"type":"data","payload":"hello"}\n
//! {"type":"data","payload":"world"}\n
//! {"type":"eof"}\n
//! ```
//!
//! This matches the Unix convention of text lines separated by `\n` and
//! makes the protocol trivially inspectable with standard Unix tools
//! (`cat`, `jq`, `grep`, `wc -l`, …).
//!
//! ## Architecture
//!
//! ```text
//!  Agent A                                    Agent B
//!  ┌─────────────────┐                        ┌─────────────────┐
//!  │ execute_task()  │                        │ execute_task()  │
//!  │   │             │    TextStream pipe     │   │             │
//!  │   ▼             │  ─────────────────►   │   ▼             │
//!  │ StreamWriter    │  NDJSON over channel   │ StreamReader    │
//!  └─────────────────┘                        └─────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Message protocol
// ---------------------------------------------------------------------------

/// A single message in the inter-agent text stream.
///
/// The wire format is newline-delimited JSON (NDJSON).  Each variant
/// serialises to a distinct `"type"` field so that readers can dispatch
/// without schema knowledge of the payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// A data frame carrying an arbitrary text payload.
    Data {
        /// The text payload.  For structured data this is a JSON-encoded string.
        payload: String,
        /// Optional content-type hint (e.g., `"application/json"`,
        /// `"text/plain"`).  Consumers MAY ignore this field.
        #[serde(skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    /// A structured error frame.  The stream is still open after an error;
    /// subsequent `Data` or `Eof` frames may follow.
    Error {
        /// Short error code (e.g., `"RATE_LIMITED"`, `"TIMEOUT"`).
        code: String,
        /// Human-readable description.
        message: String,
    },
    /// Signals the end of the stream.  No further frames will follow.
    Eof,
}

impl StreamMessage {
    /// Convenience constructor for a plain-text data frame.
    pub fn text(payload: impl Into<String>) -> Self {
        StreamMessage::Data {
            payload: payload.into(),
            content_type: Some("text/plain".to_string()),
        }
    }

    /// Convenience constructor for a JSON data frame.
    pub fn json(value: &serde_json::Value) -> Self {
        StreamMessage::Data {
            payload: value.to_string(),
            content_type: Some("application/json".to_string()),
        }
    }

    /// Convenience constructor for an error frame.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        StreamMessage::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Serialise this message to a single NDJSON line (without the trailing `\n`).
    pub fn to_ndjson(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialise a message from a single NDJSON line.
    pub fn from_ndjson(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }

    /// Returns `true` if this is an EOF frame.
    pub fn is_eof(&self) -> bool {
        matches!(self, StreamMessage::Eof)
    }
}

impl fmt::Display for StreamMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_ndjson() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<serialisation error: {}>", e),
        }
    }
}

// ---------------------------------------------------------------------------
// Stream writer / reader
// ---------------------------------------------------------------------------

/// The *write end* of an agent text stream.
///
/// An `AgentStreamWriter` is given to the producing agent.  It sends
/// [`StreamMessage`] frames into the channel.  When the writer is dropped
/// without an explicit `close()`, an `Eof` frame is sent automatically so
/// that the consumer is never left hanging.
pub struct AgentStreamWriter {
    sender: mpsc::Sender<StreamMessage>,
    closed: bool,
}

impl AgentStreamWriter {
    fn new(sender: mpsc::Sender<StreamMessage>) -> Self {
        Self {
            sender,
            closed: false,
        }
    }

    /// Send a data frame.
    pub async fn write(&self, msg: StreamMessage) -> Result<(), StreamError> {
        self.sender
            .send(msg)
            .await
            .map_err(|_| StreamError::ChannelClosed)
    }

    /// Write a plain-text payload.
    pub async fn write_text(&self, text: impl Into<String>) -> Result<(), StreamError> {
        self.write(StreamMessage::text(text)).await
    }

    /// Write a JSON value payload.
    pub async fn write_json(&self, value: &serde_json::Value) -> Result<(), StreamError> {
        self.write(StreamMessage::json(value)).await
    }

    /// Signal end-of-stream and close the write end.
    pub async fn close(&mut self) -> Result<(), StreamError> {
        if !self.closed {
            self.sender
                .send(StreamMessage::Eof)
                .await
                .map_err(|_| StreamError::ChannelClosed)?;
            self.closed = true;
        }
        Ok(())
    }
}

impl Drop for AgentStreamWriter {
    fn drop(&mut self) {
        if !self.closed {
            // Best-effort: try to send Eof synchronously.
            let _ = self.sender.try_send(StreamMessage::Eof);
        }
    }
}

/// The *read end* of an agent text stream.
///
/// An `AgentStreamReader` is given to the consuming agent.  It receives
/// [`StreamMessage`] frames from the channel.
pub struct AgentStreamReader {
    receiver: mpsc::Receiver<StreamMessage>,
}

impl AgentStreamReader {
    fn new(receiver: mpsc::Receiver<StreamMessage>) -> Self {
        Self { receiver }
    }

    /// Receive the next frame.  Returns `None` when the channel is closed.
    pub async fn next(&mut self) -> Option<StreamMessage> {
        self.receiver.recv().await
    }

    /// Collect all frames until `Eof` or channel close.
    pub async fn collect_all(&mut self) -> Vec<StreamMessage> {
        let mut frames = Vec::new();
        loop {
            match self.receiver.recv().await {
                None => break,
                Some(msg) => {
                    let is_eof = msg.is_eof();
                    frames.push(msg);
                    if is_eof {
                        break;
                    }
                }
            }
        }
        frames
    }

    /// Collect all data payloads as a concatenated string, ignoring errors and
    /// Eof frames.
    pub async fn collect_text(&mut self) -> String {
        let frames = self.collect_all().await;
        frames
            .into_iter()
            .filter_map(|f| match f {
                StreamMessage::Data { payload, .. } => Some(payload),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

// ---------------------------------------------------------------------------
// Stream factory
// ---------------------------------------------------------------------------

/// Create a linked (writer, reader) pair — the agent equivalent of a Unix pipe.
///
/// # Usage
/// ```text
/// let (mut writer, mut reader) = agent_pipe(16);
/// writer.write_text("hello from agent A").await.unwrap();
/// writer.close().await.unwrap();
///
/// let text = reader.collect_text().await;
/// assert_eq!(text, "hello from agent A");
/// ```
pub fn agent_pipe(buffer: usize) -> (AgentStreamWriter, AgentStreamReader) {
    let (tx, rx) = mpsc::channel(buffer);
    (AgentStreamWriter::new(tx), AgentStreamReader::new(rx))
}

// ---------------------------------------------------------------------------
// Adapter: serde_json::Value ↔ text stream
// ---------------------------------------------------------------------------

/// Adapters that convert between structured data and the text-stream protocol.
///
/// These let agents that produce `serde_json::Value` (the orchestration
/// layer's native format) slot directly into the universal text-stream pipeline
/// without manual serialisation boilerplate.
pub struct TextStreamAdapter;

impl TextStreamAdapter {
    /// Encode a `serde_json::Value` as a stream of NDJSON `Data` frames and
    /// send them over `writer`.
    ///
    /// For object/array values the entire value is sent as one frame.
    /// For large values (>1 MiB) callers should chunk manually.
    pub async fn send_value(
        writer: &AgentStreamWriter,
        value: &serde_json::Value,
    ) -> Result<(), StreamError> {
        writer.write_json(value).await
    }

    /// Receive frames from `reader` until `Eof` and decode the payloads as a
    /// single `serde_json::Value`.
    ///
    /// Concatenates all data payloads, then parses the result as JSON.
    pub async fn receive_value(
        reader: &mut AgentStreamReader,
    ) -> Result<serde_json::Value, StreamError> {
        let text = reader.collect_text().await;
        serde_json::from_str(&text).map_err(|e| StreamError::DecodeError(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during stream I/O.
#[derive(Debug)]
pub enum StreamError {
    ChannelClosed,
    EncodeError(String),
    DecodeError(String),
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            StreamError::ChannelClosed => write!(f, "stream channel closed unexpectedly"),
            StreamError::EncodeError(s) => write!(f, "failed to serialise message: {}", s),
            StreamError::DecodeError(s) => write!(f, "failed to deserialise message: {}", s),
        }
    }
}

impl std::error::Error for StreamError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl StreamError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            StreamError::ChannelClosed => hudhudscript_errors::ErrorCode::StreamChannelClosed,
            StreamError::DecodeError(..) => hudhudscript_errors::ErrorCode::StreamDecodeError,
            StreamError::EncodeError(..) => hudhudscript_errors::ErrorCode::StreamEncodeError,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<StreamError> for hudhudscript_errors::Error {
    fn from(e: StreamError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
