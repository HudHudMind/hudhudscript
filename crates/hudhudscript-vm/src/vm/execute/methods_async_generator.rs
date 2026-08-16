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
            Instruction::MethodCall {
                dst,
                obj,
                payload_idx,
                first_arg,
                arg_count,
            } => {
                // CROSS-2c: resolve the call payload from the side table.
                let payload = bytecode.get_call_payload(*payload_idx as u32);
                let method_sym = payload.sym;
                let payload_arg_count = payload.arg_count;
                let builtin = payload.builtin_method_idx;
                // P6: skip symbol resolution when builtin ID is known.
                // Fall back to string path for non-builtin methods.
                let method: String = if builtin != u32::MAX {
                    // Known builtin: resolve method name without string table lookup.
                    match builtin {
                        0 => "floor".to_string(),
                        1 => "sqrt".to_string(),
                        2 => "abs".to_string(),
                        3 => "ceil".to_string(),
                        4 => "round".to_string(),
                        _ => bytecode.resolve_symbol(method_sym.0),
                    }
                } else {
                    bytecode.resolve_symbol(method_sym.0)
                };
                let receiver = self.registers[*obj as usize];
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
                        let next_val = crate::vm::exec::helpers::generator_advance(self, state);
                        self.registers[*dst as usize] = next_val.unwrap_or(Value16::null());

                        *ip_ref = ip + 1;
                        return Ok(StepAction::Jumped);
                    }
                }

                // Clear the mutation scratch slot so a subsequent
                // WriteBackReceiver only fires for THIS method call
                // (not a leftover from a prior instance method call).
                self.last_instance_mutation = None;
                let call_site = crate::vm::call_state::DeferredCallSite {
                    dst: *dst,
                    origin_ip: ip,
                };
                match self.call_method_on_value(
                    &receiver, &method, method_sym, args, bytecode, call_site,
                )? {
                    crate::vm::call_state::MethodDispatchOutcome::Immediate(result) => {
                        self.registers[*dst as usize] = result;
                    }
                    crate::vm::call_state::MethodDispatchOutcome::Deferred => {
                        return Ok(StepAction::DeferredCall);
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
                            // V2-E: DetachedGraph promises are transported on a dedicated
                            // channel; resolve those first, then fall back to the shared
                            // registry for externally registered promises.
                            if let Some(rx) = self.detached_promises.remove(id) {
                                match rx.recv() {
                                    Ok(Ok(tree)) => {
                                        self.registers[255] =
                                            hudhudscript_bytecode::gc_detach::attach(&tree);
                                    }
                                    Ok(Err(msg)) => {
                                        return Err(compile_codes::runtime_error(format!(
                                            "Promise rejected: {}",
                                            msg
                                        )));
                                    }
                                    Err(_) => {
                                        return Err(compile_codes::runtime_error(
                                            "blocking task sender dropped".to_string(),
                                        ));
                                    }
                                }
                            } else if self.promise_registry.has_entry(id) {
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
                                return Err(compile_codes::runtime_error(format!(
                                    "await on unknown promise id {}",
                                    id
                                )));
                            }
                        }
                    }
                } else {
                    self.registers[255] = value;
                }
            }

            // ── Class extensionsClass extensions (Issue #345) ────────────────────────
            Instruction::SuperCall {
                dst: 255,
                payload_idx,
                first_arg,
                arg_count,
            } => {
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
                    .map(|s| {
                        hudhudscript_bytecode::interner::resolve(
                            hudhudscript_bytecode::interner::SymbolId(s.0),
                        )
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
                let chunk = bytecode.get_function(chunk_name.as_str()).ok_or_else(|| {
                    compile_codes::runtime_error(format!("Parent method not found: {}", chunk_name))
                })?;
                self.class_context_stack.push(hudhudscript_bytecode::SymId(
                    hudhudscript_bytecode::interner::intern(&parent_name).0,
                ));
                let func_sym = hudhudscript_bytecode::SymId(
                    hudhudscript_bytecode::interner::intern(&chunk_name).0,
                );
                // Put chunk in call_cache so exec_call can find it on the fast path.
                let sym_id = func_sym.0 as usize;
                if self.call_cache.len() <= sym_id {
                    self.call_cache.resize(sym_id + 1, None);
                }
                let params_box = Box::new(chunk.params.clone());
                self.call_cache[sym_id] = Some((
                    std::ptr::null(),
                    Arc::as_ptr(&chunk),
                    Box::into_raw(params_box) as *const Vec<String>,
                ));
                // P5.1: call_cache'ye yeni eklenen chunk sabitleri GC root.
                self.add_chunk_constants(&chunk);
                self.pending_super_call = true;
                return Ok(StepAction::Call {
                    func_sym,
                    function_idx: payload.function_idx,
                    arg_count: payload.arg_count,
                    first_arg: *first_arg,
                    dst: 255,
                    ip,
                });
            }

            Instruction::GetStatic(payload_idx) => {
                // CROSS-2d: two-sym payload — first = class symbol,
                // second = member symbol.
                let payload = bytecode.get_two_sym_payload(*payload_idx as u32);
                let class_name = bytecode.resolve_symbol(payload.first);
                let member_name = bytecode.resolve_symbol(payload.second);
                let key = format!("{}::{}", class_name, member_name);
                if bytecode.has_function(&key) {
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
                let static_methods: Vec<String> = payload
                    .static_methods
                    .iter()
                    .map(|s| bytecode.resolve_symbol(s.0))
                    .collect();
                let static_fields: Vec<String> = payload
                    .static_fields
                    .iter()
                    .map(|s| bytecode.resolve_symbol(s.0))
                    .collect();
                let class_val = self.get_var_cloned(&class_name);
                let mut class_obj = match class_val {
                    Some(v) => {
                        if let Some(obj) = v.as_object() {
                            obj.clone()
                        } else {
                            let mut obj = hudhudscript_bytecode::ObjMap::default();
                            obj.insert("__class".to_string(), Value16::string(class_name.clone()));
                            obj
                        }
                    }
                    None => {
                        let mut obj = hudhudscript_bytecode::ObjMap::default();
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

            _ => unreachable!("instruction routed to wrong execute helper"),
        }

        Ok(StepAction::Advance)
    }
}
