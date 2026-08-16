mod execute_chunk;
pub(crate) mod helpers;
mod super_call;

use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::FunctionData;
use hudhudscript_bytecode::{Bytecode, Value16};
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
            if !chunk.captures.is_empty() {
                // Captured functions need the runtime FunctionData value so the
                // closure's captured cell map is passed into the frame.
            } else {
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
                    self.fast_call_push_frame(
                        &chunk,
                        &chunk.params,
                        args,
                        name_sym,
                        first_arg,
                        dst,
                    )?;
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
        }

        // FAST PATH: call-cache hit — raw pointer, no atomic ops.
        let cached = self
            .call_cache
            .get(sym_id)
            .and_then(|slot| slot.as_ref().map(|(fd, cp, pp)| (*fd, *cp, *pp)));

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
        let result = 'cache: {
            // Try call-cache hit with validation guard.
            if let Some((cached_fd, chunk_ptr, params_ptr)) = cached {
                // P3: raw-pointer guard — skip guard if fd_ptr is null
                // (e.g. method-call cache entries); otherwise compare pointers.
                let guard_ok = if cached_fd.is_null() {
                    true
                } else {
                    let func_val = self.get_var_cloned_by_sym(sym_id as u32);
                    func_val
                        .and_then(|v| v.as_function_data_ptr())
                        .map(|fd_ptr| fd_ptr == cached_fd)
                        .unwrap_or(false)
                };
                if guard_ok {
                    #[cfg(feature = "telemetry")]
                    {
                        self.telemetry.call_cache_hit += 1;
                    }
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
                    break 'cache Ok(());
                } else {
                    // Guard failed — invalidate cache entry, fall through.
                    self.call_cache[sym_id] = None;
                }
            }
            // Slow path (fresh lookup or cache-invalidated)
            #[cfg(feature = "telemetry")]
            {
                self.telemetry.call_cache_miss += 1;
            }
            let name = bytecode.resolve_symbol(name_sym.0);
            if self.is_builtin(&name) {
                self.call_builtin(
                    &name,
                    arg_count,
                    first_arg,
                    bytecode,
                    crate::vm::call_state::DeferredCallSite { dst, origin_ip: ip },
                )?;
                Ok(())
            } else if let Some(func_val) = self.get_var_cloned(&name) {
                if let Some(func) = func_val.as_function_data() {
                    let FunctionData {
                        chunk_name,
                        params,
                        captures,
                        ..
                    } = func;
                    if let Some(chunk) = bytecode.get_function(chunk_name.as_str()) {
                        if captures.is_empty() {
                            if self.call_cache.len() <= sym_id {
                                self.call_cache.resize(sym_id + 1, None);
                            }
                            let params_box = Box::new(params.clone());
                            let fd_ptr = func_val.as_function_data_ptr();
                            self.call_cache[sym_id] = Some((
                                fd_ptr.unwrap_or(std::ptr::null()),
                                Arc::as_ptr(&chunk),
                                Box::into_raw(params_box) as *const Vec<String>,
                            ));
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
}
