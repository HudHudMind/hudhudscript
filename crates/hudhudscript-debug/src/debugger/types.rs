use crate::breakpoint::BreakpointId;

/// Debug state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugState {
    Running,
    Paused,
    Stepping,
}

/// Step mode — controls how the debugger advances after a pause.
///
/// These modes mirror the standard debugger step commands used in DAP-compatible
/// debuggers (e.g. VS Code, GDB):
///
/// - `Over`  – execute the next statement; do not descend into function calls.
/// - `Into`  – execute the next statement; if it is a function call, pause at
///   the first statement inside that function.
/// - `Out`   – run until the current function returns, then pause at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    /// Step over the next statement (do not descend into calls).
    Over,
    /// Step into the next function call.
    Into,
    /// Step out of the current function.
    Out,
}

/// Reason the debugger paused execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PauseReason {
    /// A user-set breakpoint was hit.
    Breakpoint(BreakpointId),
    /// A step operation completed.
    Step,
    /// [`Debugger::pause`] was called explicitly (e.g. "pause" button).
    Explicit,
    /// An exception breakpoint was triggered.
    Exception(String),
}

/// A single entry in the call stack, carrying both the human-readable label
/// and the source location at the time of the call.
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Human-readable frame label, e.g. `"myFunction"`.
    pub name: String,
    /// Source file at the point of the call (if known).
    pub file: Option<String>,
    /// Line number at the point of the call (if known).
    pub line: Option<usize>,
}

/// A watch expression registered by the user.
#[derive(Debug, Clone)]
pub struct WatchExpression {
    /// The expression text (HudHudScript syntax).
    pub expression: String,
    /// Last evaluated value (if any).
    pub last_value: Option<String>,
}

/// A variable snapshot visible in the current scope.
#[derive(Debug, Clone)]
pub struct ScopeVariable {
    pub name: String,
    pub value: String,
    pub ty: String,
}

/// Result of a breakpoint hit check. The debugger returns this from
/// `on_statement` so the runtime knows whether to pause and whether there are
/// logpoint messages to emit.
#[derive(Debug, Clone)]
pub struct StatementAction {
    /// Whether execution should pause.
    pub should_pause: bool,
    /// Logpoint messages to emit (if any breakpoint at this location is a
    /// logpoint). The runtime should write these to the debug console.
    pub log_messages: Vec<String>,
    /// The pause reason (only meaningful when `should_pause` is true).
    pub pause_reason: Option<PauseReason>,
}
