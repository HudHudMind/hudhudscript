use crate::vm::call_state::ReturnSink;
use crate::vm::machine::{CallFrame, ChunkCache};
use crate::vm::prepack::prepack_instructions;
use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, FunctionChunk, SymId, Value16};
use std::sync::Arc;

impl VM {
    /// Execute a function chunk through the canonical frame driver.
    #[inline(never)]
    pub(crate) fn execute_chunk(
        &mut self,
        chunk: &FunctionChunk,
        bytecode: &Bytecode,
    ) -> CompileResult<()> {
        let cache_key = chunk.instructions.as_ptr() as usize;
        let mut cache = if self.chunk_cache_last_key == cache_key {
            Arc::clone(self.chunk_cache_last_val.as_ref().unwrap())
        } else if let Some(cache) = self.chunk_cache.get(&cache_key) {
            let cache = Arc::clone(cache);
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(&cache));
            cache
        } else {
            let cache = Arc::new(ChunkCache {
                packed: Arc::new(prepack_instructions(&chunk.instructions)),
                local_syms: Arc::new(Vec::new()),
                max_sym: 0,
            });
            self.chunk_cache.insert(cache_key, Arc::clone(&cache));
            self.gc_constant_roots.extend_from_slice(&chunk.constants);
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(&cache));
            cache
        };
        if cache.packed.is_empty() {
            let updated = Arc::new(ChunkCache {
                packed: Arc::new(prepack_instructions(&chunk.instructions)),
                local_syms: Arc::clone(&cache.local_syms),
                max_sym: cache.max_sym,
            });
            self.chunk_cache.insert(cache_key, Arc::clone(&updated));
            self.chunk_cache_last_key = cache_key;
            self.chunk_cache_last_val = Some(Arc::clone(&updated));
            cache = updated;
        }

        let packed = &cache.packed;
        let stop_depth = self.frame_stack.len();
        self.frame_stack.push(CallFrame {
            chunk_ptr: chunk as *const FunctionChunk,
            owned_chunk: None,
            packed: Arc::as_ptr(packed),
            func_sym: SymId(0),
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
            return_sink: ReturnSink::Register(255),
            receiver_context: None,
            swallow_error: false,
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
