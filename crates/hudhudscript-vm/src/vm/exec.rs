pub(crate) mod helpers;
use crate::vm::machine::ChunkCache;
use crate::vm::prepack::prepack_instructions;
use crate::vm::VM;
use crate::vm::{numeric_slot, GenStep, NumericSlot};
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::{CompileError, CompileResult};
use hudhudscript_bytecode::FunctionData;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, SymId, Value16};
use std::mem::ManuallyDrop;
use std::sync::Arc;

impl VM {
    #[inline]
    pub(crate) fn exec_call(
        &mut self,
        name_sym: hudhudscript_bytecode::SymId,
        function_idx: u32,
        arg_count: u8,
        first_arg: u8,
        dst: u8,
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<()> {
        let sym_id = name_sym.0 as usize;

        // DIRECT INDEX FAST PATH: payload already resolved the function index.
        // Skip symbol resolution, variable lookup, and call-cache setup.
        if function_idx != u32::MAX {
            let chunk = {
                let funcs = bytecode.functions.borrow();
                if let Some(chunk) = funcs.get(function_idx as usize).cloned() {
                    chunk
                } else {
                    return Err(compile_codes::runtime_error(format!(
                        "Call direct index {} out of bounds",
                        function_idx
                    )));
                }
            };
            let n = arg_count as usize;
            let first = first_arg as usize;
            let mut arr = [Value16::null(); 8];
            let mut saved_scratch = Vec::new();
            let args: &[Value16] = if n > 8 {
                self.args_scratch.clear();
                self.args_scratch
                    .extend((0..n).map(|i| self.registers[first + i]));
                saved_scratch = std::mem::take(&mut self.args_scratch);
                &saved_scratch
            } else {
                for i in 0..n {
                    arr[i] = self.registers[first + i];
                }
                &arr[..n]
            };
            if chunk.is_async {
                let name = bytecode.resolve_symbol(name_sym.0);
                let promise = self.spawn_async_chunk(
                    Arc::clone(&chunk),
                    &chunk.params,
                    args,
                    bytecode,
                    &name,
                    None,
                );
                self.registers[255] = promise;
            } else if chunk.is_plain_function {
                // P1: fast path for plain recursive functions
                self.fast_call_push_frame(&chunk, &chunk.params, args, name_sym, first_arg, dst)?;
            } else {
                self.exec_call_push_frame(
                    &chunk,
                    &chunk.params,
                    args,
                    bytecode,
                    name_sym,
                    None,
                    first_arg,
                    dst,
                )?;
            }
            return Ok(());
        }

        // FAST PATH: call-cache hit — raw pointer, no atomic ops.
        let cached = self
            .call_cache
            .get(sym_id)
            .and_then(|slot| slot.as_ref().map(|(ptr, params_ptr)| (*ptr, *params_ptr)));

        let n = arg_count as usize;
        let first = first_arg as usize;

        // PERF-T1-1: Eliminate Vec::from heap alloc for n≤8.
        let mut arr = [Value16::null(); 8];
        let mut saved_scratch = Vec::new();
        let args: &[Value16] = if n > 8 {
            self.args_scratch.clear();
            self.args_scratch
                .extend((0..n).map(|i| self.registers[first + i]));
            saved_scratch = std::mem::take(&mut self.args_scratch);
            &saved_scratch
        } else {
            for i in 0..n {
                arr[i] = self.registers[first + i];
            }
            &arr[..n]
        };

        // T2-7: Inline exec_call_inner logic — eliminates one Rust stack frame per call.
        let result = if let Some((chunk_ptr, params_ptr)) = cached {
            let chunk = unsafe { &*chunk_ptr };
            let params = unsafe { &*params_ptr };
            if chunk.is_async {
                let arc_chunk = ManuallyDrop::new(unsafe { Arc::from_raw(chunk_ptr) });
                let name = bytecode.resolve_symbol(name_sym.0);
                let promise = self.spawn_async_chunk(
                    Arc::clone(&*arc_chunk),
                    params,
                    args,
                    bytecode,
                    &name,
                    None,
                );
                self.registers[255] = promise;
            } else {
                self.exec_call_push_frame(
                    chunk, params, args, bytecode, name_sym, None, first_arg, dst,
                )?;
            }
            Ok(())
        } else {
            let name = bytecode.resolve_symbol(name_sym.0);
            if self.is_builtin(&name) {
                self.call_builtin(&name, arg_count, first_arg, bytecode)?;
                Ok(())
            } else if let Some(func_val) = self.get_var_cloned(&name) {
                if let Some(func) = func_val.as_function_data() {
                    let FunctionData {
                        chunk_name,
                        params,
                        captures,
                        ..
                    } = func;
                    if let Some(chunk) = bytecode
                        .get_function(chunk_name.as_str())
                    {
                        if captures.is_empty() {
                            if self.call_cache.len() <= sym_id {
                                self.call_cache.resize(sym_id + 1, None);
                            }
                            let params_box = Box::new(params.clone());
                            self.call_cache[sym_id] =
                                Some((Arc::as_ptr(&chunk), Box::into_raw(params_box)));
                        }
                        if chunk.is_async {
                            let promise = self.spawn_async_chunk(
                                Arc::clone(&chunk),
                                &params,
                                args,
                                bytecode,
                                &name,
                                Some(&captures),
                            );
                            self.registers[255] = promise;
                        } else {
                            self.exec_call_push_frame(
                                &chunk,
                                &params,
                                args,
                                bytecode,
                                name_sym,
                                Some(&captures),
                                first_arg,
                                dst,
                            )?;
                        }
                        Ok(())
                    } else {
                        Err(Self::runtime_error_with_pos(
                            format!("Unknown function in call: {}", name),
                            bytecode,
                            ip,
                        ))
                    }
                } else {
                    // Try action_registry for perform-style calls (AgentName.actionName)
                    if let Some(action_chunk) = bytecode
                        .action_registry
                        .borrow()
                        .get(name.as_str())
                        .cloned()
                    {
                        if action_chunk.params.len() != n {
                            return Err(Self::runtime_error_with_pos(
                                format!(
                                    "Action {} expects {} arguments, got {}",
                                    name,
                                    action_chunk.params.len(),
                                    n
                                ),
                                bytecode,
                                ip,
                            ));
                        }
                        if action_chunk.is_async {
                            let promise = self.spawn_async_chunk(
                                Arc::clone(&action_chunk),
                                &action_chunk.params,
                                args,
                                bytecode,
                                &name,
                                None,
                            );
                            self.registers[255] = promise;
                        } else {
                            self.exec_call_push_frame(
                                &action_chunk,
                                &action_chunk.params,
                                args,
                                bytecode,
                                name_sym,
                                None,
                                first_arg,
                                dst,
                            )?;
                        }
                        Ok(())
                    } else {
                        Err(Self::runtime_error_with_pos(
                            format!("Unknown function in call: {}", name),
                            bytecode,
                            ip,
                        ))
                    }
                }
            } else {
                // Direct functions lookup (effects registered as functions)
                if let Some(chunk) = bytecode.get_function(name.as_str()) {
                    if chunk.is_async {
                        let promise = self.spawn_async_chunk(
                            Arc::clone(&chunk),
                            &chunk.params,
                            args,
                            bytecode,
                            &name,
                            None,
                        );
                        self.registers[255] = promise;
                    } else {
                        self.exec_call_push_frame(
                            &chunk,
                            &chunk.params,
                            args,
                            bytecode,
                            name_sym,
                            None,
                            first_arg,
                            dst,
                        )?;
                    }
                    Ok(())
                } else if let Some(action_chunk) = bytecode
                    .action_registry
                    .borrow()
                    .get(name.as_str())
                    .cloned()
                {
                    if action_chunk.params.len() != n {
                        return Err(Self::runtime_error_with_pos(
                            format!(
                                "Action {} expects {} arguments, got {}",
                                name,
                                action_chunk.params.len(),
                                n
                            ),
                            bytecode,
                            ip,
                        ));
                    }
                    if action_chunk.is_async {
                        let promise = self.spawn_async_chunk(
                            Arc::clone(&action_chunk),
                            &action_chunk.params,
                            args,
                            bytecode,
                            &name,
                            None,
                        );
                        self.registers[255] = promise;
                    } else {
                        self.exec_call_push_frame(
                            &action_chunk,
                            &action_chunk.params,
                            args,
                            bytecode,
                            name_sym,
                            None,
                            first_arg,
                            dst,
                        )?;
                    }
                    Ok(())
                } else {
                    Err(Self::runtime_error_with_pos(
                        format!("Unknown action or function: {}", name),
                        bytecode,
                        ip,
                    ))
                }
            }
        };

        self.args_scratch = saved_scratch;
        result
    }

    /// Handle `Instruction::SuperCall` — extracted from `execute_instructions`
    /// to reduce the recursive frame size.
    #[inline(never)]
    pub(crate) fn exec_super_call(
        &mut self,
        method_name_sym: hudhudscript_bytecode::SymId,
        arg_count: u8,
        first_arg: u8,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let n = arg_count as usize;
        let first = first_arg as usize;

        // PERF-T1-1: Same zero-alloc fast path as exec_call.
        if n > 8 {
            self.args_scratch.clear();
            self.args_scratch
                .extend((0..n).map(|i| self.registers[first + i]));
            let args = std::mem::take(&mut self.args_scratch);
            let result = self.exec_super_call_inner(&args, method_name_sym, bytecode);
            self.args_scratch = args;
            result
        } else {
            let mut arr = [Value16::null(); 8];
            for i in 0..n {
                arr[i] = self.registers[first + i];
            }
            self.exec_super_call_inner(&arr[..n], method_name_sym, bytecode)
        }
    }

    fn exec_super_call_inner(
        &mut self,
        args: &[Value16],
        method_name_sym: hudhudscript_bytecode::SymId,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let method_name = bytecode.resolve_symbol(method_name_sym.0);
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
        if let Some(chunk) = bytecode
            .get_function(chunk_name.as_str())
        {
            self.class_context_stack.push(parent_name.clone());
            let func_sym = hudhudscript_bytecode::interner::intern(&chunk_name);
            self.exec_call_push_frame(
                &chunk,
                &chunk.params,
                args,
                bytecode,
                hudhudscript_bytecode::SymId(func_sym.0),
                Some(&std::collections::HashMap::new()),
                0,
                255,
            )?;
            if let Some(frame) = self.frame_stack.last_mut() {
                frame.class_context = true;
            }
            Ok(())
        } else {
            Err(compile_codes::runtime_error(format!(
                "Parent method not found: {}",
                chunk_name
            )))
        }
    }

    /// Execute a function chunk's instruction list.
    /// Delegates to the unified `execute_instructions` and pushes a default
    /// Null return value so `call_chunk` always has something to pop.
    #[inline(never)]
    pub(crate) fn execute_chunk(
        &mut self,
        chunk: &FunctionChunk,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let cache_key = chunk.instructions.as_ptr() as usize;
        let mut cc = if self.chunk_cache_last_key == cache_key {
            Arc::clone(self.chunk_cache_last_val.as_ref().unwrap())
        } else if let Some(c) = self.chunk_cache.get(&cache_key) {
            let c = Arc::clone(c);
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(&c));
            c
        } else {
            let c = Arc::new(ChunkCache {
                packed: Arc::new(prepack_instructions(&chunk.instructions)),
                local_syms: Arc::new(Vec::new()),
                max_sym: 0,
            });
            self.chunk_cache.insert(cache_key, Arc::clone(&c));
            // P5.1: Sonradan yüklenen chunk sabitleri GC root.
            self.gc_constant_roots.extend_from_slice(&chunk.constants);
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(&c));
            c
        };
        if cc.packed.is_empty() {
            let p = Arc::new(prepack_instructions(&chunk.instructions));
            let updated = ChunkCache {
                packed: p,
                local_syms: Arc::clone(&cc.local_syms),
                max_sym: cc.max_sym,
            };
            let new_cc = Arc::new(updated);
            self.chunk_cache.insert(cache_key, Arc::clone(&new_cc));
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(&new_cc));
            cc = new_cc;
        }

        let packed = &cc.packed;
        let stop_depth = self.frame_stack.len();
        self.frame_stack.push(crate::vm::machine::CallFrame {
            chunk_ptr: chunk as *const FunctionChunk,
            packed: Arc::as_ptr(packed),
            func_sym: hudhudscript_bytecode::SymId(0),
            ip: 0,
            dst: 255,
            reg_base: 0,
            reg_size: 0,
            saved_finally: None,
            has_captures: false,
            debugger_pushed: false,
            call_depth: self.call_depth,
            owned_local_syms: false,
            class_context: false,
        });

        let returned = self.run_frame_loop(bytecode, &*packed, stop_depth)?;

        while let Some(frame) = self.frame_stack.pop() {
            self.teardown_frame(frame);
        }

        if !returned {
            self.registers[255] = Value16::null();
        }
        Ok(())
    }
}
