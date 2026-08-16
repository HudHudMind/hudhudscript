use crate::vm::call_state::{ContinuationResume, ReturnSink};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::Bytecode;

#[cfg(test)]
thread_local! {
    static DRIVER_ENTRY_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

impl VM {
    #[cfg(test)]
    pub(crate) fn reset_driver_entry_count_for_test() {
        DRIVER_ENTRY_COUNT.with(|count| count.set(0));
    }

    #[cfg(test)]
    pub(crate) fn driver_entry_count_for_test() -> usize {
        DRIVER_ENTRY_COUNT.with(std::cell::Cell::get)
    }

    #[inline]
    fn push_pending_vm_call(&mut self, bytecode: &Bytecode) -> CompileResult<bool> {
        let Some(request) = self.pending_vm_call.take() else {
            return Ok(false);
        };
        self.exec_vm_call_push_frame(request, bytecode)?;
        Ok(true)
    }

    #[inline]
    fn push_pending_direct_call(&mut self, bytecode: &Bytecode) -> CompileResult<bool> {
        let Some((func_sym, function_idx, arg_count, first_arg, dst, call_ip)) =
            self.pending_call.take()
        else {
            return Ok(false);
        };

        let frame_count_before = self.frame_stack.len();
        let is_super = std::mem::take(&mut self.pending_super_call);
        self.exec_call(
            func_sym,
            function_idx,
            arg_count,
            first_arg,
            dst,
            bytecode,
            call_ip,
        )?;

        if self.frame_stack.len() > frame_count_before {
            if is_super {
                if let Some(frame) = self.frame_stack.last_mut() {
                    frame.class_context = true;
                }
            }
        } else if self.pending_vm_call.is_none() {
            self.registers[dst as usize] = self.registers[255];
        }
        Ok(true)
    }

    fn deliver_return(
        &mut self,
        sink: ReturnSink,
        value: hudhudscript_bytecode::Value16,
    ) -> CompileResult<()> {
        match sink {
            ReturnSink::Register(dst) => self.registers[dst as usize] = value,
            ReturnSink::Continuation(id) => match self.resume_continuation(id, value)? {
                ContinuationResume::Schedule(request) => self.schedule_vm_call(request)?,
                ContinuationResume::Complete { dst, value } => {
                    self.registers[dst as usize] = value;
                }
                ContinuationResume::Discard => {}
            },
            ReturnSink::Discard => {}
        }
        Ok(())
    }

    /// Run the canonical frame driver until every frame created after
    /// `stop_depth` has returned. User chunks never open another driver from
    /// inside this loop; they schedule a request and yield control here.
    pub(crate) fn run_frame_loop(
        &mut self,
        bytecode: &Bytecode,
        main_packed: &[u32],
        stop_depth: usize,
    ) -> CompileResult<bool> {
        #[cfg(test)]
        DRIVER_ENTRY_COUNT.with(|count| count.set(count.get() + 1));
        self.frame_stack.reserve(64);
        let mut returned = false;

        let result = 'outer: loop {
            // Supports host entry with a pre-scheduled request and continuation
            // chains that temporarily have no active callee frame.
            if let Err(error) = self.push_pending_vm_call(bytecode) {
                self.unwind_driver_frames_on_error(stop_depth);
                break 'outer Err(error);
            }

            if self.frame_stack.len() <= stop_depth && self.pending_flow.is_none() {
                break 'outer Ok(());
            }
            let frame_idx = self.frame_stack.len().saturating_sub(1);
            let (instructions, constants, packed_slice, chunk_sp, mut ip) = {
                let frame = &self.frame_stack[frame_idx];
                if frame.chunk_ptr.is_null() {
                    (
                        &bytecode.instructions,
                        &bytecode.constants,
                        main_packed,
                        None,
                        frame.ip,
                    )
                } else {
                    let chunk = unsafe { &*frame.chunk_ptr };
                    let packed_slice = unsafe { &*frame.packed } as &[u32];
                    let chunk_sp = if chunk.source_positions.is_empty() {
                        None
                    } else {
                        Some(&chunk.source_positions[..])
                    };
                    (
                        &chunk.instructions,
                        &chunk.constants,
                        packed_slice,
                        chunk_sp,
                        frame.ip,
                    )
                }
            };

            self.current_chunk_ptr = self.frame_stack[frame_idx].chunk_ptr;
            match self.execute_instructions(
                instructions,
                constants,
                bytecode,
                packed_slice,
                chunk_sp,
                &mut ip,
            ) {
                Ok(hit_return) => {
                    self.frame_stack[frame_idx].ip = ip;

                    // Driver priority is contractual: VM request, direct call,
                    // return delivery, then throw/finally propagation.
                    let dispatched = self.push_pending_vm_call(bytecode).and_then(|pushed| {
                        if pushed {
                            return Ok(true);
                        }
                        self.push_pending_direct_call(bytecode)
                    });
                    match dispatched {
                        Ok(true) => continue 'outer,
                        Ok(false) => {}
                        Err(error) => {
                            self.unwind_driver_frames_on_error(stop_depth);
                            break 'outer Err(error);
                        }
                    }
                    if hit_return {
                        returned = true;
                        let value = self.last_return;
                        let frame = self.frame_stack.pop().expect("active frame must exist");
                        let sink = frame.return_sink;
                        self.teardown_frame(frame);
                        self.deliver_return(sink, value)?;

                        if self.pending_vm_call.is_some() {
                            continue 'outer;
                        }
                        if self.frame_stack.len() <= stop_depth {
                            self.registers[255] = value;
                            break 'outer Ok(());
                        }
                    } else if let Some(crate::vm::PendingFlow::Throw(thrown)) =
                        self.pending_flow.take()
                    {
                        if self.frame_stack.len() > stop_depth + 1 {
                            let frame = self.frame_stack.pop().expect("throwing frame must exist");
                            let sink = frame.return_sink;
                            self.teardown_frame(frame);
                            // G07-14: a dying frame's continuation is
                            // cancelled — never resumed with the thrown value.
                            self.cancel_frame_continuation(sink);
                            self.pending_flow = Some(crate::vm::PendingFlow::Throw(thrown));
                        } else {
                            return Err(compile_codes::runtime_error(format!(
                                "Uncaught exception: {}",
                                crate::vm::exception_value::exception_field_str(
                                    &*thrown,
                                    "description"
                                )
                            )));
                        }
                    } else {
                        break 'outer Ok(());
                    }
                }
                Err(error) => {
                    if self.unwind_swallowed_error(stop_depth)? {
                        continue 'outer;
                    }
                    if self.unwind_runtime_error_to_caller(&error, stop_depth) {
                        continue 'outer;
                    }
                    self.unwind_driver_frames_on_error(stop_depth);
                    break 'outer Err(error);
                }
            }
        };

        result.map(|_| returned)
    }

    /// G07-15: route a runtime error to the nearest caller-side catch or
    /// finally instead of stranding callee frames. Each popped frame's
    /// continuation sink is cancelled. `teardown_frame` restores the
    /// caller's saved try/finally state, so its handlers are visible here.
    #[cold]
    #[inline(never)]
    fn unwind_runtime_error_to_caller(
        &mut self,
        error: &hudhudscript_errors::Error,
        stop_depth: usize,
    ) -> bool {
        while self.frame_stack.len() > stop_depth + 1 {
            let frame = self.frame_stack.pop().expect("throwing frame must exist");
            let sink = frame.return_sink;
            self.teardown_frame(frame);
            self.cancel_frame_continuation(sink);

            if let Some(catch_ip) = self.try_catch_runtime_error(error) {
                if let Some(caller) = self.frame_stack.last_mut() {
                    caller.ip = catch_ip;
                }
                return true;
            }
            if let Some(finally_ip) = self.handle_err_finally(error) {
                if let Some(caller) = self.frame_stack.last_mut() {
                    caller.ip = finally_ip;
                }
                return true;
            }
        }
        false
    }

    /// G07-14: a runtime error kills every frame this driver owns. Each
    /// frame's continuation sink is cancelled — never resumed — so STM
    /// transaction context is cleared and no pending call leaks.
    #[cold]
    #[inline(never)]
    fn unwind_driver_frames_on_error(&mut self, stop_depth: usize) {
        self.pending_vm_call = None;
        while self.frame_stack.len() > stop_depth {
            let frame = self.frame_stack.pop().expect("erroring frame must exist");
            let sink = frame.return_sink;
            self.teardown_frame(frame);
            self.cancel_frame_continuation(sink);
        }
    }

    /// SOP effect frames discard body errors instead of unwinding them to
    /// the host. Pops frames down to (and including) the innermost
    /// swallow-marked frame this driver owns, then resumes its sink.
    #[cold]
    #[inline(never)]
    fn unwind_swallowed_error(&mut self, stop_depth: usize) -> CompileResult<bool> {
        let has_target = self.frame_stack[stop_depth.min(self.frame_stack.len())..]
            .iter()
            .any(|frame| frame.swallow_error);
        if !has_target {
            return Ok(false);
        }
        while self.frame_stack.len() > stop_depth {
            let frame = self.frame_stack.pop().expect("swallowing frame must exist");
            let sink = frame.return_sink;
            let swallow = frame.swallow_error;
            self.teardown_frame(frame);
            if swallow {
                self.deliver_return(sink, hudhudscript_bytecode::Value16::null())?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
