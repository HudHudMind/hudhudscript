//! Debugger implementation
//!
//! Provides execution control (step, continue, pause), breakpoint management
//! (including conditional, logpoints, and exception breakpoints), variable
//! inspection, watch expressions, scope introspection, and call-stack tracking.

pub mod breakpoints;
pub mod control;
pub mod hooks;
pub mod types;
pub mod variables;

pub use types::*;

use crate::breakpoint::{Breakpoint, BreakpointId};
use std::collections::HashMap;

/// Debugger
///
/// Tracks execution state and integrates with the interpreter via hook methods.
/// The interpreter is expected to call:
///
/// - [`Debugger::on_statement`] at the start of every statement.
/// - [`Debugger::push_frame`] when entering a function.
/// - [`Debugger::pop_frame`] when leaving a function.
/// - [`Debugger::on_exception`] when an exception is thrown.
pub struct Debugger {
    pub(crate) state: DebugState,
    pub(crate) breakpoints: HashMap<BreakpointId, Breakpoint>,
    pub(crate) next_bp_id: BreakpointId,
    pub(crate) current_line: Option<usize>,
    pub(crate) current_file: Option<String>,
    pub(crate) call_stack: Vec<CallFrame>,
    /// Active step mode (only meaningful when state == Stepping).
    pub(crate) step_mode: Option<StepMode>,
    /// Call-stack depth recorded when the current step began.
    pub(crate) step_start_depth: usize,
    /// Most recent pause reason (useful for a DAP `StoppedEvent`).
    pub(crate) pause_reason: Option<PauseReason>,
    /// Scope variables provided by the runtime at the last pause.
    pub(crate) scope_variables: Vec<ScopeVariable>,
    /// Watch expressions registered by the user.
    pub(crate) watch_expressions: HashMap<String, WatchExpression>,
    /// Exception breakpoints (line-independent, matched by error type).
    pub(crate) exception_breakpoints: Vec<BreakpointId>,
    /// Whether exception breakpoints are enabled globally.
    pub(crate) break_on_all_exceptions: bool,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            state: DebugState::Running,
            breakpoints: HashMap::new(),
            next_bp_id: 1,
            current_line: None,
            current_file: None,
            call_stack: Vec::new(),
            step_mode: None,
            step_start_depth: 0,
            pause_reason: None,
            scope_variables: Vec::new(),
            watch_expressions: HashMap::new(),
            exception_breakpoints: Vec::new(),
            break_on_all_exceptions: false,
        }
    }

    // -------------------------------------------------------------------------
    // State accessors
    // -------------------------------------------------------------------------

    pub fn state(&self) -> DebugState {
        self.state
    }

    /// Returns the reason the debugger last paused, if any.
    pub fn pause_reason(&self) -> Option<&PauseReason> {
        self.pause_reason.as_ref()
    }

    /// Returns the currently tracked source location.
    pub fn current_location(&self) -> Option<(&str, usize)> {
        match (&self.current_file, self.current_line) {
            (Some(f), Some(l)) => Some((f.as_str(), l)),
            _ => None,
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}
