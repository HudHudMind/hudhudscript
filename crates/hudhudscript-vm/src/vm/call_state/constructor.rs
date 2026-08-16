//! G06A constructor scheduling and completion.
//!
//! `NewInstance` no longer opens a nested driver for the constructor
//! chunk. It schedules the chunk with a receiver context and a
//! [`ConstructorContinuation`]; when the deferred frame returns, the
//! continuation rebuilds the instance from the mutated receiver (or its
//! write-back) and discards the constructor's own return value.

use super::{ConstructorContinuation, ReceiverContext, ReturnSink, VmCallRequest, VmContinuation};
use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{FunctionChunk, InstanceData, ObjMap, SymId, Value16};
use rustc_hash::FxHashMap;
use std::sync::Arc;

impl VM {
    /// Schedule the constructor chunk as a deferred call. The pending-slot
    /// invariant is enforced inside `schedule_vm_call_with_continuation`
    /// before any continuation is stored.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn schedule_constructor_call(
        &mut self,
        chunk: Arc<FunctionChunk>,
        ctor_sym: SymId,
        args: Vec<Value16>,
        class_name: String,
        class_value: Value16,
        receiver: Value16,
        origin_ip: usize,
    ) -> CompileResult<()> {
        let class_sym = SymId(hudhudscript_bytecode::interner::intern(&class_name).0);
        let continuation = VmContinuation::ConstructorCall(ConstructorContinuation {
            dst: 255,
            class_name,
            class_value,
            receiver,
        });
        let request = Box::new(VmCallRequest {
            chunk,
            func_sym: ctor_sym,
            args,
            captures: FxHashMap::default(),
            dst: 255,
            origin_ip,
            receiver_context: Some(ReceiverContext::new(receiver, Some(class_sym), true)),
            return_sink: ReturnSink::Discard,
            swallow_error: false,
        });
        self.schedule_vm_call_with_continuation(continuation, request)
    }

    /// Runs when a deferred constructor frame returns. Mirrors
    /// `collect_this_fields` precedence: the write-back `this` wins, then
    /// the in-place-mutated receiver, then `this.field` composite globals.
    pub(crate) fn finish_constructor_call(
        &mut self,
        continuation: ConstructorContinuation,
    ) -> Value16 {
        let ConstructorContinuation {
            class_name,
            class_value,
            receiver,
            ..
        } = continuation;

        let mutated_fields = self
            .last_instance_mutation
            .take()
            .and_then(|mutated| fields_of(*mutated));
        let mut instance: ObjMap = mutated_fields
            .or_else(|| fields_of(receiver))
            .unwrap_or_default();

        // "this.field" composite-key globals (collect_this_fields parity).
        for (key, value) in self.globals.iter() {
            let key_name = hudhudscript_bytecode::interner::resolve(*key);
            if let Some(field) = key_name.strip_prefix("this.") {
                instance.insert(field.to_string(), value.clone());
            }
        }

        Value16::instance(InstanceData {
            class_name,
            fields: instance.into_iter().collect(),
            class: class_value,
        })
    }
}

fn fields_of(value: Value16) -> Option<ObjMap> {
    if let Some(object) = value.as_object() {
        return Some(object.iter().map(|(k, v)| (k.clone(), *v)).collect());
    }
    value
        .as_instance_data()
        .map(|inst| inst.fields.iter().map(|(k, v)| (k.clone(), *v)).collect())
}
