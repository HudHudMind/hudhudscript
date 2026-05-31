#![allow(unused_imports)]

use super::*;

impl VM {
    #[inline]
    pub(crate) fn step_methods_async_generator(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let ip_ref = &mut *ctx.ip_ref;

        match instr {
            Instruction::MethodCall { dst: 255, obj: 255, payload_idx, first_arg, arg_count } => {
                // CROSS-2c: resolve the call payload from the side table.
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                let method_sym = payload.sym;
                let payload_arg_count = payload.arg_count;
                let method = bytecode.resolve_symbol(method_sym.0);
                let receiver = self.registers[255];
                let first = *first_arg as usize;
                let n = *arg_count as usize;
                let use_regs = *first_arg != 0 || n > 0;
                let mut args: Vec<Value16> = if use_regs {
                    (0..n).map(|i| self.registers[first + i]).collect()
                } else {
                    let mut v = Vec::new();
                    for _ in 0..payload_arg_count {
                        v.push(self.registers[255]);
                    }
                    v.reverse();
                    v
                };

                // Issue #747: Generator.next() — lazy pull from channel
                // Returns just the yielded value (or null when exhausted).
                if method == "next" {
                    if let Some(state) = receiver.as_generator_state() {
                        let next_val = state.lock().advance();
                        self.registers[255] = next_val.unwrap_or(Value16::null());

                        *ip_ref = ip + 1;
                        return Ok(StepAction::Jumped);
                    }
                }

                // Clear the mutation scratch slot so a subsequent
                // WriteBackReceiver only fires for THIS method call
                // (not a leftover from a prior instance method call).
                self.last_instance_mutation = None;
                let result = self.call_method_on_value(&receiver, &method, args, bytecode)?;
                self.registers[255] = result;

            }

            Instruction::WriteBackReceiver(var_sym) => {
                // Flush the `this` mutation captured by the preceding
                // MethodCall back into the receiver variable. No-op
                // when the slot is empty (non-instance receivers,
                // builtin methods, etc.). The method result already
                // sits on top of the stack and is left untouched.
                if let Some(updated) = self.last_instance_mutation.take() {
                    let sym_id = var_sym.0;
                    let mut stored = false;
                    if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
                        let local_syms = unsafe { &**local_syms_ptr };
                        if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                            let slot = local_syms[idx].1 as i32;
                            if slot >= 0 {
                                let local_idx = slot as usize;
                                if local_idx < self.registers.len() {
                                    self.registers[local_idx] = *updated;
                                    stored = true;
                                }
                            }
                        }
                    }
                    if !stored {
                        let name = bytecode.resolve_symbol(sym_id);
                        let _ = self.set_var(&name, *updated);
                    }
                }
            }

            // ── Async/Await (Issue #342) ─────────────────────────────
            Instruction::Await { .. } => {
                // ── #518: reject await inside atomically() ───────
                if self.in_stm_context {
                    return Err(compile_codes::runtime_error(
                        "Cannot use await inside an STM atomically() block".to_string(),
                    ));
                }

                let value = self.registers[255];
                if let Some(ps) = value.as_promise_state() {
                    match ps {
                        hudhudscript_bytecode::PromiseState16::Resolved(inner) => {
                            self.registers[255] = **inner;

                        }
                        hudhudscript_bytecode::PromiseState16::Pending => {
                            return Err(compile_codes::runtime_error(
                                "await on a Pending promise that has no resolver; this usually \
                             means Promise.race/Promise.all was invoked with non-async \
                             promises that never settle, or a promise was constructed \
                             without a backing task"
                                    .to_string(),
                            ));
                        }
                        hudhudscript_bytecode::PromiseState16::Rejected(msg) => {
                            return Err(compile_codes::runtime_error(format!(
                                "Promise rejected: {}",
                                msg
                            )));
                        }
                        hudhudscript_bytecode::PromiseState16::AsyncPending(id) => {
                            if self.promise_registry.has_entry(id) {
                                match self.promise_registry.await_blocking(id) {
                                    Ok(val) => self.registers[255] = val,
                                    Err(hudhudscript_async::RegistryError::Rejected(msg)) => {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Promise rejected: {}",
                                            msg
                                        )));
                                    }
                                    Err(e) => {
                                        return Err(compile_codes::runtime_error(format!("{}", e)));
                                    }
                                }
                            } else {
                                self.registers[255] = Value16::promise(
                                    hudhudscript_bytecode::PromiseState16::AsyncPending(id.clone()),
                                );
                            }
                        }
                    }
                } else {
                    self.registers[255] = value;

                }
            }

            // ── Class extensionsClass extensions (Issue #345) ────────────────────────
            Instruction::SuperCall { dst: 255, payload_idx, first_arg, arg_count } => {
                // Trampoline-aware: resolve parent method, put chunk in call_cache,
                // set pending_call + pending_super_call.
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                let method_name = bytecode.resolve_symbol(payload.sym.0);
                if self.get_var("this").is_none() {
                    return Err(compile_codes::runtime_error(
                        "super() called without 'this' context".to_string(),
                    ));
                }
                let current_class = self
                    .class_context_stack
                    .last()
                    .cloned()
                    .or_else(|| {
                        self.get_var("__current_class")
                            .and_then(|v| v.as_string().map(|s| s.to_string()))
                    })
                    .ok_or_else(|| {
                        compile_codes::runtime_error("super used outside class context".to_string())
                    })?;
                let parent_name = self
                    .classes
                    .get(&current_class)
                    .and_then(|(p, _)| p.clone())
                    .ok_or_else(|| {
                        compile_codes::runtime_error(format!(
                            "Class {} has no parent for super call",
                            current_class
                        ))
                    })?;
                let chunk_name = format!("{}::{}", parent_name, method_name);
                let chunk = bytecode.functions.borrow().get(chunk_name.as_str()).cloned().ok_or_else(|| {
                    compile_codes::runtime_error(format!(
                        "Parent method not found: {}",
                        chunk_name
                    ))
                })?;
                self.class_context_stack.push(parent_name.clone());
                let func_sym = hudhudscript_bytecode::SymId(
                    hudhudscript_bytecode::interner::intern(&chunk_name).0,
                );
                // Put chunk in call_cache so exec_call can find it on the fast path.
                let sym_id = func_sym.0 as usize;
                if self.call_cache.len() <= sym_id {
                    self.call_cache.resize(sym_id + 1, None);
                }
                let params_box = Box::new(chunk.params.clone());
                self.call_cache[sym_id] = Some((Arc::as_ptr(&chunk), Box::into_raw(params_box)));
                self.pending_super_call = true;
                return Ok(StepAction::Call {
                    func_sym,
                    arg_count: payload.arg_count,
                    first_arg: *first_arg,
                    dst: 255,
                });
            }

            Instruction::GetStatic(payload_idx) => {
                // CROSS-2d: two-sym payload — first = class symbol,
                // second = member symbol.
                let payload = bytecode.get_two_sym_payload(*payload_idx as u32);
                let class_name = bytecode.resolve_symbol(payload.first);
                let member_name = bytecode.resolve_symbol(payload.second);
                let key = format!("{}::{}", class_name, member_name);
                if bytecode.functions.borrow().contains_key(&key) {
                    self.registers[255] = Value16::string(key);

                } else if let Some(val) = self.get_var_cloned(&key) {
                    self.registers[255] = val;

                } else {
                    return Err(compile_codes::runtime_error(format!(
                        "Static member not found: {}.{}",
                        class_name, member_name
                    )));
                }
            }

            Instruction::ClassStaticDecl(payload_idx) => {
                // CROSS-2a: payload lives in `bytecode.class_static_decl_payloads`.
                let payload = &bytecode.class_static_decl_payloads[*payload_idx as usize];
                let class_name = bytecode.resolve_symbol(payload.class_name.0);
                let static_methods: Vec<String> =
                    payload.static_methods.iter().map(|s| bytecode.resolve_symbol(s.0)).collect();
                let static_fields: Vec<String> =
                    payload.static_fields.iter().map(|s| bytecode.resolve_symbol(s.0)).collect();
                let class_val = self.get_var_cloned(&class_name);
                let mut class_obj = match class_val {
                    Some(v) => {
                        if let Some(obj) = v.as_object() {
                            obj.clone()
                        } else {
                            let mut obj = HashMap::new();
                            obj.insert("__class".to_string(), Value16::string(class_name.clone()));
                            obj
                        }
                    }
                    None => {
                        let mut obj = HashMap::new();
                        obj.insert("__class".to_string(), Value16::string(class_name.clone()));
                        obj
                    }
                };
                for method in &static_methods {
                    let chunk_name = format!("{}::{}", class_name, method);
                    class_obj.insert(method.clone(), Value16::string(chunk_name));
                }
                // OOP0003: static fields — move values from globals into class object
                for field in &static_fields {
                    let global_key = format!("{}::{}", class_name, field);
                    if let Some(val) = self.get_var_cloned(&global_key) {
                        class_obj.insert(field.clone(), val);
                    }
                }
                self.set_var(&class_name, Value16::object(class_obj))?;
            }

            // Issue #667: Yield — send value lazily or collect eagerly
            Instruction::Yield { .. } => {
                let val = self.registers[255];
                if let Some(ref sender) = self.yield_sender {
                    // Lazy path: send value through rendezvous channel.
                    // This blocks until the consumer calls next() / recv().
                    if sender.send(val).is_err() {
                        // Consumer dropped — stop execution (exits outer
                        // loop with `hit_return = false`).
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
            Instruction::MakeGenerator { payload_idx, first_arg, arg_count } => {
                // CROSS-2c: resolve the call payload from the side table.
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                let func_name_sym = payload.sym;
                let func_name = bytecode.resolve_symbol(func_name_sym.0);
                let n = *arg_count as usize;
                let first = *first_arg as usize;
                let args: Vec<Value16> = (0..n).map(|i| self.registers[first + i]).collect();

                if let Some(chunk) = bytecode.functions.borrow().get(&func_name).cloned() {
                    // Rendezvous channel: sender blocks until receiver calls recv()
                    let (yield_tx, yield_rx) = std::sync::mpsc::sync_channel::<Value16>(0);

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
                        // compat) so `LoadLocal(slot)` resolves correctly
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
                        // Pre-size slot table from chunk metadata.

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
                                if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
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
                                if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                                    let slot = local_syms[idx].1 as i32;
                                    if slot >= 0 {
                                        gen_vm.registers[slot as usize] = rest_val;
                                        stored = true;
                                    }
                                }
                            }
                            if !stored {
                                gen_vm.globals.insert(rest_name, rest_val);
                            }
                        }

                        // Execute the generator body; when it finishes (or
                        // errors) the sender is dropped, signalling "done".
                        let _ = gen_vm.execute_chunk(&chunk_arc, &bytecode_clone);
                        // gen_vm (and yield_tx) dropped here → recv returns Err → done
                    });

                    self.registers[255] = Value16::generator(Arc::new(parking_lot::Mutex::new(
                        GeneratorState16::new(yield_rx),
                    )));
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
