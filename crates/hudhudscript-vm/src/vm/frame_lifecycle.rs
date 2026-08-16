use crate::vm::call_state::{ReceiverContext, ReturnSink, VmCallRequest};
use crate::vm::machine::CallFrame;
use crate::vm::machine::ChunkCache;
use crate::vm::prepack::prepack_instructions;
use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, SymId, Value16};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::Arc;

impl VM {
    pub(crate) fn exec_call_push_frame(
        &mut self,
        chunk: &FunctionChunk,
        params: &[String],
        args: &[Value16],
        _bytecode: &Bytecode,
        func_sym: SymId,
        closure_captures: Option<&HashMap<String, Arc<parking_lot::RwLock<Value16>>>>,
        first_arg: u8,
        dst: u8,
    ) -> CompileResult<()> {
        self.exec_call_push_frame_with_state(
            chunk,
            params,
            args,
            _bytecode,
            func_sym,
            closure_captures,
            first_arg,
            dst,
            ReturnSink::Register(dst),
            None,
            None,
        )
    }

    pub(crate) fn exec_vm_call_push_frame(
        &mut self,
        request: Box<VmCallRequest>,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let VmCallRequest {
            chunk,
            func_sym,
            args,
            captures,
            dst,
            origin_ip: _,
            receiver_context,
            return_sink,
            swallow_error,
        } = *request;
        let owned_chunk = Arc::clone(&chunk);
        self.exec_call_push_frame_with_state(
            &chunk,
            &chunk.params,
            &args,
            bytecode,
            func_sym,
            Some(&captures),
            0,
            dst,
            return_sink,
            receiver_context.map(Box::new),
            Some(owned_chunk),
        )?;
        if swallow_error {
            self.frame_stack
                .last_mut()
                .expect("frame must exist")
                .swallow_error = true;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn exec_call_push_frame_with_state<S: BuildHasher>(
        &mut self,
        chunk: &FunctionChunk,
        params: &[String],
        args: &[Value16],
        _bytecode: &Bytecode,
        func_sym: SymId,
        closure_captures: Option<&HashMap<String, Arc<parking_lot::RwLock<Value16>>, S>>,
        _first_arg: u8,
        dst: u8,
        return_sink: ReturnSink,
        mut receiver_context: Option<Box<ReceiverContext>>,
        owned_chunk: Option<Arc<FunctionChunk>>,
    ) -> CompileResult<()> {
        // Issue #965: Recursion depth guard (Phase 1 — heuristic ceiling).
        //
        // Fast path: check depth BEFORE push to avoid Vec realloc on deep recursion.
        if self.call_depth + 1 > self.max_call_depth {
            return Err(hudhudscript_bytecode::interner::resolve_with(
                hudhudscript_bytecode::interner::SymbolId(func_sym.0),
                |name| {
                    compile_codes::runtime_error(format!(
                        "Maximum call depth exceeded ({}) — possible infinite recursion in '{}'. \
                     Increase with vm.with_max_call_depth(n) (hard ceiling: 4000).",
                        self.max_call_depth, name
                    ))
                },
            ));
        }
        self.call_depth += 1;
        self.call_stack_names.push(func_sym);
        debug_assert!(self.call_depth <= self.max_call_depth);

        // Issue #661: Debugger push frame
        if let Some(ref mut dbg) = self.debugger {
            hudhudscript_bytecode::interner::resolve_with(
                hudhudscript_bytecode::interner::SymbolId(func_sym.0),
                |name| dbg.push_frame(name.to_string()),
            );
        }

        // G5-slotvec: push scope_cells with slot-vector (cells + sym_ids for legacy lookups).
        let has_captures =
            !chunk.captures.is_empty() || closure_captures.map_or(false, |c| !c.is_empty());
        if has_captures {
            let num = chunk.captures.len();
            // sym_ids: Arc<[u32]> shared from chunk — single ref-count bump, no clone
            let sym_ids: Arc<[u32]> = chunk.capture_sym_ids.clone().into();
            self.push_scope_cells(num, sym_ids);
        }

        // G5-slotvec: install capture cells by slot index (no hash, no sym lookup).
        let capture_slots = chunk.capture_slots.as_slice();
        for (i, capture_name) in chunk.captures.iter().enumerate() {
            let sym_id = *chunk.capture_sym_ids.get(i).unwrap_or_else(|| {
                panic!(
                    "Compiler invariant violation: capture_sym_ids missing for '{}' (index {} out of {})",
                    capture_name, i, chunk.capture_sym_ids.len()
                );
            });
            let slot = capture_slots.get(i).copied().unwrap_or(i as u8);
            let cell = if let Some(cell) = closure_captures.and_then(|c| c.get(capture_name)) {
                Arc::clone(cell)
            } else if let Some(parent_cell) = self.find_cell_excluding_top(capture_name) {
                parent_cell
            } else if sym_id != 0 {
                if let Some(v) = self.get_var_cloned_by_sym(sym_id) {
                    Arc::new(parking_lot::RwLock::new(v))
                } else {
                    continue;
                }
            } else if let Some(v) = self.get_var_cloned(capture_name) {
                Arc::new(parking_lot::RwLock::new(v))
            } else {
                continue;
            };
            self.install_cell_by_slot(slot, cell);
        }
        // G5-slotvec: legacy captures not in chunk.captures — install via linear sym_id scan.
        if let Some(captures) = closure_captures {
            for (cap_name, cap_cell) in captures {
                if !chunk.captures.contains(cap_name) {
                    let sym = hudhudscript_bytecode::interner::intern(cap_name).0;
                    let already_installed = self
                        .scope_cells
                        .last()
                        .map(|(_, sym_ids)| sym_ids.iter().any(|&s| s == sym))
                        .unwrap_or(false);
                    if !already_installed {
                        // Legacy: no slot available, skip — these captures are
                        // only present in old .hudb bytecode, not new compilations.
                    }
                }
            }
        }

        // PERF0004: Flat call-frame stack — zero mallocs after warmup.

        // K9: Arena-based register frame — no memcpy save/restore.
        // FIX: allocate full 256-entry frames so callee's register window never
        // overlaps caller's live low registers (class receiver / recursion bug).
        let reg_size = 256usize;
        self.registers.advance(reg_size);
        // PERF-T2-1: Compiler invariant — every register is written before it
        // is read (param_slots bind params, local registers are initialised
        // by the compiler). zero_frame was unconditional memset = 1.15% fib, 35% ack.  Removed.
        // self.registers.zero_frame(reg_size);

        // Compiler invariant: every param must appear in chunk.local_names.
        if chunk.local_count == 0 && !params.is_empty() {
            self.registers.retreat(reg_size);
            return Err(hudhudscript_bytecode::interner::resolve_with(
                hudhudscript_bytecode::interner::SymbolId(func_sym.0),
                |name| {
                    compile_codes::runtime_error(format!(
                        "Compiler invariant violation in '{}': function has {} params but \
                     chunk.local_count == 0. Params must be included in chunk.local_names.",
                        name,
                        params.len()
                    ))
                },
            ));
        }
        // PERF0011: Unified chunk cache — single lookup for sym + packed.
        let cache_key = chunk.instructions.as_ptr() as usize;
        let cc = if self.chunk_cache_last_key == cache_key {
            #[cfg(feature = "telemetry")]
            {
                self.telemetry.chunk_cache_hit += 1;
            }
            Arc::clone(self.chunk_cache_last_val.as_ref().unwrap())
        } else if let Some(cc) = self.chunk_cache.get(&cache_key) {
            #[cfg(feature = "telemetry")]
            {
                self.telemetry.chunk_cache_hit += 1;
            }
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(cc));
            Arc::clone(cc)
        } else {
            #[cfg(feature = "telemetry")]
            {
                self.telemetry.chunk_cache_miss += 1;
            }
            let mut built: Vec<(u32, usize, Option<usize>)> =
                Vec::with_capacity(chunk.local_names.len());
            let mut max_sym: u32 = 0;
            for (i, name) in chunk.local_names.iter().enumerate() {
                let sym_id = hudhudscript_bytecode::interner::intern(name).0;
                if sym_id > max_sym {
                    max_sym = sym_id;
                }
                let param_idx = params
                    .iter()
                    .position(|p| p == name || p.strip_prefix("...").map_or(false, |s| s == name));
                built.push((sym_id, i, param_idx));
            }
            built.sort_by_key(|(sym_id, _, _)| *sym_id);
            let cc = Arc::new(ChunkCache {
                packed: Arc::new(prepack_instructions(&chunk.instructions)),
                local_syms: Arc::new(built),
                max_sym,
            });
            self.chunk_cache.insert(cache_key, Arc::clone(&cc));
            // P5.1: Sonradan yüklenen chunk'ların sabit havuzları da GC root.
            self.gc_constant_roots.extend_from_slice(&chunk.constants);
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(&cc));
            cc
        };

        // K1-1: Bind arguments directly to registers (r0, r1, ...).
        let has_rest_fn = params.last().map(|p| p.starts_with("...")).unwrap_or(false);
        let rest_idx = if has_rest_fn {
            params.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        for (pi, _) in chunk.param_slots.iter().enumerate() {
            if pi == rest_idx && has_rest_fn {
                let rest_values: Vec<Value16> = if args.len() > pi {
                    args[pi..].to_vec()
                } else {
                    Vec::new()
                };
                self.registers[pi] = Value16::array(rest_values);
            } else if let Some(val) = args.get(pi) {
                self.registers[pi] = *val;
            }
        }

        self.call_stack_local_syms.push(Arc::as_ptr(&cc.local_syms));

        // Bug F Part 2: Promote captured PARAMETERs to upvalue cells.
        // Only params are already initialized at this point; plain locals are
        // written by the callee's register initialisation during execute_chunk. Creating a cell here for
        // a non-param local reads an uninitialized slot (set_len without fill)
        // and installs a stale null cell that shadows the live slot value when
        // upvalue_cell_for runs later at DefineFunction time.
        for cap_name in chunk.captures.iter() {
            // Only promote if this capture is a function parameter
            let is_param = params
                .iter()
                .any(|p| p == cap_name || p.strip_prefix("...").map_or(false, |s| s == cap_name));
            if !is_param {
                continue;
            }
            if let Some(sym_id) = hudhudscript_bytecode::interner::try_resolve_id(cap_name) {
                if let Some(local_syms_ptr) = self.call_stack_local_syms.last() {
                    let local_syms = unsafe { &**local_syms_ptr };
                    if let Ok(idx) = local_syms.binary_search_by_key(&sym_id, |(s, _, _)| *s) {
                        let slot = local_syms[idx].1 as i32;
                        if slot >= 0 {
                            let local_idx = slot as usize;
                            if local_idx < self.registers.len() {
                                let val = self.registers[local_idx];
                                let sym_in_scope =
                                    self.scope_cells.last().map_or(false, |(_, sym_ids)| {
                                        sym_ids.iter().any(|&s| s == sym_id)
                                    });
                                if !sym_in_scope {
                                    let cell = std::sync::Arc::new(parking_lot::RwLock::new(val));
                                    if let Some((_, sym_ids)) = self.scope_cells.last() {
                                        if let Some(slot_idx) =
                                            sym_ids.iter().position(|&s| s == sym_id)
                                        {
                                            self.scope_cells.last_mut().unwrap().0[slot_idx] =
                                                Some(cell);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Register-based VM: no operand stack frame base needed.
        let saved_stack_frame_base = 0;
        let _new_stack_base = 0;

        // Gap 2 — snapshot caller's try/finally state.
        let saved_fin = self.save_finally_state();

        let debugger_pushed = self.debugger.is_some();
        let owned_local_syms = false;

        let class_context = receiver_context
            .as_ref()
            .and_then(|context| context.class_sym)
            .is_some();
        if let Some(context) = receiver_context.as_mut() {
            self.activate_receiver_context(context);
        }

        self.frame_stack.push(CallFrame {
            chunk_ptr: chunk as *const FunctionChunk,
            owned_chunk,
            packed: Arc::as_ptr(&cc.packed),
            func_sym,
            ip: 0,
            dst,
            reg_base: saved_stack_frame_base,
            reg_size,
            saved_finally: saved_fin,
            has_captures,
            debugger_pushed,
            call_depth: self.call_depth,
            owned_local_syms,
            class_context,
            return_sink,
            receiver_context,
            swallow_error: false,
        });

        Ok(())
    }

    pub(crate) fn teardown_frame(&mut self, frame: CallFrame) {
        // P3-A3: fast path for simple frames (no debugger, class, capture, finally, owned syms)
        if !frame.debugger_pushed
            && !frame.class_context
            && !frame.has_captures
            && !frame.owned_local_syms
            && frame.saved_finally.is_none()
            && frame.receiver_context.is_none()
        {
            self.call_depth = frame.call_depth.saturating_sub(1);
            self.call_stack_names.pop();
            self.stack_frame_base = frame.reg_base;
            self.registers.retreat(frame.reg_size);
            self.call_stack_local_syms.pop();
            return;
        }
        if frame.debugger_pushed {
            if let Some(ref mut dbg) = self.debugger {
                dbg.pop_frame();
            }
        }
        if frame.class_context {
            self.class_context_stack.pop();
        }
        self.call_depth = frame.call_depth.saturating_sub(1);
        self.call_stack_names.pop();
        self.stack_frame_base = frame.reg_base;
        self.restore_finally_state(frame.saved_finally);
        if let Some(context) = frame.receiver_context {
            self.close_receiver_context(*context);
        }
        self.registers.retreat(frame.reg_size);
        if frame.has_captures {
            self.pop_scope_cells();
        }
        self.call_stack_local_syms.pop();
        if frame.owned_local_syms {
            if let Some(ptr) = self.owned_local_sym_refs.pop() {
                if !ptr.is_null() {
                    unsafe { drop(Box::from_raw(ptr as *mut Vec<(u32, usize, Option<usize>)>)) };
                }
            }
        }
    }
}
