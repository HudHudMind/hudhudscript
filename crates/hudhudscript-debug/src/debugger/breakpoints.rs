use crate::breakpoint::{Breakpoint, BreakpointId, BreakpointKind};

use super::Debugger;

impl Debugger {
    pub fn add_breakpoint(&mut self, file: String, line: usize) -> BreakpointId {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        let bp = Breakpoint::new(id, file, line);
        self.breakpoints.insert(id, bp);
        id
    }

    /// Add a conditional breakpoint that only pauses when `condition` evaluates
    /// to a truthy value.
    pub fn add_conditional_breakpoint(
        &mut self,
        file: String,
        line: usize,
        condition: String,
    ) -> BreakpointId {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        let bp = Breakpoint::new(id, file, line).with_condition(condition);
        self.breakpoints.insert(id, bp);
        id
    }

    /// Add a logpoint that logs a message template without pausing.
    pub fn add_logpoint(&mut self, file: String, line: usize, message: String) -> BreakpointId {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        let bp = Breakpoint::new(id, file, line).with_log_message(message);
        self.breakpoints.insert(id, bp);
        id
    }

    /// Add an exception breakpoint.
    pub fn add_exception_breakpoint(&mut self, filter: Option<String>) -> BreakpointId {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        let bp = Breakpoint::new(id, "<exception>".to_string(), 0).as_exception(filter);
        self.breakpoints.insert(id, bp);
        self.exception_breakpoints.push(id);
        id
    }

    /// Enable breaking on all exceptions, regardless of whether any specific
    /// exception breakpoints have been registered.
    pub fn set_break_on_all_exceptions(&mut self, enabled: bool) {
        self.break_on_all_exceptions = enabled;
    }

    /// Returns `true` if the debugger will break on all exceptions.
    pub fn break_on_all_exceptions(&self) -> bool {
        self.break_on_all_exceptions
    }

    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> bool {
        if self.breakpoints.remove(&id).is_some() {
            self.exception_breakpoints.retain(|&eid| eid != id);
            true
        } else {
            false
        }
    }

    /// Returns an immutable reference to a breakpoint by its ID.
    pub fn get_breakpoint(&self, id: BreakpointId) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Returns a mutable reference to a breakpoint by its ID.
    pub fn get_breakpoint_mut(&mut self, id: BreakpointId) -> Option<&mut Breakpoint> {
        self.breakpoints.get_mut(&id)
    }

    /// Returns all breakpoints.
    pub fn breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Toggle a breakpoint on/off.
    pub fn toggle_breakpoint(&mut self, id: BreakpointId) -> bool {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = !bp.enabled;
            bp.enabled
        } else {
            false
        }
    }

    /// Returns the breakpoint (if any) that matches `file` and `line`.
    pub(crate) fn find_breakpoint(&self, file: &str, line: usize) -> Option<BreakpointId> {
        self.breakpoints
            .values()
            .find(|bp| {
                bp.enabled
                    && bp.file == file
                    && bp.line == line
                    && !bp.is_logpoint()
                    && !bp.is_exception()
                    && !bp.is_conditional()
            })
            .map(|bp| bp.id)
    }

    /// Returns all non-logpoint, non-exception breakpoints at the given location,
    /// including conditional ones. Each entry is `(id, Option<condition_string>)`.
    pub fn find_breakpoints_at(
        &self,
        file: &str,
        line: usize,
    ) -> Vec<(BreakpointId, Option<String>)> {
        self.breakpoints
            .values()
            .filter(|bp| {
                bp.enabled
                    && bp.file == file
                    && bp.line == line
                    && !bp.is_logpoint()
                    && !bp.is_exception()
            })
            .map(|bp| {
                let cond = match &bp.kind {
                    BreakpointKind::Conditional(c) => Some(c.clone()),
                    _ => None,
                };
                (bp.id, cond)
            })
            .collect()
    }

    /// Returns all logpoints that match `file` and `line`.
    pub(crate) fn find_logpoints(&self, file: &str, line: usize) -> Vec<BreakpointId> {
        self.breakpoints
            .values()
            .filter(|bp| bp.enabled && bp.file == file && bp.line == line && bp.is_logpoint())
            .map(|bp| bp.id)
            .collect()
    }
}
