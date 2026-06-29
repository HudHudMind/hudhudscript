use crate::vm::VM;
use crate::vm::{EndFinallyAction, FinallyStep, PendingFlow, SavedFinally};
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::{CompileError, CompileResult};
use hudhudscript_bytecode::Bytecode;
use hudhudscript_bytecode::{Instruction, Value16};

impl VM {
    pub(crate) fn try_catch_runtime_error(&mut self, err: &CompileError) -> Option<usize> {
        let (catch_ip, iter_depth, loop_depth) = self.try_frames.pop()?;

        // Issue #661 — exception breakpoint hook. Fire BEFORE unwinding
        // so the paused debugger still sees a coherent scope / stack
        // snapshot from inside the try body. Caller is expected to
        // block-wait while `DebugState::Paused` is set (same busy-wait
        // loop as the on_statement hook).
        self.notify_debugger_exception(err);

        // Register-based VM: no stack truncation needed.
        self.iterators.truncate(iter_depth);
        self.iterator_generators.truncate(iter_depth);
        self.loop_headers.truncate(loop_depth);

        let err_val = Self::runtime_error_to_value(err);
        self.registers[255] = err_val;

        Some(catch_ip)
    }

    /// Gap 2 — if an enclosing finally frame is active, stash the pending
    /// return (top of stack) in `pending_flow` and return the finally IP so
    /// the caller can jump to the finally body.  The stored return value
    /// is re-pushed by `FinallyExit` once the finally body completes
    /// normally.  No-op (returns `None`) when no finally frame is active.
    ///
    /// Called by the `Return` instruction.  The VM's stack isolation
    /// guarantees the finally frame belongs to the *current* function:
    /// `call_chunk_with_captures` swaps the entire `self.stack` and every
    /// nested chunk drains its own try/finally frames before returning
    /// (see `Stmt::Try` compiler emission).
    ///
    /// Gap 2 — counterpart of route-through-finally for `Throw`.
    /// Called when a throw is raised but NO try frame is ready to catch it
    /// (either because the guarded region had no catch clause, or the
    /// catch itself re-threw).  If a finally frame is active we stash the
    /// thrown value and jump to the finally IP so the finally body runs
    /// before the exception propagates upwards.
    pub(crate) fn route_throw_through_finally(&mut self, thrown: Value16) -> Option<usize> {
        let (finally_ip, _iter_depth, _loop_depth) = self.finally_frames.pop()?;
        self.pending_flow = Some(PendingFlow::Throw(Box::new(thrown)));
        Some(finally_ip)
    }

    /// Gap 2 — invoked from the `execute_instructions` outer dispatch
    /// loop when a runtime error escapes all try frames.  Converts the
    /// error to a catchable value and, if a finally frame is still
    /// active, routes through it; returns the finally IP.  No-op
    /// (`None`) when no finally is active.
    #[inline(never)]
    pub(crate) fn handle_err_finally(&mut self, err: &CompileError) -> Option<usize> {
        let err_val = Self::runtime_error_to_value(err);
        self.route_throw_through_finally(err_val)
    }

    /// Gap 2 — invoked from the `execute_instructions` outer dispatch
    /// loop when a function-scope `Return` step fires.  Returns the IP
    /// of the enclosing finally body (after stashing the pending return
    /// value) if one is active, otherwise `None`.
    ///
    /// Marked `#[inline(never)]` and takes a single simple type so its
    /// locals stay out of `execute_instructions`' recursive stack frame.
    #[inline]
    pub(crate) fn handle_return_finally(&mut self) -> Option<usize> {
        if self.finally_frames.is_empty() {
            return None;
        }
        let (finally_ip, _id, _ld) = self.finally_frames.pop()?;
        // E2: the loop driver already stashed the return value in
        // last_return; save it so FinallyExit can restore it.
        let return_val = self.last_return;
        self.pending_flow = Some(PendingFlow::Return(Box::new(return_val)));
        Some(finally_ip)
    }

    /// Gap 2 — save try/finally state for later restoration across a
    /// function-call boundary.  Returning a `Box<_>` keeps the caller's
    /// stack frame at 8 bytes (pointer) instead of inlining the full
    /// Vecs and Option — critical for mutual-recursion tests whose 2 MB
    /// thread stack is tight.
    ///
    /// PERF-41a: hot-path gate. When try_frames and finally_frames are
    /// both empty AND pending_flow is None (100% of the time for fib-
    /// style pure recursive calls), return None — avoids a Box alloc
    /// and three mem::take operations on every call.
    #[inline(always)]
    pub(crate) fn save_finally_state(&mut self) -> Option<Box<SavedFinally>> {
        if self.try_frames.is_empty()
            && self.finally_frames.is_empty()
            && self.pending_flow.is_none()
        {
            return None;
        }
        Some(Box::new(SavedFinally {
            try_frames: std::mem::take(&mut self.try_frames),
            finally_frames: std::mem::take(&mut self.finally_frames),
            pending_flow: self.pending_flow.take(),
        }))
    }

    /// Gap 2 — restore try/finally state captured by
    /// `save_finally_state`.
    ///
    /// PERF-41a: No-op when `saved` is None (hot path — nothing was
    /// saved because caller's try/finally state was already empty).
    #[inline(always)]
    pub(crate) fn restore_finally_state(&mut self, saved: Option<Box<SavedFinally>>) {
        let Some(saved) = saved else { return };
        let SavedFinally {
            try_frames,
            finally_frames,
            pending_flow,
        } = *saved;
        self.try_frames = try_frames;
        self.finally_frames = finally_frames;
        self.pending_flow = pending_flow;
    }

    /// Gap 1 — extracted body of `Instruction::CallSpread`.
    /// Marked `#[inline(never)]` so the argument-flattening Vec stays
    /// out of the recursive `execute_instructions` frame.
    #[inline(never)]
    pub(crate) fn exec_call_spread(
        &mut self,
        name_sym: hudhudscript_bytecode::SymId,
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<()> {
        let args_val = self.registers[255];
        let args_arr = match args_val.as_array() {
            Some(a) => a,
            _ => {
                return Err(compile_codes::runtime_error(format!(
                    "CallSpread expects Array on stack, got {}",
                    Self::bytecode_value_type_name(&args_val)
                )))
            }
        };
        // `Call`'s arg count is u8-encoded; reject loudly rather than
        // silently truncate on the vanishingly rare >255-arg case.
        if args_arr.len() > u8::MAX as usize {
            return Err(compile_codes::runtime_error(format!(
                "spread call overflows u8 arg count: {}",
                args_arr.len()
            )));
        }
        let argc = args_arr.len() as u8;
        // Register-based: write spread args to sequential registers
        let first_arg = 1u8; // Use register 1 as base for spread args
        for (i, v) in args_arr.iter().enumerate() {
            self.registers[first_arg as usize + i] = v.clone();
        }
        self.exec_call(name_sym, u32::MAX, argc, first_arg, 255, bytecode, ip)
    }

    /// Gap 1 — unified dispatcher for `CallSpread` / `MethodCallSpread`
    /// so the main match only needs one arm.
    #[inline(never)]
    pub(crate) fn exec_spread_call(
        &mut self,
        instr: &Instruction,
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<()> {
        match instr {
            Instruction::CallSpread(name_sym) => self.exec_call_spread(*name_sym, bytecode, ip),
            Instruction::MethodCallSpread(method_sym) => {
                self.exec_method_call_spread(*method_sym, bytecode)
            }
            _ => Err(compile_codes::runtime_error(
                "exec_spread_call: unexpected instruction".to_string(),
            )),
        }
    }

    /// Gap 1 — extracted body of `Instruction::SpreadIntoObject`
    /// (object-literal spread `{...a}`).
    #[inline(never)]
    pub(crate) fn exec_spread_into_object(&mut self) -> CompileResult<()> {
        let src = self.registers[255];
        let dest = self.registers[255];
        let mut dest_map: hudhudscript_bytecode::ObjMap = match dest.as_object() {
            Some(m) => m.clone(),
            _ => {
                return Err(compile_codes::runtime_error(format!(
                    "SpreadIntoObject expects Object on stack, got {}",
                    Self::bytecode_value_type_name(&dest)
                )))
            }
        };
        if let Some(m) = src.as_object() {
            for (k, v) in m {
                dest_map.insert(k.clone(), v.clone());
            }
        } else if let Some(inst) = src.as_instance_data() {
            for (k, v) in &inst.fields {
                dest_map.insert(k.clone(), v.clone());
            }
        }
        // Non-object sources: silently no-op (interpreter parity —
        // `{...arr}` was a runtime no-op).
        self.registers[255] = Value16::object(dest_map);

        Ok(())
    }

    /// Gap 1 — extracted body of `Instruction::MethodCallSpread`.
    #[inline(never)]
    pub(crate) fn exec_method_call_spread(
        &mut self,
        method_sym: hudhudscript_bytecode::SymId,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let receiver = self.registers[255];
        let args_val = self.registers[255];
        let args = match args_val.as_array() {
            Some(a) => a.clone(),
            None => {
                return Err(compile_codes::runtime_error(format!(
                    "MethodCallSpread expects Array on stack, got {}",
                    Self::bytecode_value_type_name(&args_val)
                )))
            }
        };
        let method = bytecode.resolve_symbol(method_sym.0);
        self.last_instance_mutation = None;
        let result = self.call_method_on_value(&receiver, &method, args, bytecode)?;
        self.registers[255] = result;

        Ok(())
    }

    /// Gap 2 — extracted body of `Instruction::FinallyBegin`.  Each
    /// finally frame records a 5-tuple (ip + four depth ints) which
    /// sums to 40 bytes; inlining the push into `execute_instructions`
    /// inflated its recursive frame enough to overflow the 2 MB test
    /// thread stack on 10-deep mutual recursion.
    #[inline(never)]
    pub(crate) fn exec_push_finally(&mut self, finally_target: usize) {
        self.finally_frames.push((
            finally_target,
            self.iterators.len(),
            self.loop_headers.len(),
        ));
    }

    /// Gap 2 — unified dispatcher for the three finally instructions
    /// (`FinallyBegin`, `FinallyEnd`, `FinallyExit`).  Combining them into
    /// a single `#[inline(never)]` helper keeps the recursive
    /// `execute_instructions` frame small: three separate match arms
    /// were enough to overflow mutual-recursion tests on the 2 MB test
    /// thread stack.
    #[inline(never)]
    pub(crate) fn exec_finally_instr(
        &mut self,
        instr: &Instruction,
        ip: usize,
    ) -> CompileResult<FinallyStep> {
        match instr {
            Instruction::FinallyBegin(finally_offset) => {
                // Relative → absolute (Audit v3 F4.2).
                let abs = (ip as i64).wrapping_add(*finally_offset as i64);
                if abs < 0 {
                    return Err(compile_codes::runtime_error(format!(
                        "FinallyBegin out of bounds: ip={} offset={} → {}",
                        ip, finally_offset, abs
                    )));
                }
                self.exec_push_finally(abs as usize);
                Ok(FinallyStep::Advance)
            }
            Instruction::FinallyEnd => {
                self.finally_frames.pop();
                Ok(FinallyStep::Advance)
            }
            Instruction::FinallyExit(after_offset) => {
                let abs = (ip as i64).wrapping_add(*after_offset as i64);
                if abs < 0 {
                    return Err(compile_codes::runtime_error(format!(
                        "FinallyExit out of bounds: ip={} offset={} → {}",
                        ip, after_offset, abs
                    )));
                }
                match self.exec_end_finally(abs as usize)? {
                    EndFinallyAction::Jump(t) => Ok(FinallyStep::Jump(t)),
                    EndFinallyAction::Return => Ok(FinallyStep::Return),
                }
            }
            _ => Err(compile_codes::runtime_error(
                "exec_finally_instr: unexpected instruction".to_string(),
            )),
        }
    }

    /// Gap 2 — extracted body of the `FinallyExit` instruction.  Marked
    /// `#[inline(never)]` so its locals (notably the throw unwinding
    /// path) don't inflate the recursive `execute_instructions` frame
    /// beyond the 2 MB test thread stack budget.
    #[inline(never)]
    pub(crate) fn exec_end_finally(&mut self, after_ip: usize) -> CompileResult<EndFinallyAction> {
        match self.pending_flow.take() {
            Some(PendingFlow::Return(v)) => {
                self.registers[255] = *v;

                Ok(EndFinallyAction::Return)
            }
            Some(PendingFlow::Throw(v)) => {
                let thrown = *v;
                // Re-raise: next try-frame → catch; else next finally →
                // propagate; else uncaught.
                if let Some((catch_ip, iter_depth, loop_depth)) = self.try_frames.pop() {
                    // Register-based VM: no stack truncation.
                    self.iterators.truncate(iter_depth);
                    self.iterator_generators.truncate(iter_depth);
                    self.loop_headers.truncate(loop_depth);
                    self.registers[255] = thrown;

                    return Ok(EndFinallyAction::Jump(catch_ip));
                }
                if let Some(finally_ip) = self.route_throw_through_finally(thrown.clone()) {
                    return Ok(EndFinallyAction::Jump(finally_ip));
                }
                Err(compile_codes::runtime_error(format!(
                    "Uncaught exception: {}",
                    self.value_to_string_inner(&thrown)
                )))
            }
            None => Ok(EndFinallyAction::Jump(after_ip)),
        }
    }

    /// Issue #661 — route a runtime error through the attached
    /// debugger's `on_exception` hook, blocking while paused.
    ///
    /// No-op when no debugger is attached, or when
    /// `break_on_all_exceptions` is off AND no matching exception
    /// breakpoint is registered (the hook itself returns `false`).
    ///
    /// The error type string is `"RuntimeError"` — the VM currently
    /// uses a single `ErrorCode::CompileRuntimeError` for every
    /// runtime failure (see `runtime_error_to_value`); a richer
    /// taxonomy can plug in here later without changing callers.
    pub(crate) fn notify_debugger_exception(&mut self, err: &CompileError) {
        if self.debugger.is_none() {
            return;
        }
        let msg = format!("{}", err);
        let should_pause = if let Some(ref mut dbg) = self.debugger {
            dbg.on_exception("RuntimeError", &msg)
        } else {
            false
        };
        if should_pause {
            loop {
                if let Some(ref dbg) = self.debugger {
                    if dbg.state() != hudhudscript_debug::DebugState::Paused {
                        break;
                    }
                }
                std::thread::yield_now();
            }
        }
    }
}
