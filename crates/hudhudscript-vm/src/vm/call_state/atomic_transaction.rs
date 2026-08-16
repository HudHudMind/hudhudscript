use super::{ContinuationId, ContinuationResume, ReturnSink, VmCallRequest, VmContinuation};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{gc, DynamicObject, FunctionChunk, SymId, Value16};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub(crate) struct AtomicTransactionAttemptState {
    pub(crate) function: Value16,
    pub(crate) chunk: Arc<FunctionChunk>,
    pub(crate) func_sym: SymId,
    pub(crate) captures: FxHashMap<String, Arc<RwLock<Value16>>>,
    pub(crate) dst: u8,
    pub(crate) origin_ip: usize,
    pub(crate) attempt: usize,
    pub(crate) started_at: std::time::Instant,
    pub(crate) config: hudhudscript_stm::StmConfig,
    pub(crate) backoff_us: u64,
}

impl AtomicTransactionAttemptState {
    pub(crate) fn request(&self, sink: ReturnSink) -> Box<VmCallRequest> {
        Box::new(VmCallRequest {
            chunk: Arc::clone(&self.chunk),
            func_sym: self.func_sym,
            args: Vec::new(),
            captures: self
                .captures
                .iter()
                .map(|(name, cell)| (name.clone(), Arc::clone(cell)))
                .collect(),
            dst: self.dst,
            origin_ip: self.origin_ip,
            receiver_context: None,
            return_sink: sink,
            swallow_error: false,
        })
    }

    pub(super) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        gc::trace_value(self.function, gray);
        for cell in self.captures.values() {
            gc::trace_value(*cell.read(), gray);
        }
        for value in &self.chunk.constants {
            gc::trace_value(*value, gray);
        }
    }
}

#[cold]
#[inline(never)]
fn atomic_error(message: String) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(message)
}

impl VM {
    /// Install a fresh transaction and schedule the first attempt body on
    /// the trampoline. Commit/retry decisions happen in the continuation.
    pub(crate) fn start_atomic_transaction_attempt(
        &mut self,
        state: AtomicTransactionAttemptState,
    ) -> CompileResult<()> {
        self.current_tx = Some(Box::new(hudhudscript_stm::Transaction::new()));
        self.in_stm_context = true;
        let request = state.request(ReturnSink::Discard);
        self.schedule_vm_call_with_continuation(
            VmContinuation::AtomicTransactionAttempt(state),
            request,
        )
    }

    pub(super) fn resume_atomic_transaction_attempt(
        &mut self,
        id: ContinuationId,
        mut state: AtomicTransactionAttemptState,
        callback_result: Value16,
    ) -> CompileResult<ContinuationResume> {
        self.in_stm_context = false;
        let committed = self
            .current_tx
            .take()
            .map(|tx| tx.try_commit())
            .unwrap_or(true);
        if committed {
            return Ok(ContinuationResume::Complete {
                dst: state.dst,
                value: callback_result,
            });
        }

        state.attempt += 1;
        let elapsed_ms = state.started_at.elapsed().as_millis() as u64;
        if elapsed_ms > state.config.timeout_ms {
            return Err(atomic_error(
                hudhudscript_stm::err_timeout(state.config.timeout_ms, elapsed_ms).message,
            ));
        }
        if state.attempt >= state.config.max_retries {
            return Err(atomic_error(
                hudhudscript_stm::err_max_retries_exceeded(state.config.max_retries).message,
            ));
        }
        if state.backoff_us < 10 {
            std::thread::yield_now();
        } else {
            std::thread::sleep(std::time::Duration::from_micros(state.backoff_us));
        }
        state.backoff_us = (state.backoff_us * 2).min(state.config.max_backoff_us);
        self.current_tx = Some(Box::new(hudhudscript_stm::Transaction::new()));
        self.in_stm_context = true;
        let request = state.request(ReturnSink::Continuation(id));
        self.vm_continuations[id.0] = VmContinuation::AtomicTransactionAttempt(state);
        Ok(ContinuationResume::Schedule(request))
    }
}
