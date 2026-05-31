use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_debug::Debugger;

impl VM {
    pub fn with_fuel(&mut self, limit: u64) {
        self.fuel_limit = Some(limit);
        self.fuel_remaining = limit;
    }

    /// Alias that makes new call sites read "set_fuel" while leaving
    /// the historical `with_fuel` name in place for preserved tests.
    pub fn set_fuel(&mut self, limit: u64) {
        self.with_fuel(limit);
    }

    /// Remaining fuel.  Returns the raw `u64` counter (0 when no fuel
    /// limit is configured).  Preserved coverage tests assert on the
    /// raw integer (`vm.remaining_fuel() < 100`) so we keep the
    /// concrete-u64 signature here; the interpreter-parity
    /// `Option<u64>` shape is exposed through
    /// [`VM::remaining_fuel_opt`] and the `vm_interpreter::Interpreter`
    /// shim.
    pub fn remaining_fuel(&self) -> u64 {
        self.fuel_remaining
    }

    /// Interpreter-parity accessor: `None` when no fuel limit has been
    /// configured, otherwise `Some(remaining)`.
    pub fn remaining_fuel_opt(&self) -> Option<u64> {
        if self.fuel_limit.is_some() {
            Some(self.fuel_remaining)
        } else {
            None
        }
    }

    // ── Issue #661: Debugger integration ──────────────────────────────

    /// Attach a debugger to this VM (Issue #661).
    /// PERF-39: debugger is stored in a Box for `Option<...>` niche so
    /// the per-instruction `is_some()` probe is a nullable-pointer check.
    pub fn attach_debugger(&mut self, debugger: Debugger) {
        self.debugger = Some(Box::new(debugger));
    }

    /// Detach the debugger and return it (Issue #661).
    pub fn detach_debugger(&mut self) -> Option<Debugger> {
        self.debugger.take().map(|b| *b)
    }

    /// Set the current source file name for debugger location tracking.
    pub fn set_current_file(&mut self, file: String) {
        self.current_file = Some(file);
    }

    /// Get a mutable reference to the attached debugger (Issue #661).
    pub fn debugger_mut(&mut self) -> Option<&mut Debugger> {
        self.debugger.as_deref_mut()
    }

    /// Read-only accessor for the attached debugger — used by the CLI
    /// `hudi debug` REPL to poll pause state without having to lock the
    /// VM for mutation.
    pub fn debugger_ref(&self) -> Option<&Debugger> {
        self.debugger.as_deref()
    }

    /// Reset fuel to original limit
    pub fn reset_fuel(&mut self) {
        if let Some(limit) = self.fuel_limit {
            self.fuel_remaining = limit;
        }
    }

    /// Consume one unit of fuel. Called every instruction, so the
    /// `fuel_limit.is_none()` hot path must be one branch, no call.
    /// Audit v3 Finding 5.1 (dispatch loop hygiene).
    #[inline(always)]
    pub(crate) fn consume_fuel(&mut self) -> CompileResult<()> {
        if self.fuel_limit.is_some() {
            if self.fuel_remaining == 0 {
                return Err(compile_codes::runtime_error(
                    "Out of gas: execution fuel exhausted".to_string(),
                ));
            }
            self.fuel_remaining -= 1;
        }
        Ok(())
    }
}
