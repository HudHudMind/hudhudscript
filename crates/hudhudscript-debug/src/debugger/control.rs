use super::{DebugState, Debugger, PauseReason, StepMode};

impl Debugger {
    pub fn pause(&mut self) {
        self.state = DebugState::Paused;
        self.step_mode = None;
        self.pause_reason = Some(PauseReason::Explicit);
    }

    /// Resume execution until the next breakpoint (continue).
    pub fn resume(&mut self) {
        self.state = DebugState::Running;
        self.step_mode = None;
        self.pause_reason = None;
    }

    /// Alias for [`resume`](Self::resume) — run to the next breakpoint.
    pub fn continue_execution(&mut self) {
        self.resume();
    }

    /// Begin a step operation.
    ///
    /// The debugger transitions to [`DebugState::Stepping`] and records the
    /// current call-stack depth so that [`on_statement`] can decide when to
    /// pause next.
    ///
    /// | Mode | Pauses when |
    /// |------|-------------|
    /// | `Over` | next statement at the **same or shallower** call depth |
    /// | `Into` | **any** next statement (including inside a callee) |
    /// | `Out`  | next statement at a **shallower** call depth than now |
    pub fn step(&mut self, mode: StepMode) {
        self.state = DebugState::Stepping;
        self.step_mode = Some(mode);
        self.step_start_depth = self.call_stack.len();
        self.pause_reason = None;
    }

    /// Convenience: step over the next statement.
    pub fn step_over(&mut self) {
        self.step(StepMode::Over);
    }

    /// Convenience: step into the next function call.
    pub fn step_into(&mut self) {
        self.step(StepMode::Into);
    }

    /// Convenience: step out of the current function.
    pub fn step_out(&mut self) {
        self.step(StepMode::Out);
    }
}
