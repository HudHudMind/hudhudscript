use super::{ContinuationResume, ReturnSink, VmCallRequest};
use hudhudscript_bytecode::{gc, DynamicObject, FunctionChunk, SymId, Value16};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub(crate) struct PromiseCallbackState {
    pub(crate) dst: u8,
    pub(crate) callback: Value16,
    pub(crate) origin_ip: usize,
    pub(crate) argument: Value16,
    pub(crate) chunk: Arc<FunctionChunk>,
    pub(crate) func_sym: SymId,
    pub(crate) captures: FxHashMap<String, Arc<RwLock<Value16>>>,
}

impl PromiseCallbackState {
    pub(crate) fn request(&self) -> Box<VmCallRequest> {
        Box::new(VmCallRequest {
            chunk: Arc::clone(&self.chunk),
            func_sym: self.func_sym,
            args: vec![self.argument],
            captures: self
                .captures
                .iter()
                .map(|(name, cell)| (name.clone(), Arc::clone(cell)))
                .collect(),
            dst: self.dst,
            origin_ip: self.origin_ip,
            receiver_context: None,
            return_sink: ReturnSink::Discard,
            swallow_error: false,
        })
    }

    pub(super) fn finish(self, value: Value16) -> ContinuationResume {
        ContinuationResume::Complete {
            dst: self.dst,
            value: Value16::promise(hudhudscript_bytecode::PromiseState16::Resolved(Box::new(
                value,
            ))),
        }
    }

    pub(super) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        gc::trace_value(self.callback, gray);
        gc::trace_value(self.argument, gray);
        for cell in self.captures.values() {
            gc::trace_value(*cell.read(), gray);
        }
        for value in &self.chunk.constants {
            gc::trace_value(*value, gray);
        }
    }
}
