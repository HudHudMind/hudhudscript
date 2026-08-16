use crate::vm::call_state::ReturnSink;
use crate::vm::machine::CallFrame;
use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{FunctionChunk, SymId, Value16};
use std::sync::Arc;

impl VM {
    /// B2 FastCall: Minimal frame push for simple functions (no captures, no
    /// try/finally, non-async, non-generator). Skips debugger, scope_cells,
    /// capture loops, save_finally_state, and capture-param promotion.
    ///
    /// Called from run_frame_loop's inline cache-hit path (B1) when the
    /// chunk qualifies as simple. Avoids the full exec_call_push_frame call
    /// overhead and its dead branch checks.
    pub(crate) fn fast_call_push_frame(
        &mut self,
        chunk: &FunctionChunk,
        params: &[String],
        args: &[Value16],
        func_sym: SymId,
        first_arg: u8,
        dst: u8,
    ) -> CompileResult<()> {
        self.call_depth += 1;
        self.call_stack_names.push(func_sym);
        if self.call_depth > self.max_call_depth {
            self.call_depth -= 1;
            self.call_stack_names.pop();
            return Err(hudhudscript_bytecode::interner::resolve_with(
                hudhudscript_bytecode::interner::SymbolId(func_sym.0),
                |name| {
                    compile_codes::runtime_error(format!(
                        "Maximum call depth exceeded ({}) — possible infinite recursion in '{}'.",
                        self.max_call_depth, name
                    ))
                },
            ));
        }

        // FIX: allocate full 256-entry frames so callee's register window never
        // overlaps caller's live low registers (class receiver / recursion bug).
        let reg_size = 256usize;
        self.registers.advance(reg_size);

        // Param bind (same as exec_call_push_frame but without rest-fn check)
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

        // Chunk cache: single unified lookup (local_syms + packed).
        // P3-A1: cache-hit path avoids Arc::clone — takes pointers
        // directly from chunk_cache_last_val (still held alive).
        let cache_key = chunk.instructions.as_ptr() as usize;
        let (packed_ptr, local_syms_ptr) = if self.chunk_cache_last_key == cache_key {
            let cc = self
                .chunk_cache_last_val
                .as_ref()
                .expect("chunk cache last val missing");
            (Arc::as_ptr(&cc.packed), Arc::as_ptr(&cc.local_syms))
        } else if let Some(cc) = self.chunk_cache.get(&cache_key) {
            self.chunk_cache_last_key = cache_key;
            let cc = Arc::clone(cc);
            let pp = Arc::as_ptr(&cc.packed);
            let sp = Arc::as_ptr(&cc.local_syms);
            self.chunk_cache_last_val = Some(cc);
            (pp, sp)
        } else {
            // Build on miss (first call to this function).
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
            let cc = Arc::new(crate::vm::machine::ChunkCache {
                packed: Arc::new(crate::vm::prepack::prepack_instructions(
                    &chunk.instructions,
                )),
                local_syms: Arc::new(built),
                max_sym,
            });
            self.chunk_cache.insert(cache_key, Arc::clone(&cc));
            // P5.1: Sonradan yüklenen chunk sabitleri GC root.
            self.gc_constant_roots.extend_from_slice(&chunk.constants);
            self.chunk_cache_last_key = cache_key;
            let pp = Arc::as_ptr(&cc.packed);
            let sp = Arc::as_ptr(&cc.local_syms);
            self.chunk_cache_last_val = Some(cc);
            (pp, sp)
        };

        self.call_stack_local_syms.push(local_syms_ptr);

        // BUG1: plain functions also need to save the caller's try/finally
        // state so that cross-frame throw can restore it when unwinding.
        let saved_fin = self.save_finally_state();

        self.frame_stack.push(CallFrame {
            chunk_ptr: chunk as *const FunctionChunk,
            owned_chunk: None,
            packed: packed_ptr,
            func_sym,
            ip: 0,
            dst,
            reg_base: 0,
            reg_size,
            saved_finally: saved_fin,
            has_captures: false,
            debugger_pushed: false,
            call_depth: self.call_depth,
            owned_local_syms: false,
            class_context: false,
            return_sink: ReturnSink::Register(dst),
            receiver_context: None,
            swallow_error: false,
        });

        Ok(())
    }
}
