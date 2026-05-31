use crate::breakpoint::BreakpointKind;

use super::{DebugState, Debugger, PauseReason, StatementAction, StepMode};

impl Debugger {
    /// Called by the interpreter at the start of **every statement**.
    ///
    /// Returns `true` when execution should pause (breakpoint hit or step
    /// completed).  The interpreter should yield control back to the debug
    /// client / REPL when this returns `true`.
    pub fn on_statement(&mut self, file: &str, line: usize) -> bool {
        self.current_file = Some(file.to_string());
        self.current_line = Some(line);

        if let Some(bp_id) = self.find_breakpoint(file, line) {
            if let Some(bp) = self.breakpoints.get_mut(&bp_id) {
                bp.record_hit();
            }
            self.state = DebugState::Paused;
            self.step_mode = None;
            self.pause_reason = Some(PauseReason::Breakpoint(bp_id));
            return true;
        }

        if self.state == DebugState::Stepping {
            let current_depth = self.call_stack.len();
            let should_pause = match self.step_mode {
                Some(StepMode::Into) => true,
                Some(StepMode::Over) => current_depth <= self.step_start_depth,
                Some(StepMode::Out) => current_depth < self.step_start_depth,
                None => false,
            };

            if should_pause {
                self.state = DebugState::Paused;
                self.step_mode = None;
                self.pause_reason = Some(PauseReason::Step);
                return true;
            }
        }

        false
    }

    /// Statement hook with condition evaluation support.
    pub fn on_statement_with_eval<F>(&mut self, file: &str, line: usize, eval_condition: F) -> bool
    where
        F: Fn(&str) -> bool,
    {
        self.current_file = Some(file.to_string());
        self.current_line = Some(line);

        let bps = self.find_breakpoints_at(file, line);
        for (bp_id, condition) in bps {
            let should_trigger = match condition {
                Some(ref cond) => eval_condition(cond),
                None => true,
            };
            if should_trigger {
                if let Some(bp) = self.breakpoints.get_mut(&bp_id) {
                    bp.record_hit();
                }
                self.state = DebugState::Paused;
                self.step_mode = None;
                self.pause_reason = Some(PauseReason::Breakpoint(bp_id));
                return true;
            }
        }

        if self.state == DebugState::Stepping {
            let current_depth = self.call_stack.len();
            let should_pause = match self.step_mode {
                Some(StepMode::Into) => true,
                Some(StepMode::Over) => current_depth <= self.step_start_depth,
                Some(StepMode::Out) => current_depth < self.step_start_depth,
                None => false,
            };

            if should_pause {
                self.state = DebugState::Paused;
                self.step_mode = None;
                self.pause_reason = Some(PauseReason::Step);
                return true;
            }
        }

        false
    }

    /// Enhanced statement hook that also processes logpoints.
    pub fn on_statement_extended(&mut self, file: &str, line: usize) -> StatementAction {
        let logpoint_ids = self.find_logpoints(file, line);
        let mut log_messages = Vec::new();
        for lp_id in logpoint_ids {
            if let Some(bp) = self.breakpoints.get_mut(&lp_id) {
                bp.record_hit();
                if let BreakpointKind::Logpoint(ref msg) = bp.kind {
                    log_messages.push(msg.clone());
                }
            }
        }

        let should_pause = self.on_statement(file, line);

        StatementAction {
            should_pause,
            log_messages,
            pause_reason: self.pause_reason.clone(),
        }
    }

    /// Called by the runtime when an exception is thrown.
    ///
    /// Returns `true` if the debugger should pause (i.e. an exception
    /// breakpoint matches).
    pub fn on_exception(&mut self, error_type: &str, message: &str) -> bool {
        if self.break_on_all_exceptions {
            self.state = DebugState::Paused;
            self.step_mode = None;
            self.pause_reason = Some(PauseReason::Exception(format!(
                "{}: {}",
                error_type, message
            )));
            return true;
        }

        for &bp_id in &self.exception_breakpoints {
            if let Some(bp) = self.breakpoints.get(&bp_id) {
                if !bp.enabled {
                    continue;
                }
                let matches = match &bp.kind {
                    BreakpointKind::Exception(None) => true,
                    BreakpointKind::Exception(Some(filter)) => error_type.contains(filter.as_str()),
                    _ => false,
                };
                if matches {
                    if let Some(bp) = self.breakpoints.get_mut(&bp_id) {
                        bp.record_hit();
                    }
                    self.state = DebugState::Paused;
                    self.step_mode = None;
                    self.pause_reason = Some(PauseReason::Exception(format!(
                        "{}: {}",
                        error_type, message
                    )));
                    return true;
                }
            }
        }

        false
    }

    /// Called by the interpreter **before** entering a function body.
    pub fn push_frame(&mut self, name: String) {
        let frame = super::CallFrame {
            name,
            file: self.current_file.clone(),
            line: self.current_line,
        };
        self.call_stack.push(frame);
    }

    /// Called by the interpreter **after** a function body has finished.
    pub fn pop_frame(&mut self) {
        self.call_stack.pop();
    }

    /// Returns the call stack as a slice of frame name strings (for
    /// backwards compatibility).
    pub fn call_stack(&self) -> Vec<&str> {
        self.call_stack.iter().map(|f| f.name.as_str()).collect()
    }

    /// Returns the full call stack with source location information.
    pub fn call_frames(&self) -> &[super::CallFrame] {
        &self.call_stack
    }

    /// Returns the current call-stack depth.
    pub fn call_depth(&self) -> usize {
        self.call_stack.len()
    }
}
