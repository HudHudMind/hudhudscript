use super::{
    ContinuationId, ContinuationResume, ReceiverContext, ReturnSink, VmCallRequest, VmContinuation,
};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{gc, DynamicObject, FunctionChunk, SymId, Value16};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub(crate) struct CustomIteratorSequence {
    pub(crate) receiver: Value16,
    pub(crate) elements: Vec<Value16>,
    pub(crate) variable_name: String,
    pub(crate) limit: usize,
    pub(crate) dst: u8,
    pub(crate) origin_ip: usize,
    pub(crate) chunk: Arc<FunctionChunk>,
    pub(crate) func_sym: SymId,
    pub(crate) captures: FxHashMap<String, Arc<RwLock<Value16>>>,
    pub(crate) class_sym: Option<SymId>,
    pub(crate) write_back: bool,
}

impl CustomIteratorSequence {
    fn request(&self, sink: ReturnSink) -> Box<VmCallRequest> {
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
            receiver_context: Some(ReceiverContext::new(
                self.receiver,
                self.class_sym,
                self.write_back,
            )),
            return_sink: sink,
            swallow_error: false,
        })
    }

    pub(super) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        gc::trace_value(self.receiver, gray);
        for value in &self.elements {
            gc::trace_value(*value, gray);
        }
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
fn iterator_error(message: String) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(message)
}

impl VM {
    pub(crate) fn start_custom_iterator_sequence(
        &mut self,
        receiver: Value16,
        variable_name: String,
        bytecode: &hudhudscript_bytecode::Bytecode,
        dst: u8,
        origin_ip: usize,
    ) -> CompileResult<()> {
        let next_sym = SymId(hudhudscript_bytecode::interner::intern("next").0);
        let outcome = self.call_method_on_value(
            &receiver,
            "next",
            next_sym,
            Vec::new(),
            bytecode,
            super::DeferredCallSite { dst, origin_ip },
        )?;

        if !matches!(outcome, super::MethodDispatchOutcome::Deferred) {
            return Err(iterator_error(
                "Custom iterator next() unexpectedly completed without a VM call".to_string(),
            ));
        }

        let request = self.pending_vm_call.take().ok_or_else(|| {
            iterator_error("Custom iterator next() produced no pending VmCallRequest".to_string())
        })?;
        let class_sym = request
            .receiver_context
            .as_ref()
            .and_then(|context| context.class_sym);
        let write_back = request
            .receiver_context
            .as_ref()
            .map(|context| context.write_back)
            .unwrap_or(false);
        let state = CustomIteratorSequence {
            receiver,
            elements: Vec::new(),
            variable_name,
            limit: self.max_builtin_iter,
            dst,
            origin_ip,
            chunk: Arc::clone(&request.chunk),
            func_sym: request.func_sym,
            captures: request
                .captures
                .iter()
                .map(|(name, cell)| (name.clone(), Arc::clone(cell)))
                .collect(),
            class_sym,
            write_back,
        };
        self.schedule_vm_call_with_continuation(
            VmContinuation::CustomIteratorSequence(state),
            request,
        )
    }

    pub(super) fn resume_custom_iterator(
        &mut self,
        id: ContinuationId,
        mut state: CustomIteratorSequence,
        value: Value16,
    ) -> CompileResult<ContinuationResume> {
        if value.is_null() {
            self.iterators
                .push((state.elements, state.variable_name, 0));
            self.iterator_generators.push(None);
            self.loop_headers.push((usize::MAX, usize::MAX));
            return Ok(ContinuationResume::Discard);
        }

        state.elements.push(value);
        if state.elements.len() >= state.limit {
            return Err(iterator_error(format!(
                "Custom iterator exceeded maximum iteration limit of {}",
                state.limit
            )));
        }

        let request = state.request(ReturnSink::Continuation(id));
        self.vm_continuations[id.0] = VmContinuation::CustomIteratorSequence(state);
        Ok(ContinuationResume::Schedule(request))
    }
}
