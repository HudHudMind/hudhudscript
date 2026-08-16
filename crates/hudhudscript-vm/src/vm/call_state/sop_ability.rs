use super::{ContinuationId, ContinuationResume, ReturnSink, VmCallRequest, VmContinuation};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{gc, DynamicObject, FunctionChunk, SymId, Value16};
use std::sync::Arc;

pub(crate) enum SopResultPolicy {
    Ignore,
    Replace,
}

pub(crate) struct SopCallStep {
    pub(crate) chunk: Arc<FunctionChunk>,
    pub(crate) func_sym: SymId,
    pub(crate) result_policy: SopResultPolicy,
    pub(crate) swallow_error: bool,
}

pub(crate) struct SopAbilitySequence {
    pub(crate) steps: Vec<SopCallStep>,
    pub(crate) index: usize,
    pub(crate) args: Vec<Value16>,
    pub(crate) result: Option<Value16>,
    pub(crate) dst: u8,
    pub(crate) origin_ip: usize,
}

impl SopAbilitySequence {
    fn request(&self, sink: ReturnSink) -> Box<VmCallRequest> {
        let step = &self.steps[self.index];
        Box::new(VmCallRequest {
            chunk: Arc::clone(&step.chunk),
            func_sym: step.func_sym,
            args: self.args.clone(),
            captures: rustc_hash::FxHashMap::default(),
            dst: self.dst,
            origin_ip: self.origin_ip,
            receiver_context: None,
            return_sink: sink,
            swallow_error: step.swallow_error,
        })
    }

    pub(super) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        for value in &self.args {
            gc::trace_value(*value, gray);
        }
        if let Some(result) = self.result {
            gc::trace_value(result, gray);
        }
        for step in &self.steps {
            for value in &step.chunk.constants {
                gc::trace_value(*value, gray);
            }
        }
    }
}

#[cold]
#[inline(never)]
fn sop_error(message: String) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(message)
}

impl VM {
    pub(crate) fn start_sop_ability_sequence(
        &mut self,
        steps: Vec<SopCallStep>,
        args: Vec<Value16>,
        dst: u8,
        origin_ip: usize,
    ) -> CompileResult<super::MethodDispatchOutcome> {
        let state = SopAbilitySequence {
            steps,
            index: 0,
            args,
            result: None,
            dst,
            origin_ip,
        };
        let request = state.request(ReturnSink::Discard);
        self.schedule_vm_call_with_continuation(
            VmContinuation::SopAbilitySequence(state),
            request,
        )?;
        Ok(super::MethodDispatchOutcome::Deferred)
    }

    pub(super) fn resume_sop_ability(
        &mut self,
        id: ContinuationId,
        mut state: SopAbilitySequence,
        value: Value16,
    ) -> CompileResult<ContinuationResume> {
        if matches!(
            state.steps[state.index].result_policy,
            SopResultPolicy::Replace
        ) {
            state.result = Some(value);
        }
        state.index += 1;
        if state.index == state.steps.len() {
            let value = state
                .result
                .ok_or_else(|| sop_error("SOP sequence completed without a result".to_string()))?;
            return Ok(ContinuationResume::Complete {
                dst: state.dst,
                value,
            });
        }
        let request = state.request(ReturnSink::Continuation(id));
        self.vm_continuations[id.0] = VmContinuation::SopAbilitySequence(state);
        Ok(ContinuationResume::Schedule(request))
    }
}
