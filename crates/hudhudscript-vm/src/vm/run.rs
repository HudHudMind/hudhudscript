use crate::vm::machine::CallFrame;
use crate::vm::prepack::prepack_instructions;
use crate::vm::ExecutionState;
use crate::vm::VM;

use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::Bytecode;
use hudhudscript_bytecode::SymId;
use hudhudscript_bytecode::Value16;
impl VM {
    pub(crate) fn add_chunk_constants(&mut self, chunk: &hudhudscript_bytecode::FunctionChunk) {
        // G3-2: dedup — aynı chunk birden fazla eklenmez
        let ptr = chunk as *const hudhudscript_bytecode::FunctionChunk;
        if !self.constant_root_chunks.insert(ptr) {
            return; // already registered
        }
        self.gc_constant_roots.extend_from_slice(&chunk.constants);
    }

    pub fn execute(&mut self, bytecode: &Bytecode) -> CompileResult<()> {
        // GATE-2: reset telemetry counters for this execution
        #[cfg(feature = "telemetry")]
        {
            self.telemetry.reset();
            // FAZ0-A/E2: fusion "emitted" census — nihai bytecode kesiti
            // (tek envanter: telemetry::fused_name).
            self.telemetry.census_fusion_emitted(bytecode);
        }

        bytecode.validate()?;

        // P5.1: Sabit havuzlarını GC root olarak kaydet
        self.gc_constant_roots.clear();
        self.constant_root_chunks.clear();
        self.gc_constant_roots
            .extend_from_slice(&bytecode.constants);

        let has_slots = bytecode.main_local_count > 0;
        self.main_local_slots.clear();
        self.shared_globals_vec.clear();
        if has_slots {
            let shared_count = bytecode
                .main_local_names
                .iter()
                .enumerate()
                .filter(|(i, _)| bytecode.main_local_shared.get(*i).copied().unwrap_or(false))
                .count();
            self.shared_globals_vec
                .resize(shared_count, Value16::null());
            let mut shared_idx: u32 = 0;
            let mut sym_ids: Vec<u32> = Vec::with_capacity(bytecode.main_local_names.len());
            let mut max_sym = 0u32;
            for name in &bytecode.main_local_names {
                let sym_id = hudhudscript_bytecode::interner::intern(name).0;
                if sym_id > max_sym {
                    max_sym = sym_id;
                }
                sym_ids.push(sym_id);
            }
            self.main_local_slots
                .resize((max_sym + 1) as usize, u32::MAX);
            let mut built: Vec<(u32, usize, Option<usize>)> =
                Vec::with_capacity(bytecode.main_local_names.len());
            for (slot, &sym_id) in sym_ids.iter().enumerate() {
                let shared = bytecode
                    .main_local_shared
                    .get(slot)
                    .copied()
                    .unwrap_or(false);
                let si = if shared {
                    let cur = shared_idx;
                    shared_idx += 1;
                    cur
                } else {
                    0
                };
                let encoded = slot as u32 | ((shared as u32) << 8) | (si << 9);
                if self.main_local_slots[sym_id as usize] == u32::MAX {
                    self.main_local_slots[sym_id as usize] = encoded;
                }
                built.push((sym_id, slot, None));
            }
            built.sort_unstable_by_key(|(sym_id, _, _)| *sym_id);
            let built_ptr = Box::into_raw(Box::new(built));
            self.call_stack_local_syms.push(built_ptr);
            self.owned_local_sym_refs.push(built_ptr);
        }

        let packed = {
            let borrowed = bytecode.packed.borrow();
            if let Some(ref p) = *borrowed {
                p.clone()
            } else {
                drop(borrowed);
                let p = prepack_instructions(&bytecode.instructions);
                *bytecode.packed.borrow_mut() = Some(p.clone());
                p
            }
        };

        self.frame_stack.push(CallFrame {
            chunk_ptr: std::ptr::null(),
            owned_chunk: None,
            packed: &packed as *const Vec<u32>,
            func_sym: SymId(0),
            ip: 0,
            dst: 255,
            reg_base: 0,
            reg_size: 0,
            saved_finally: None,
            has_captures: false,
            debugger_pushed: false,
            call_depth: 0,
            owned_local_syms: has_slots,
            class_context: false,
            return_sink: crate::vm::call_state::ReturnSink::Register(255),
            receiver_context: None,
            swallow_error: false,
        });

        let stop_depth = 0;
        let result = self.run_frame_loop(bytecode, &packed, stop_depth);

        while let Some(frame) = self.frame_stack.pop() {
            self.teardown_frame(frame);
        }

        if let Err(e) = result {
            return Err(e);
        }

        if has_slots {
            for (slot, name) in bytecode.main_local_names.iter().enumerate() {
                let sym_id = hudhudscript_bytecode::interner::intern(name);
                let value = self.registers.get_absolute(slot);
                self.globals.entry(sym_id).or_insert(value);
            }
        }

        if self.iterators.is_empty() {
            self.iterators.shrink_to_fit();
        }
        if self.try_frames.is_empty() {
            self.try_frames.shrink_to_fit();
        }
        Ok(())
    }

    /// Execute bytecode for at most `max_duration`.
    pub fn execute_for(
        &mut self,
        bytecode: &Bytecode,
        max_duration: std::time::Duration,
    ) -> CompileResult<ExecutionState> {
        bytecode.validate()?;
        // G1.4.4: clear dedup state for fresh execution (mirrors execute())
        self.gc_constant_roots.clear();
        self.constant_root_chunks.clear();
        self.gc_constant_roots
            .extend_from_slice(&bytecode.constants);
        let deadline = std::time::Instant::now() + max_duration;
        self.execution_deadline = Some(deadline);
        self.suspended_ip = None;

        self.main_local_slots.clear();
        self.shared_globals_vec.clear();
        if bytecode.main_local_count > 0 {
            let shared_count = bytecode
                .main_local_names
                .iter()
                .enumerate()
                .filter(|(i, _)| bytecode.main_local_shared.get(*i).copied().unwrap_or(false))
                .count();
            self.shared_globals_vec
                .resize(shared_count, Value16::null());
            let mut shared_idx: u32 = 0;
            let mut max_sym = 0u32;
            for name in bytecode.main_local_names.iter() {
                let sym_id = hudhudscript_bytecode::interner::intern(name).0;
                if sym_id > max_sym {
                    max_sym = sym_id;
                }
            }
            self.main_local_slots
                .resize((max_sym + 1) as usize, u32::MAX);
            for (slot, name) in bytecode.main_local_names.iter().enumerate() {
                let sym_id = hudhudscript_bytecode::interner::intern(name).0;
                let shared = bytecode
                    .main_local_shared
                    .get(slot)
                    .copied()
                    .unwrap_or(false);
                let si = if shared {
                    let cur = shared_idx;
                    shared_idx += 1;
                    cur
                } else {
                    0
                };
                let encoded = slot as u32 | ((shared as u32) << 8) | (si << 9);
                if self.main_local_slots[sym_id as usize] == u32::MAX {
                    self.main_local_slots[sym_id as usize] = encoded;
                }
            }
        }

        let packed = {
            let borrowed = bytecode.packed.borrow();
            if let Some(ref p) = *borrowed {
                p.clone()
            } else {
                drop(borrowed);
                let p = prepack_instructions(&bytecode.instructions);
                *bytecode.packed.borrow_mut() = Some(p.clone());
                p
            }
        };

        self.frame_stack.push(CallFrame {
            chunk_ptr: std::ptr::null(),
            owned_chunk: None,
            packed: &packed as *const Vec<u32>,
            func_sym: SymId(0),
            ip: 0,
            dst: 255,
            reg_base: 0,
            reg_size: 0,
            saved_finally: None,
            has_captures: false,
            debugger_pushed: false,
            call_depth: 0,
            owned_local_syms: false,
            class_context: false,
            return_sink: crate::vm::call_state::ReturnSink::Register(255),
            receiver_context: None,
            swallow_error: false,
        });

        let stop_depth = 0;
        let result = self.run_frame_loop(bytecode, &packed, stop_depth);

        while let Some(frame) = self.frame_stack.pop() {
            self.teardown_frame(frame);
        }

        self.execution_deadline = None;

        match result {
            Ok(_) => {
                if self.iterators.is_empty() {
                    self.iterators.shrink_to_fit();
                }
                if self.try_frames.is_empty() {
                    self.try_frames.shrink_to_fit();
                }
                Ok(ExecutionState::Completed)
            }
            Err(e)
                if e.context_get("message")
                    .is_some_and(|m| m.contains("Execution time limit reached")) =>
            {
                Ok(ExecutionState::Suspended {
                    ip: self.suspended_ip.unwrap_or(0),
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Resume execution after a previous `Suspended` result.
    pub fn execute_resume(
        &mut self,
        bytecode: &Bytecode,
        max_duration: std::time::Duration,
    ) -> CompileResult<ExecutionState> {
        self.execute_for(bytecode, max_duration)
    }

    /// Clean up transient VM state after execution (#532).
    pub(crate) fn cleanup(&mut self) {
        // Register-based VM: stack field no longer used for cleanup.
        self.iterators.clear();
        self.iterator_generators.clear();
        self.iterators.shrink_to_fit();
        self.try_frames.clear();
        self.try_frames.shrink_to_fit();
        self.finally_frames.clear();
        self.finally_frames.shrink_to_fit();
        self.pending_flow = None;
        self.class_context_stack.clear();
        self.class_context_stack.shrink_to_fit();
    }
}
