//! Debug Adapter Protocol (DAP) implementation.
//!
//! This module implements the DAP wire protocol over stdio, enabling
//! HudHudScript programs to be debugged from any DAP-compatible client
//! such as VS Code.
//!
//! The protocol uses a base protocol of HTTP-style headers followed by a
//! JSON payload:
//!
//! ```text
//! Content-Length: <length>\r\n
//! \r\n
//! <JSON payload>
//! ```
//!
//! Reference: <https://microsoft.github.io/debug-adapter-protocol/specification>

use crate::breakpoint::BreakpointId;
use crate::debugger::{Debugger, PauseReason, StepMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};

// ---------------------------------------------------------------------------
// DAP base protocol message types
// ---------------------------------------------------------------------------

/// A DAP protocol message (the envelope that wraps requests, responses, events).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DapMessage {
    #[serde(rename = "request")]
    Request(DapRequest),
    #[serde(rename = "response")]
    Response(DapResponse),
    #[serde(rename = "event")]
    Event(DapEvent),
}

/// A DAP request from the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapRequest {
    pub seq: i64,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// A DAP response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapResponse {
    pub seq: i64,
    pub request_seq: i64,
    pub success: bool,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
}

/// A DAP event sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapEvent {
    pub seq: i64,
    #[serde(rename = "event")]
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
}

// ---------------------------------------------------------------------------
// DAP argument / body helpers (typed subsets)
// ---------------------------------------------------------------------------

/// Arguments for the `initialize` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeArguments {
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub client_name: Option<String>,
    #[serde(default)]
    pub lines_start_at1: Option<bool>,
    #[serde(default)]
    pub columns_start_at1: Option<bool>,
}

/// Arguments for the `launch` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchArguments {
    /// The script file to debug.
    #[serde(default)]
    pub program: Option<String>,
    /// If `true`, stop at the first statement.
    #[serde(default)]
    pub stop_on_entry: Option<bool>,
    /// Working directory.
    #[serde(default)]
    pub cwd: Option<String>,
}

/// A single source-breakpoint from `setBreakpoints`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    pub line: usize,
    #[serde(default)]
    pub condition: Option<String>,
}

/// The `source` object used in various requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Arguments for the `setBreakpoints` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsArguments {
    pub source: Source,
    #[serde(default)]
    pub breakpoints: Option<Vec<SourceBreakpoint>>,
}

/// Arguments for the `stackTrace` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceArguments {
    pub thread_id: i64,
    #[serde(default)]
    pub start_frame: Option<i64>,
    #[serde(default)]
    pub levels: Option<i64>,
}

/// Arguments for the `scopes` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopesArguments {
    pub frame_id: i64,
}

/// Arguments for the `variables` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesArguments {
    pub variables_reference: i64,
}

/// Arguments for the `continue` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueArguments {
    pub thread_id: i64,
}

/// Arguments for `next` / `stepIn` / `stepOut` requests.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepArguments {
    pub thread_id: i64,
}

/// Arguments for the `disconnect` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisconnectArguments {
    #[serde(default)]
    pub restart: Option<bool>,
    #[serde(default)]
    pub terminate_debuggee: Option<bool>,
}

/// Arguments for the `evaluate` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluateArguments {
    pub expression: String,
    #[serde(default)]
    pub frame_id: Option<i64>,
    #[serde(default)]
    pub context: Option<String>,
}

/// Arguments for the `setExceptionBreakpoints` request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetExceptionBreakpointsArguments {
    pub filters: Vec<String>,
}

// ---------------------------------------------------------------------------
// Variable storage for DAP scopes/variables
// ---------------------------------------------------------------------------

/// A snapshot of a variable visible during a paused state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable {
    pub name: String,
    pub value: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub variables_reference: i64,
}

// ---------------------------------------------------------------------------
// DapServer — processes DAP messages and drives the Debugger
// ---------------------------------------------------------------------------

/// A DAP server that communicates over arbitrary `Read`/`Write` streams.
///
/// In production this is wired to stdin/stdout; in tests it can be driven
/// with in-memory buffers.
pub struct DapServer {
    pub debugger: Debugger,
    /// Monotonically increasing sequence number for outgoing messages.
    seq: i64,
    /// Whether the client has sent the `initialize` request.
    initialized: bool,
    /// Whether a `launch` or `attach` has been received.
    pub launched: bool,
    /// Whether a `disconnect` has been received.
    disconnected: bool,
    /// Program path from the launch request.
    program: Option<String>,
    /// Whether to stop on entry.
    stop_on_entry: bool,
    /// Tracks breakpoint IDs that were set via `setBreakpoints`, keyed by
    /// source path. Used to clear stale breakpoints when a new
    /// `setBreakpoints` arrives for the same file.
    source_breakpoints: HashMap<String, Vec<BreakpointId>>,
    /// Variables captured at the last pause, keyed by `variablesReference`.
    /// Reference `1` is the local scope.
    pub variable_store: HashMap<i64, Vec<Variable>>,
}

/// The single thread ID we report (HudHudScript is single-threaded).
pub const THREAD_ID: i64 = 1;
pub const THREAD_NAME: &str = "main";

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

mod constructor;
mod launch;
mod evaluate;

pub use constructor::*;
pub use launch::*;
pub use evaluate::*;
