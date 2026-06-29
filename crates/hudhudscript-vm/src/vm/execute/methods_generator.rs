#![allow(unused_imports)]

use super::*;
use std::sync::Arc;

impl VM {
    /// Generator instructions: `Yield` and `MakeGenerator`.
    ///
    /// Split out of `methods_async_generator.rs` to keep the latter under the
    /// 400-line source limit.  Both instructions use the `DetachedGraph`
    /// channel model introduced in G2-A2.
    #[inline]
    pub(crate) fn step_methods_generator(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let bytecode = ctx.bytecode;

        match instr {
            // Issue #667: Yield — send value lazily or collect eagerly
            Instruction::Yield { .. } => {
                let val = self.registers[255];
                if let Some(ref sender) = self.yield_sender {
                    // V2-B: detach → sender thread'in heap'inden çıkar
                    let tree = match hudhudscript_bytecode::gc_detach::detach(val) {
                        Ok(t) => t,
                        Err(code) => {
                            eprintln!(
                                "[hudhudscript-vm] generator yield: detach failed ({code:?})"
                            );
                            return Ok(StepAction::Break);
                        }
                    };
                    if sender.send(tree).is_err() {
                        eprintln!("[hudhudscript-vm] generator: consumer dropped, stopping early");
                        return Ok(StepAction::Break);
                    }
                } else {
                    // Eager fallback: collect into __yield_collector
                    if let Some(collector_val) = self.get_var_cloned("__yield_collector") {
                        if let Some(arr) = collector_val.as_array() {
                            let mut new_arr = arr.clone();
                            new_arr.push(val);
                            self.set_var("__yield_collector", Value16::array(new_arr))?;
                        }
                    }
                }
                // Push null back (yield expression evaluates to null)
                self.registers[255] = Value16::null();
            }

            // Issue #667: MakeGenerator — spawn a lazy generator thread
            Instruction::MakeGenerator {
                payload_idx,
                first_arg,
                arg_count,
            } => {
                // CROSS-2c: resolve the call payload from the side table.
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                let func_name_sym = payload.sym;
                let func_name = bytecode.resolve_symbol(func_name_sym.0);
                let n = *arg_count as usize;
                let first = *first_arg as usize;
                let args: Vec<Value16> = (0..n).map(|i| self.registers[first + i]).collect();

                if let Some(chunk) = bytecode.get_function(&func_name) {
                    // Rendezvous channel: sender blocks until receiver calls recv()
                    let (yield_tx, yield_rx) = std::sync::mpsc::sync_channel::<
                        hudhudscript_bytecode::gc_detach::DetachedGraph,
                    >(0);

                    // Snapshot state for the generator thread
                    let chunk_arc = Arc::clone(&chunk);
                    let bytecode_clone = bytecode.clone();
                    let global_scope = self.globals.clone();
                    let classes_clone = self.classes.clone();
                    let declarations_clone = self.declarations.clone();
                    let params_clone: Vec<String> = chunk.params.clone();

                    std::thread::spawn(move || {
                        let mut gen_vm = VM::new();
                        // Share the caller's globals
                        for (k, v) in global_scope {
                            gen_vm.globals.entry(k).or_insert(v);
                        }
                        gen_vm.classes = classes_clone;
                        gen_vm.declarations = declarations_clone;
                        // Install the yield sender so Yield instructions
                        // send through the channel instead of collecting.
                        gen_vm.yield_sender = Some(yield_tx);

                        // Bind parameters.
                        // Gap 1 — rest-param parity (`...rest` bundles
                        // trailing args into an Array, keyed by the
                        // un-dotted name).
                        //
                        // S2.2b: bind params to registers
                        // (and `sym_to_slot` for LoadVar slow-path
                        // compat) so register reads/writes resolve correctly
                        // inside the generator body.
                        let has_rest = params_clone
                            .last()
                            .map(|p| p.starts_with("..."))
                            .unwrap_or(false);
                        let regular_count = if has_rest {
                            params_clone.len() - 1
                        } else {
                            params_clone.len()
                        };

                        // Populate call_stack_local_syms for LoadVar slow-path compat.
                        let mut built: Vec<(u32, usize, Option<usize>)> =
                            Vec::with_capacity(chunk_arc.local_names.len());
                        let mut max_sym: u32 = 0;
                        for (slot, name) in chunk_arc.local_names.iter().enumerate() {
                            let sid = hudhudscript_bytecode::interner::intern(name).0;
                            if sid > max_sym {
                                max_sym = sid;
                            }
                            let param_idx = params_clone.iter().position(|p| {
                                p == name || p.strip_prefix("...").map_or(false, |s| s == name)
                            });
                            built.push((sid, slot, param_idx));
                        }
                        built.sort_by_key(|(sym_id, _, _)| *sym_id);
                        let built_ptr = Box::into_raw(Box::new(built));
                        gen_vm.call_stack_local_syms.push(built_ptr);
                        gen_vm.owned_local_sym_refs.push(built_ptr);

                        // Populate slot values from args for each param
                        // that has a matching slot.
                        for (i, param) in params_clone.iter().enumerate().take(regular_count) {
                            let val = args.get(i).cloned().unwrap_or(Value16::null());
                            if let Some(local_syms_ptr) = gen_vm.call_stack_local_syms.last() {
                                let local_syms = unsafe { &**local_syms_ptr };
                                let sym_id = hudhudscript_bytecode::interner::intern(param).0;
                                if let Ok(idx) =
                                    local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s)
                                {
                                    let slot = local_syms[idx].1 as i32;
                                    if slot >= 0 {
                                        gen_vm.registers[slot as usize] = val;
                                        continue;
                                    }
                                }
                            }
                            // Fallback: no slot (shouldn't happen for
                            // params).  Globals would be the fallback, but
                            // params are always slot-allocated.
                        }
                        if has_rest {
                            let rest_name = params_clone[regular_count]
                                .trim_start_matches('.')
                                .to_string();
                            let rest_values: Vec<Value16> = if args.len() > regular_count {
                                args[regular_count..].to_vec()
                            } else {
                                Vec::new()
                            };
                            let rest_val = Value16::array(rest_values);
                            let mut stored = false;
                            if let Some(local_syms_ptr) = gen_vm.call_stack_local_syms.last() {
                                let local_syms = unsafe { &**local_syms_ptr };
                                let sym_id = hudhudscript_bytecode::interner::intern(&rest_name).0;
                                if let Ok(idx) =
                                    local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s)
                                {
                                    let slot = local_syms[idx].1 as i32;
                                    if slot >= 0 {
                                        gen_vm.registers[slot as usize] = rest_val;
                                        stored = true;
                                    }
                                }
                            }
                            if !stored {
                                let sym = hudhudscript_bytecode::interner::intern(&rest_name);
                                gen_vm.globals.insert(sym, rest_val);
                            }
                        }

                        // Execute the generator body; when it finishes (or
                        // errors) the sender is dropped, signalling "done".
                        let _ = gen_vm.execute_chunk(&chunk_arc, &bytecode_clone);
                        // gen_vm (and yield_tx) dropped here → recv returns Err → done
                    });

                    let yield_id = self.next_yield_id;
                    self.next_yield_id += 1;
                    self.yield_receivers.insert(yield_id, yield_rx);
                    let (_dummy_tx, dummy_rx) = std::sync::mpsc::sync_channel::<Value16>(0);
                    let mut state = GeneratorState16::new(dummy_rx);
                    state.yield_id = Some(yield_id);
                    self.registers[255] =
                        Value16::generator(Arc::new(parking_lot::Mutex::new(state)));
                } else {
                    // Function not found — push an empty, exhausted generator
                    let (_tx, rx) = std::sync::mpsc::sync_channel::<Value16>(0);
                    self.registers[255] = Value16::generator(Arc::new(parking_lot::Mutex::new(
                        GeneratorState16::new(rx),
                    )));
                }
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
