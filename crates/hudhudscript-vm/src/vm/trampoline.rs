use crate::vm::machine::CallFrame;
use crate::vm::machine::ChunkCache;
use crate::vm::prepack::prepack_instructions;
use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, SymId, Value16};
use std::collections::HashMap;
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

        // Push a new scope_cells map for the function's upvalues.
        let has_captures =
            !chunk.captures.is_empty() || closure_captures.map_or(false, |c| !c.is_empty());
        if has_captures {
            self.push_scope_cells();
        }

        // PERF-1 (2026-04-17): params bind EXCLUSIVELY via slot-based
        // register arena + sym_to_slot (populated below).
        // if a param isn't covered by `chunk.local_names`, that's a compiler
        // bug caught by parity tests.
        //
        // Previously: HashMap<String, Value>::insert was unconditional for
        // every param on every call — perf hotspot for fib(30) (6.43% CPU on
        // hash+insert + triggered first-allocation of the scope HashMap, i.e.
        // ~4% of total mi_page_malloc). With this change, the scope HashMap
        // stays empty for function params; no insert, no alloc, no string
        // hashing. pop_scope drops an empty HashMap at zero cost.

        // Upvalue cell install (Issue #1078 — interpreter parity):
        // Each captured name carries an `Arc<RwLock<Value>>` cell.  We
        // install the cell into the new function's `scope_cells` sidecar
        // so that every `LoadVar` / `StoreVar` for that name routes
        // through the cell via `get_var_cloned` / `set_var`.  Globals
        // stay untouched (reads are cell-first).
        //
        // Because the cell is the canonical storage, both `c1()` and
        // `c2()` from independent `counter()` calls see THEIR OWN cell,
        // and mutations persist across calls — matching the interpreter's
        // `Arc<Environment>` capture exactly.
        for capture_name in chunk.captures.iter() {
            let cell = if let Some(cell) = closure_captures.and_then(|c| c.get(capture_name)) {
                Arc::clone(cell)
            } else if let Some(parent_cell) = self.find_cell_excluding_top(capture_name) {
                // Fallback: a cell already exists in an outer scope (rare:
                // typically means the closure was loaded via a path that
                // didn't populate `closure_captures`, e.g. stray LoadConst).
                parent_cell
            } else if let Some(v) = self.get_var_cloned(capture_name) {
                // Last-resort fallback: wrap the current live value.
                Arc::new(parking_lot::RwLock::new(v))
            } else {
                continue;
            };
            self.install_cell(capture_name.clone(), cell);
        }
        // Also install any captures that the caller passed in but which
        // aren't in `chunk.captures` (covers legacy/mixed bytecode paths).
        if let Some(captures) = closure_captures {
            for (cap_name, cap_cell) in captures {
                if !chunk.captures.contains(cap_name)
                    && !self
                        .scope_cells
                        .last()
                        .map(|c| c.contains_key(cap_name))
                        .unwrap_or(false)
                {
                    self.install_cell(cap_name.clone(), Arc::clone(cap_cell));
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
            Arc::clone(self.chunk_cache_last_val.as_ref().unwrap())
        } else if let Some(cc) = self.chunk_cache.get(&cache_key) {
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(cc));
            Arc::clone(cc)
        } else {
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
                                if !self
                                    .scope_cells
                                    .last()
                                    .map_or(false, |c| c.contains_key(cap_name))
                                {
                                    let cell = std::sync::Arc::new(parking_lot::RwLock::new(val));
                                    self.install_cell(cap_name.clone(), cell);
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

        self.frame_stack.push(CallFrame {
            chunk_ptr: chunk as *const FunctionChunk,
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
            class_context: false,
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

    /// Run the frame loop until `frame_stack.len() <= stop_depth`.
    /// `stop_depth` is the frame_stack length *before* the caller pushed its own frame(s).
    /// This ensures the loop stops when all frames pushed by/after this call are popped,
    /// without touching frames that belong to an outer caller (e.g., run.rs main frame).
    pub(crate) fn run_frame_loop(
        &mut self,
        bytecode: &Bytecode,
        main_packed: &[u32],
        stop_depth: usize,
    ) -> CompileResult<bool> {
        // ISSUE-3: Pre-allocate frame_stack capacity to avoid Vec realloc
        // memcpy during deep recursion (fib/hanoi/ack). Reduces __memcpy
        // from 2.69M per fib(30) run.
        self.frame_stack.reserve(64);
        let mut returned = false;
        let result = 'outer: loop {
            // Early exit: all frames we own have been popped.
            if self.frame_stack.len() <= stop_depth {
                break 'outer Ok(());
            }
            let frame_idx = self.frame_stack.len().saturating_sub(1);
            let (instructions, constants, packed_slice, chunk_sp, mut ip, dst) = {
                let frame = &self.frame_stack[frame_idx];
                if frame.chunk_ptr.is_null() {
                    (
                        &bytecode.instructions,
                        &bytecode.constants,
                        main_packed,
                        None,
                        frame.ip,
                        frame.dst,
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
                        frame.dst,
                    )
                }
            };
            // Set current chunk for IC fast-paths (property/call).
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
                    if let Some((func_sym, function_idx, arg_count, first_arg, dst, call_ip)) = self.pending_call.take() {
                        let frame_count_before = self.frame_stack.len();
                        let is_super = self.pending_super_call;
                        self.pending_super_call = false;
                        // PERF-B1: Inline cache-hit fast path into run_frame_loop.
                        let sym_id = func_sym.0 as usize;
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
                        self.exec_call(func_sym, function_idx, arg_count, first_arg, dst, bytecode, 0)?;
                        if is_super {
                            if let Some(frame) = self.frame_stack.last_mut() {
                                frame.class_context = true;
                            }
                        }
                        if self.frame_stack.len() == frame_count_before {
                            self.registers[dst as usize] = self.registers[255];
                        }
                    } else if hit_return {
                        returned = true;
                        let val = self.last_return;
                        let frame = self.frame_stack.pop().unwrap();
                        self.teardown_frame(frame);
                        if self.frame_stack.len() <= stop_depth {
                            self.registers[255] = val;
                            break 'outer Ok(());
                        }
                        self.registers[dst as usize] = val;
                    } else {
                        break 'outer Ok(());
                    }
                }
                Err(e) => break 'outer Err(e),
            }
        };
        result.map(|_| returned)
    }
}
