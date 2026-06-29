use crate::vm::execute::StepContext;
use crate::vm::util::evaluate_condition_static;
use crate::vm::VM;
use crate::vm::{EndFinallyAction, GenStep, StepAction};
use hudhudscript_bytecode::error::{compile_codes, CompileResult, SourcePosition};
use hudhudscript_bytecode::{Bytecode, Instruction, Value16};

impl crate::vm::VM {
    #[inline(never)]
    pub(crate) fn execute_instructions(
        &mut self,
        instructions: &[Instruction],
        constants: &[Value16],
        bytecode: &Bytecode,
        packed: &[u32],
        chunk_source_positions: Option<&[Option<(usize, usize)>]>,
        ip: &mut usize,
    ) -> CompileResult<bool> {
        let mut local_ip = *ip;
        let mut hit_return = false;

        // CROSS-1 (TCO): snapshot function-frame entry depths so
        // `StepAction::TailCall` can unwind any try / finally / loop
        // frames pushed by the running body before jumping back to
        // ip = 0.  Captured once at entry — the unwind loop restores
        // to EXACTLY these depths on every tail call.
        let tco_entry_try = self.try_frames.len();
        let tco_entry_finally = self.finally_frames.len();
        let tco_entry_loops = self.loop_headers.len();

        // PERF0008: pre-check fuel once; loop-internal counter avoids
        // fuel_limit.is_some() per instruction (dead when fuel is None).
        let has_fuel = self.fuel_limit.is_some();
        let mut fuel_left = self.fuel_remaining;

        // P5/G7: GC safepoint sayacı. Her 1024 instruction'da bir bayrak kontrolü.
        let mut gc_safepoint_tick: u32 = 0;

        // WI-2: Unrestricted mode — skip all per-instruction guards when
        // fuel, debugger, deadline, and cancellation are all disabled.
        let unrestricted = !has_fuel
            && self.debugger.is_none()
            && self.execution_deadline.is_none()
            && !self
                .cancellation_token
                .load(std::sync::atomic::Ordering::Relaxed);

        while local_ip < instructions.len() {
            // GC SAFEPOINT (G7): instruction sınırı — canlı &DynamicObject borrow YOK.
            // alloc() sadece GC_PENDING kaldırır; collect YALNIZ buradan çağrılır.
            gc_safepoint_tick = gc_safepoint_tick.wrapping_add(1);
            if gc_safepoint_tick & 1023 == 0 && hudhudscript_bytecode::gc::safepoint_due() {
                hudhudscript_bytecode::gc::collect(self);
            }
            // ── Guarded checks (skipped in unrestricted mode) ─────
            if !unrestricted {
                if has_fuel {
                    if fuel_left == 0 {
                        return Err(compile_codes::runtime_error(
                            "Out of gas: execution fuel exhausted".to_string(),
                        ));
                    }
                    fuel_left -= 1;
                }

                // PERF-24: cancellation / deadline / stack-overflow all fire
                // every 1024 instructions.  Each instruction can push at most a
                // handful of stack slots, so 1024-op drift against MAX_STACK_SIZE
                // (1_000_000) is negligible.  Pulling the stack check out of the
                // per-instruction path removes a load + compare from the hot loop.
                if local_ip & 0x7FF == 0 {
                    if self
                        .cancellation_token
                        .load(std::sync::atomic::Ordering::Relaxed)
                    {
                        return Err(compile_codes::runtime_error(
                            "Execution cancelled".to_string(),
                        ));
                    }
                    if let Some(deadline) = self.execution_deadline {
                        if std::time::Instant::now() >= deadline {
                            self.suspended_ip = Some(local_ip);
                            return Err(compile_codes::runtime_error(
                                "Execution time limit reached".to_string(),
                            ));
                        }
                    }
                    // Stack overflow check removed — register-based VM
                }

                // Issue #661: Debugger hook — check breakpoints and step mode
                if self.debugger.is_some() {
                    let pos = match chunk_source_positions {
                        Some(sp) => sp.get(local_ip).copied().flatten(),
                        None => bytecode.get_source_position(local_ip),
                    };
                    if let Some((line, _col)) = pos {
                        let file_ref = self
                            .current_file
                            .as_deref()
                            .unwrap_or("<bytecode>")
                            .to_string();
                        let should_pause = if let Some(ref mut dbg) = self.debugger {
                            dbg.on_statement_with_eval(&file_ref, line, |cond| {
                                evaluate_condition_static(cond)
                            })
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
            } // WI-2: end guarded checks (unrestricted skips all above)

            // E1/E3: Tek execution yolu — packed + inline hot + dispatch_unpacked
            // hepsi `step_one`'da.  `execute_instructions` sadece loop driver.
            match self.step_one(instructions, constants, bytecode, packed, &mut local_ip) {
                Ok(StepAction::Advance) => {
                    local_ip += 1;
                }
                Ok(StepAction::Jumped) => {}
                Ok(StepAction::Return { src }) => {
                    self.last_return = self.registers[src as usize];
                    if !self.finally_frames.is_empty() {
                        if let Some(target_ip) = self.handle_return_finally() {
                            local_ip = target_ip;
                            continue;
                        }
                    }
                    hit_return = true;
                    break;
                }
                Ok(StepAction::Call {
                    func_sym,
                    function_idx,
                    arg_count,
                    first_arg,
                    dst,
                    ip,
                }) => {
                    local_ip += 1;
                    self.pending_call = Some((func_sym, function_idx, arg_count, first_arg, dst, ip));
                    hit_return = false;
                    break;
                }
                Ok(StepAction::Break) => {
                    break;
                }
                Ok(StepAction::TailCall) => {
                    let args = self.tco_args.take().unwrap_or_default();
                    while self.try_frames.len() > tco_entry_try {
                        self.try_frames.pop();
                    }
                    while self.finally_frames.len() > tco_entry_finally {
                        self.finally_frames.pop();
                    }
                    while self.loop_headers.len() > tco_entry_loops {
                        self.loop_headers.pop();
                    }
                    for (i, v) in args.iter().cloned().enumerate() {
                        self.registers[i] = v;
                    }
                    self.args_scratch = args;
                    local_ip = 0;
                    continue;
                }
                Err(e) => {
                    if let Some(catch_ip) = self.try_catch_runtime_error(&e) {
                        local_ip = catch_ip;
                    } else if let Some(finally_ip) = self.handle_err_finally(&e) {
                        local_ip = finally_ip;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // PERF0008: sync local fuel counter back to VM for remaining_fuel() API
        self.fuel_remaining = fuel_left;
        *ip = local_ip;
        Ok(hit_return)
    }
}
