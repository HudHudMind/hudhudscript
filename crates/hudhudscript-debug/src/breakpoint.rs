//! Breakpoint management
//!
//! Supports standard line breakpoints, conditional breakpoints, logpoints,
//! and exception breakpoints.

use serde::{Deserialize, Serialize};

/// Breakpoint ID
pub type BreakpointId = usize;

/// The kind of a breakpoint, controlling what happens when the location is hit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BreakpointKind {
    /// A normal breakpoint — pause execution unconditionally.
    Normal,
    /// A conditional breakpoint — pause only when the expression evaluates to a
    /// truthy value. The string is an expression in HudHudScript syntax.
    Conditional(String),
    /// A logpoint — evaluate the message template and log it to the debug
    /// console *without* pausing execution.
    ///
    /// Placeholders inside `{...}` are evaluated as expressions.
    /// Example: `"x is now {x}"` logs the value of `x`.
    Logpoint(String),
    /// An exception breakpoint — pause when an exception matching the
    /// (optional) filter is thrown. If the filter is `None`, all exceptions
    /// trigger the breakpoint.
    Exception(Option<String>),
}

/// A breakpoint set at a specific source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: BreakpointId,
    pub file: String,
    pub line: usize,
    pub condition: Option<String>,
    pub enabled: bool,
    /// The kind of breakpoint. Defaults to [`BreakpointKind::Normal`].
    pub kind: BreakpointKind,
    /// Number of times this breakpoint has been hit (useful for diagnostics
    /// and hit-count breakpoints in the future).
    pub hit_count: u64,
}

impl Breakpoint {
    pub fn new(id: BreakpointId, file: String, line: usize) -> Self {
        Self {
            id,
            file,
            line,
            condition: None,
            enabled: true,
            kind: BreakpointKind::Normal,
            hit_count: 0,
        }
    }

    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition.clone());
        self.kind = BreakpointKind::Conditional(condition);
        self
    }

    /// Create a logpoint that logs a message template without pausing.
    pub fn with_log_message(mut self, message: String) -> Self {
        self.kind = BreakpointKind::Logpoint(message);
        self
    }

    /// Create an exception breakpoint with an optional filter.
    pub fn as_exception(mut self, filter: Option<String>) -> Self {
        self.kind = BreakpointKind::Exception(filter);
        self
    }

    /// Set the kind explicitly.
    pub fn with_kind(mut self, kind: BreakpointKind) -> Self {
        self.kind = kind;
        self
    }

    /// Record a hit and return the new hit count.
    pub fn record_hit(&mut self) -> u64 {
        self.hit_count += 1;
        self.hit_count
    }

    /// Returns `true` if this breakpoint is a logpoint.
    pub fn is_logpoint(&self) -> bool {
        matches!(self.kind, BreakpointKind::Logpoint(_))
    }

    /// Returns `true` if this breakpoint is an exception breakpoint.
    pub fn is_exception(&self) -> bool {
        matches!(self.kind, BreakpointKind::Exception(_))
    }

    /// Returns `true` if this breakpoint is conditional.
    pub fn is_conditional(&self) -> bool {
        matches!(self.kind, BreakpointKind::Conditional(_))
    }
}
