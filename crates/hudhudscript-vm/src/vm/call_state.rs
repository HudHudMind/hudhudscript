//! Canonical VM-to-VM call and continuation state.
//!
//! Large call payloads live on `VM`, never in `StepAction`. This keeps the
//! instruction dispatch result compact while the outer frame loop owns every
//! user-chunk control transfer.

use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{gc, DynamicObject, FunctionChunk, SymId, Value16};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub(crate) use array_callback::{ArrayCallbackOperation, FunctionCallbackSequence};
pub(crate) use atomic_transaction::AtomicTransactionAttemptState;
pub(crate) use custom_iterator::CustomIteratorSequence;
pub(crate) use governance_dispatch::GovernanceDispatchState;
pub(crate) use promise_callback::PromiseCallbackState;
pub(crate) use sop_ability::{SopAbilitySequence, SopCallStep, SopResultPolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContinuationId(pub(crate) usize);

/// G06A: instruction-side call-site identity for deferred method calls.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DeferredCallSite {
    pub(crate) dst: u8,
    pub(crate) origin_ip: usize,
}

/// G06A: constructor completion state. The constructor's own return value
/// is discarded by contract; the instance is rebuilt from the mutated
/// receiver (or its write-back) when the deferred frame returns.
pub(crate) struct ConstructorContinuation {
    pub(crate) dst: u8,
    pub(crate) class_name: String,
    pub(crate) class_value: Value16,
    pub(crate) receiver: Value16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReturnSink {
    Register(u8),
    Continuation(ContinuationId),
    Discard,
}

pub(crate) struct ReceiverContext {
    pub(crate) receiver: Value16,
    pub(crate) previous_this: Option<Value16>,
    pub(crate) class_sym: Option<SymId>,
    pub(crate) write_back: bool,
}

impl ReceiverContext {
    pub(crate) fn new(receiver: Value16, class_sym: Option<SymId>, write_back: bool) -> Self {
        Self {
            receiver,
            previous_this: None,
            class_sym,
            write_back,
        }
    }

    pub(crate) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        gc::trace_value(self.receiver, gray);
        if let Some(previous_this) = self.previous_this {
            gc::trace_value(previous_this, gray);
        }
    }
}

pub(crate) struct VmCallRequest {
    pub(crate) chunk: Arc<FunctionChunk>,
    pub(crate) func_sym: SymId,
    pub(crate) args: Vec<Value16>,
    pub(crate) captures: FxHashMap<String, Arc<RwLock<Value16>>>,
    pub(crate) dst: u8,
    pub(crate) origin_ip: usize,
    pub(crate) receiver_context: Option<ReceiverContext>,
    pub(crate) return_sink: ReturnSink,
    /// SOP effect steps discard body errors instead of unwinding them.
    pub(crate) swallow_error: bool,
}

impl VmCallRequest {
    pub(crate) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        for value in &self.args {
            gc::trace_value(*value, gray);
        }
        for cell in self.captures.values() {
            gc::trace_value(*cell.read(), gray);
        }
        if let Some(context) = &self.receiver_context {
            context.trace_roots(gray);
        }
        for value in &self.chunk.constants {
            gc::trace_value(*value, gray);
        }
    }
}

/// G05 continuation shell. G06 replaces each operation-specific shell with
/// its full state machine while preserving this single VM-owned storage lane.
pub(crate) enum VmContinuation {
    SopAbilitySequence(SopAbilitySequence),
    FunctionCallbackSequence(FunctionCallbackSequence),
    GovernanceDispatch(GovernanceDispatchState),
    AtomicTransactionAttempt(AtomicTransactionAttemptState),
    CustomIteratorSequence(CustomIteratorSequence),
    PromiseCallback(PromiseCallbackState),
    ConstructorCall(ConstructorContinuation),
    Completed,
}

impl VmContinuation {
    pub(crate) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        match self {
            Self::SopAbilitySequence(state) => state.trace_roots(gray),
            Self::GovernanceDispatch(state) => state.trace_roots(gray),
            Self::AtomicTransactionAttempt(state) => state.trace_roots(gray),
            Self::FunctionCallbackSequence(state) => state.trace_roots(gray),
            Self::CustomIteratorSequence(state) => state.trace_roots(gray),
            Self::PromiseCallback(state) => state.trace_roots(gray),
            Self::ConstructorCall(state) => {
                gc::trace_value(state.class_value, gray);
                gc::trace_value(state.receiver, gray);
            }
            Self::Completed => {}
        }
    }
}

pub(crate) enum ContinuationResume {
    Schedule(Box<VmCallRequest>),
    Complete { dst: u8, value: Value16 },
    Discard,
}

pub(crate) enum MethodDispatchOutcome {
    Immediate(Value16),
    Deferred,
}

#[cold]
#[inline(never)]
fn continuation_error(message: String) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(message)
}

#[cold]
#[inline(never)]
pub(crate) fn deferred_call_without_request() -> hudhudscript_errors::Error {
    continuation_error(
        "VM call scheduling invariant: DeferredCall has no pending VmCallRequest".to_string(),
    )
}

#[cold]
#[inline(never)]
pub(crate) fn deferred_method_in_immediate_context(context: &str) -> hudhudscript_errors::Error {
    continuation_error(format!(
        "VM call scheduling invariant: deferred method reached synchronous {} path",
        context
    ))
}

impl VM {
    pub(crate) fn activate_receiver_context(&mut self, context: &mut ReceiverContext) {
        context.previous_this = self.get_var_cloned_by_sym(self.this_sym);
        self.cur_this = context.receiver;
        if let Some(class_sym) = context.class_sym {
            self.class_context_stack.push(class_sym);
        }
    }

    pub(crate) fn close_receiver_context(&mut self, context: ReceiverContext) {
        if context.write_back && self.cur_this != context.receiver {
            self.last_instance_mutation = Some(Box::new(self.cur_this));
        }
        self.cur_this = context
            .previous_this
            .unwrap_or_else(|| Value16::object(hudhudscript_bytecode::ObjMap::default()));
    }

    pub(crate) fn schedule_vm_call(&mut self, request: Box<VmCallRequest>) -> CompileResult<()> {
        if self.pending_vm_call.is_some() {
            return Err(continuation_error(
                "VM call scheduling invariant: pending_vm_call is already occupied".to_string(),
            ));
        }
        self.pending_vm_call = Some(request);
        Ok(())
    }

    /// G06A: schedule a user chunk for deferred execution from an
    /// instruction call site. Never opens a driver; the outer frame loop
    /// owns the transfer.
    pub(crate) fn schedule_deferred_chunk_call(
        &mut self,
        chunk: Arc<FunctionChunk>,
        func_sym: SymId,
        args: Vec<Value16>,
        captures: FxHashMap<String, Arc<RwLock<Value16>>>,
        receiver_context: Option<ReceiverContext>,
        call_site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
        let request = Box::new(VmCallRequest {
            chunk,
            func_sym,
            args,
            captures,
            dst: call_site.dst,
            origin_ip: call_site.origin_ip,
            receiver_context,
            return_sink: ReturnSink::Register(call_site.dst),
            swallow_error: false,
        });
        self.schedule_vm_call(request)?;
        Ok(MethodDispatchOutcome::Deferred)
    }

    /// G06A: store a continuation, bind it as the request's return sink,
    /// then enqueue the call. Order is contractual: the pending-slot check
    /// runs first so a failure never leaks a stored continuation.
    pub(crate) fn schedule_vm_call_with_continuation(
        &mut self,
        continuation: VmContinuation,
        mut request: Box<VmCallRequest>,
    ) -> CompileResult<()> {
        if self.pending_vm_call.is_some() {
            return Err(continuation_error(
                "VM call scheduling invariant: pending_vm_call is already occupied".to_string(),
            ));
        }
        let id = self.store_continuation(continuation);
        request.return_sink = ReturnSink::Continuation(id);
        self.pending_vm_call = Some(request);
        Ok(())
    }

    pub(crate) fn store_continuation(&mut self, continuation: VmContinuation) -> ContinuationId {
        let id = ContinuationId(self.vm_continuations.len());
        self.vm_continuations.push(continuation);
        id
    }

    /// G07-14: drop a continuation without resuming it. Used when a frame
    /// dies from a throw or runtime error — a pending continuation must
    /// never observe an error as its callee result.
    pub(crate) fn cancel_continuation(&mut self, id: ContinuationId) {
        let Some(slot) = self.vm_continuations.get_mut(id.0) else {
            return;
        };
        let continuation = std::mem::replace(slot, VmContinuation::Completed);
        if matches!(continuation, VmContinuation::AtomicTransactionAttempt(_)) {
            self.in_stm_context = false;
            self.current_tx = None;
        }
    }

    pub(crate) fn cancel_frame_continuation(&mut self, sink: ReturnSink) {
        if let ReturnSink::Continuation(id) = sink {
            self.cancel_continuation(id);
        }
    }

    /// Resume exactly one continuation step. This helper never enters a chunk
    /// driver; scheduling is returned to the existing outer frame loop.
    pub(crate) fn resume_continuation(
        &mut self,
        id: ContinuationId,
        value: Value16,
    ) -> CompileResult<ContinuationResume> {
        let slot = self.vm_continuations.get_mut(id.0).ok_or_else(|| {
            continuation_error(format!("VM continuation {} does not exist", id.0))
        })?;
        let continuation = std::mem::replace(slot, VmContinuation::Completed);

        // G06A: constructor completion rebuilds the instance from the
        // mutated receiver; the callee's return value is discarded.
        if let VmContinuation::ConstructorCall(state) = continuation {
            let dst = state.dst;
            let instance = self.finish_constructor_call(state);
            return Ok(ContinuationResume::Complete {
                dst,
                value: instance,
            });
        }

        if let VmContinuation::FunctionCallbackSequence(state) = continuation {
            return self.resume_array_callback(id, state, value);
        }

        if let VmContinuation::CustomIteratorSequence(state) = continuation {
            return self.resume_custom_iterator(id, state, value);
        }

        if let VmContinuation::SopAbilitySequence(state) = continuation {
            return self.resume_sop_ability(id, state, value);
        }

        if let VmContinuation::PromiseCallback(state) = continuation {
            return Ok(state.finish(value));
        }

        if let VmContinuation::GovernanceDispatch(state) = continuation {
            return Ok(state.finish(value));
        }

        if let VmContinuation::AtomicTransactionAttempt(state) = continuation {
            return self.resume_atomic_transaction_attempt(id, state, value);
        }

        Err(continuation_error(format!(
            "VM continuation {} was already completed",
            id.0
        )))
    }
}

mod array_callback;
mod atomic_transaction;
mod constructor;
mod custom_iterator;
mod governance_dispatch;
mod promise_callback;
mod sop_ability;

#[cfg(test)]
mod governance_dispatch_tests;

#[cfg(test)]
mod sop_ability_tests;

#[cfg(test)]
mod array_callback_tests;

#[cfg(test)]
mod custom_iterator_tests;

#[cfg(test)]
mod deep_chain_tests;

#[cfg(test)]
mod g07_tests;

#[cfg(test)]
mod promise_callback_tests;

#[cfg(test)]
mod single_call_tests;

#[cfg(test)]
mod tests;
