#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_control_flow(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;

        match instr {
            Instruction::EnumDecl(payload_idx) => {
                // CROSS-2a: payload lives in `bytecode.enum_decl_payloads`.
                let payload = &bytecode.enum_decl_payloads[*payload_idx as usize];
                let name = bytecode.resolve_symbol(payload.name.0);
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                for variant_sym in &payload.variants {
                    let variant = bytecode.resolve_symbol(variant_sym.0);
                    obj.insert(
                        variant.clone(),
                        Value16::string(format!("{}::{}", name, variant)),
                    );
                }
                self.set_var(&name, Value16::object(obj))?;
            }

            Instruction::MatchVariant(payload_idx) => {
                // CROSS-2d: two-sym payload — first = enum symbol,
                // second = variant symbol.
                let payload = bytecode.get_two_sym_payload(*payload_idx as u32);
                let enum_name_sym = hudhudscript_bytecode::SymId(payload.first);
                let variant_sym = hudhudscript_bytecode::SymId(payload.second);
                let top = self.registers[255];
                let expected = format!(
                    "{}::{}",
                    bytecode.resolve_symbol(enum_name_sym.0),
                    bytecode.resolve_symbol(variant_sym.0)
                );
                let matches = top.as_string().map(|s| s == expected).unwrap_or(false);
                self.registers[255] = Value16::bool_(matches);
            }

            Instruction::BindVar(name_sym) => {
                let name = bytecode.resolve_symbol(name_sym.0);
                let value = self.registers[255];
                self.set_var(&name, value)?;
            }

            // ── New instructions in function bodies ──────────────────
            Instruction::Break => {
                // Break out of the innermost loop
                let exit_ip_opt = self.loop_headers.last().map(|(_, exit_ip)| *exit_ip);
                if let Some(exit_ip) = exit_ip_opt {
                    if exit_ip != usize::MAX {
                        // Clean up loop state
                        if !self.iterators.is_empty() {
                            self.iterators.pop();
                            self.iterator_generators.pop();
                        }
                        self.loop_headers.pop();
                        *ip_ref = exit_ip;
                        return Ok(StepAction::Jumped);
                    }
                }
                // Fallback: break out of VM execution loop (wrong but safe)
                return Ok(StepAction::Break);
            }
            Instruction::Continue => {
                // Continue to the next iteration of the innermost loop
                if let Some(&(header_ip, _)) = self.loop_headers.last() {
                    if header_ip != usize::MAX {
                        // Pop the loop header since LoopBegin will re-push it
                        // when we jump back to the loop start.
                        // For ForIn loops (header_ip set by IterNext), jump to
                        // the IterNext instruction directly without popping.
                        // For While loops (LoopBegin-based), pop so LoopBegin re-pushes.
                        // Detect: ForIn loops use iterators; While loops don't.
                        if !self.iterators.is_empty() {
                            // ForIn loop — jump to IterNext (header_ip)
                            *ip_ref = header_ip;
                        } else {
                            // While loop — pop loop header, jump to condition
                            self.loop_headers.pop();
                            *ip_ref = header_ip;
                        }
                        return Ok(StepAction::Jumped);
                    }
                }
                // If no valid loop header, treat as break (fallback)
                return Ok(StepAction::Break);
            }

            Instruction::LoopBegin(idx) => {
                #[cfg(feature = "telemetry")]
                {
                    self.telemetry.loop_begin_end_count += 1;
                }
                // CROSS-2b: resolve payload from the side table.  The
                // compiler is required to emit a valid index; out-of-bounds
                // indicates a compiler bug (Kural 7c — no fallback).
                let payload = bytecode.get_loop_payload(*idx);
                self.loop_headers
                    .push((payload.start as usize, payload.end as usize));
            }
            Instruction::LoopEnd => {
                #[cfg(feature = "telemetry")]
                {
                    self.telemetry.loop_begin_end_count += 1;
                }
                self.loop_headers.pop();
            }

            Instruction::ForIn {
                iter_reg,
                var_sym_idx: var_name_sym,
                ..
            } => {
                let var_name = bytecode.resolve_symbol(*var_name_sym as u32);
                let iterable = self.registers[*iter_reg as usize];
                // Fast path: iterables that fit an eager Vec.
                // Lazy path: generators are consumed one `advance()` at a time
                // (see `iterator_generators`) — important for infinite or
                // side-effecting generators (#P2-7).
                let (elements, generator) = if let Some(arr) = iterable.as_array() {
                    (arr.iter().cloned().collect(), None)
                } else if let Some(obj) = iterable.as_object() {
                    let has_next = obj.get("next").map_or(false, |v| {
                        v.as_string().is_some() || v.as_function_data().is_some()
                    });
                    if has_next {
                        self.start_custom_iterator_sequence(
                            iterable,
                            var_name.clone(),
                            bytecode,
                            255,
                            ip,
                        )?;
                        return Ok(StepAction::DeferredCall);
                    }
                    (
                        obj.keys().map(|k| Value16::string(k.to_string())).collect(),
                        None,
                    )
                } else if let Some(s) = iterable.as_string() {
                    (
                        s.chars().map(|c| Value16::string(c.to_string())).collect(),
                        None,
                    )
                } else if let Some(items) = iterable.as_set() {
                    (items.iter().cloned().collect(), None)
                } else if let Some(pairs) = iterable.as_map_pairs() {
                    let mapped = pairs
                        .iter()
                        .map(|(k, v)| Value16::array(vec![k.clone(), v.clone()]))
                        .collect();
                    (mapped, None)
                } else if let Some(state) = iterable.as_generator_state() {
                    (Vec::new(), Some(state.clone()))
                } else if iterable.as_instance_data().is_some() {
                    self.start_custom_iterator_sequence(
                        iterable,
                        var_name.clone(),
                        bytecode,
                        255,
                        ip,
                    )?;
                    return Ok(StepAction::DeferredCall);
                } else {
                    return Err(compile_codes::runtime_error("Cannot iterate".to_string()));
                };
                self.iterators.push((elements, var_name.clone(), 0));
                self.iterator_generators.push(generator);
                self.loop_headers.push((usize::MAX, usize::MAX));
            }

            Instruction::IterNext {
                end_offset: exit_offset,
                ..
            } => {
                // Relative offset → absolute exit IP (Audit v3 F4.2).
                let exit_abs = (ip as i64).wrapping_add(*exit_offset as i64);
                if exit_abs < 0 || exit_abs > instructions.len() as i64 {
                    return Err(compile_codes::runtime_error(format!(
                        "IterNext exit out of bounds: ip={} offset={} → {}",
                        ip, exit_offset, exit_abs
                    )));
                }
                let exit_target = exit_abs as usize;

                // Update loop header and exit IP if not set yet
                if let Some((header_ip, exit_ip)) = self.loop_headers.last_mut() {
                    if *header_ip == usize::MAX {
                        *header_ip = ip;
                    }
                    if *exit_ip == usize::MAX {
                        *exit_ip = exit_target;
                    }
                }

                // #P2-7: Generator frames pull one value per IterNext
                // via `advance()`, so side-effecting or infinite
                // generators iterate correctly (interpreter parity).
                // Dispatch via a helper to keep this frame small in
                // debug builds where Value enum bloats frame size.
                let is_gen = matches!(self.iterator_generators.last(), Some(Some(_)));

                if is_gen {
                    let step = self.step_generator_iter();
                    match step {
                        GenStep::Advanced => {}
                        GenStep::Exhausted => {
                            self.iterators.pop();
                            self.iterator_generators.pop();
                            self.loop_headers.pop();
                            *ip_ref = exit_target;
                            return Ok(StepAction::Jumped);
                        }
                    }
                } else if let Some((elements, var_name, index)) = self.iterators.last_mut() {
                    if *index >= elements.len() {
                        // Iteration done — pop iterator and loop header
                        self.iterators.pop();
                        self.iterator_generators.pop();
                        self.loop_headers.pop();
                        *ip_ref = exit_target;
                        return Ok(StepAction::Jumped);
                    } else {
                        // Issue #1021: Move element out of iterator storage instead of cloning
                        let elem = std::mem::replace(&mut elements[*index], Value16::null());
                        *index += 1;
                        // S2.2d.2: slot is authoritative for iter var
                        // when the compiler allocated one (function-body
                        // for-loops after S2.2c).
                        let sym_id = hudhudscript_bytecode::interner::intern(var_name.as_str()).0;
                        let mut slot_for_var = None;
                        if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
                            let local_syms = unsafe { &**local_syms_ptr };
                            if let Ok(idx) =
                                local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s)
                            {
                                let slot = local_syms[idx].1 as i32;
                                if slot >= 0 {
                                    slot_for_var = Some(slot);
                                }
                            }
                        }
                        if let Some(slot) = slot_for_var {
                            let idx = slot as usize;
                            if idx < self.registers.len() {
                                self.registers[idx] = elem;
                            }
                        } else {
                            // No slot (top-level for-loop):
                            // globals are authoritative.
                            let sym = hudhudscript_bytecode::interner::intern(&var_name);
                            self.globals.insert(sym, elem);
                            // Mirror to shared_globals_vec if this is a shared top-level symbol.
                            if let Some(encoded) = self.main_slot_encoded(sym.0) {
                                let (_slot, is_shared) = crate::vm::VM::main_slot_decode(encoded);
                                if is_shared {
                                    let idx = crate::vm::VM::main_slot_shared_index(encoded);
                                    if idx < self.shared_globals_vec.len() {
                                        self.shared_globals_vec[idx] = elem;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // No iterator — treat as immediate exit.
                    // exit_target is already the absolute IP (computed above).
                    *ip_ref = exit_target;
                    return Ok(StepAction::Jumped);
                }
            }

            Instruction::TryBegin(catch_offset) => {
                // Relative offset → absolute catch IP (Audit v3 F4.2).
                let catch_abs = (ip as i64).wrapping_add(*catch_offset as i64);
                if catch_abs < 0 || catch_abs > instructions.len() as i64 {
                    return Err(compile_codes::runtime_error(format!(
                        "TryBegin catch out of bounds: ip={} offset={} → {}",
                        ip, catch_offset, catch_abs
                    )));
                }
                // Save stack depth, iterator depth, and loop header depth
                // so Throw can fully restore state (not just the value stack).
                self.try_frames.push((
                    catch_abs as usize,
                    self.iterators.len(),
                    self.loop_headers.len(),
                ));
            }
            Instruction::TryEnd => {
                self.try_frames.pop();
            }
            // Gap 2 — finally instructions all delegate to the same
            // extracted helper to keep this dispatch function's
            // frame unchanged (mutual-recursion tests are tight on
            // the 2 MB test-thread stack).
            Instruction::FinallyBegin(_)
            | Instruction::FinallyEnd
            | Instruction::FinallyExit(_) => match self.exec_finally_instr(instr, ip)? {
                FinallyStep::Advance => {}
                FinallyStep::Jump(t) => {
                    *ip_ref = t;
                    return Ok(StepAction::Jumped);
                }
                FinallyStep::Return => return Ok(StepAction::Return { src: 255 }),
            },
            Instruction::Throw { src } => {
                let thrown = crate::vm::exception_value::normalize_throw_value(
                    self.registers[*src as usize],
                );
                // Issue #661 — explicit `throw` statement fires the
                // debugger's `on_exception` hook BEFORE unwinding so
                // the paused frame still reflects the throwing scope.
                if self.debugger.is_some() {
                    let msg =
                        crate::vm::exception_value::exception_field_str(&thrown, "description");
                    let should_pause = if let Some(ref mut dbg) = self.debugger {
                        dbg.on_exception("Throw", &msg)
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
                // If finally frames exist AND try frames exist, compare positions.
                // Inner finally (lower ip) should run before outer catch (higher ip).
                let has_fin = !self.finally_frames.is_empty();
                let has_try = !self.try_frames.is_empty();
                if has_fin && has_try {
                    let (fin_ip, _, _) = *self.finally_frames.last().unwrap();
                    let (catch_ip, _, _) = *self.try_frames.last().unwrap();
                    if fin_ip < catch_ip {
                        self.pending_flow = Some(crate::vm::PendingFlow::Throw(Box::new(thrown)));
                        *ip_ref = fin_ip;
                        return Ok(StepAction::Jumped);
                    }
                }
                if let Some((catch_ip, iter_depth, loop_depth)) = self.try_frames.pop() {
                    // Restore iterator depth and loop header depth.
                    self.iterators.truncate(iter_depth);
                    self.iterator_generators.truncate(iter_depth);
                    self.loop_headers.truncate(loop_depth);

                    self.registers[255] = thrown;

                    if catch_ip >= instructions.len() {
                        // BUG1: catch target belongs to an enclosing caller chunk.
                        // We cannot jump there from this chunk. Instead, treat this
                        // as a cross-frame abrupt completion: stash the thrown value,
                        // set ip past the end of this chunk, and let the frame loop
                        // pop the current frame and re-dispatch the pending throw in
                        // the caller's instruction stream.
                        self.pending_flow = Some(crate::vm::PendingFlow::Throw(Box::new(thrown)));
                        *ip_ref = instructions.len();
                        return Ok(StepAction::Jumped);
                    }
                    *ip_ref = catch_ip;
                    return Ok(StepAction::Jumped);
                } else {
                    // BUG1: no handler in this chunk. Route through finally
                    // if one exists, otherwise unwind to caller.
                    if let Some(finally_ip) = self.route_throw_through_finally(thrown.clone()) {
                        *ip_ref = finally_ip;
                        return Ok(StepAction::Jumped);
                    }
                    self.pending_flow = Some(crate::vm::PendingFlow::Throw(Box::new(thrown)));
                    *ip_ref = instructions.len();
                    return Ok(StepAction::Jumped);
                }
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
