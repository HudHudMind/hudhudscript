use super::{
    ContinuationId, ContinuationResume, DeferredCallSite, MethodDispatchOutcome, ReturnSink,
    VmCallRequest, VmContinuation,
};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{gc, Bytecode, DynamicObject, FunctionChunk, SymId, Value16};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArrayCallbackOperation {
    Map,
    Filter,
    Reduce,
    ForEach,
    Find,
    Some,
    Every,
}

impl ArrayCallbackOperation {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "map" => Some(Self::Map),
            "filter" => Some(Self::Filter),
            "reduce" => Some(Self::Reduce),
            "forEach" => Some(Self::ForEach),
            "find" => Some(Self::Find),
            "some" => Some(Self::Some),
            "every" => Some(Self::Every),
            _ => None,
        }
    }
}

pub(crate) struct FunctionCallbackSequence {
    pub(crate) operation: ArrayCallbackOperation,
    pub(crate) items: Vec<Value16>,
    pub(crate) callback: Value16,
    pub(crate) index: usize,
    pub(crate) accumulator: Option<Value16>,
    pub(crate) output: Vec<Value16>,
    pub(crate) dst: u8,
    pub(crate) origin_ip: usize,
    chunk: Arc<FunctionChunk>,
    func_sym: SymId,
    captures: FxHashMap<String, Arc<RwLock<Value16>>>,
}

impl FunctionCallbackSequence {
    fn callback_args(&self) -> CompileResult<Vec<Value16>> {
        let item = self.items.get(self.index).copied().ok_or_else(|| {
            callback_error(format!(
                "array callback index {} is out of bounds for {} items",
                self.index,
                self.items.len()
            ))
        })?;
        let index = Value16::number(self.index as f64);
        if self.operation == ArrayCallbackOperation::Reduce {
            let accumulator = self
                .accumulator
                .ok_or_else(|| callback_error("reduce callback has no accumulator".to_string()))?;
            Ok(vec![accumulator, item, index])
        } else {
            Ok(vec![item, index])
        }
    }

    fn request(&self, return_sink: ReturnSink) -> CompileResult<Box<VmCallRequest>> {
        Ok(Box::new(VmCallRequest {
            chunk: Arc::clone(&self.chunk),
            func_sym: self.func_sym,
            args: self.callback_args()?,
            captures: self
                .captures
                .iter()
                .map(|(name, cell)| (name.clone(), Arc::clone(cell)))
                .collect(),
            dst: self.dst,
            origin_ip: self.origin_ip,
            receiver_context: None,
            return_sink,
            swallow_error: false,
        }))
    }

    fn final_value(&self) -> CompileResult<Value16> {
        match self.operation {
            ArrayCallbackOperation::Map | ArrayCallbackOperation::Filter => {
                Ok(Value16::array(self.output.clone()))
            }
            ArrayCallbackOperation::Reduce => self.accumulator.ok_or_else(|| {
                callback_error("reduce sequence completed without an accumulator".to_string())
            }),
            ArrayCallbackOperation::ForEach => Ok(Value16::null()),
            ArrayCallbackOperation::Find => Ok(Value16::null()),
            ArrayCallbackOperation::Some => Ok(Value16::boolean(false)),
            ArrayCallbackOperation::Every => Ok(Value16::boolean(true)),
        }
    }

    pub(super) fn trace_roots(&self, gray: &mut Vec<*mut DynamicObject>) {
        gc::trace_value(self.callback, gray);
        for value in &self.items {
            gc::trace_value(*value, gray);
        }
        if let Some(accumulator) = self.accumulator {
            gc::trace_value(accumulator, gray);
        }
        for value in &self.output {
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
fn callback_error(message: String) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(message)
}

impl VM {
    pub(crate) fn start_array_callback_sequence(
        &mut self,
        items: Vec<Value16>,
        method: &str,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
        let operation = ArrayCallbackOperation::from_name(method).ok_or_else(|| {
            callback_error(format!("{} is not a callback-based array method", method))
        })?;
        let callback = args
            .first()
            .copied()
            .ok_or_else(|| callback_error(format!("{} requires a callback argument", method)))?;
        let (index, accumulator) = if operation == ArrayCallbackOperation::Reduce {
            if let Some(initial) = args.get(1).copied() {
                (0, Some(initial))
            } else if let Some(first) = items.first().copied() {
                (1, Some(first))
            } else {
                return Err(callback_error(
                    "reduce of empty array with no initial value".to_string(),
                ));
            }
        } else {
            (0, None)
        };
        let output = if matches!(
            operation,
            ArrayCallbackOperation::Map | ArrayCallbackOperation::Filter
        ) {
            Vec::with_capacity(items.len())
        } else {
            Vec::new()
        };

        if index >= items.len() {
            let value = match operation {
                ArrayCallbackOperation::Map | ArrayCallbackOperation::Filter => {
                    Value16::array(output)
                }
                ArrayCallbackOperation::Reduce => accumulator.ok_or_else(|| {
                    callback_error("reduce sequence completed without an accumulator".to_string())
                })?,
                ArrayCallbackOperation::ForEach | ArrayCallbackOperation::Find => Value16::null(),
                ArrayCallbackOperation::Some => Value16::boolean(false),
                ArrayCallbackOperation::Every => Value16::boolean(true),
            };
            return Ok(MethodDispatchOutcome::Immediate(value));
        }

        let function = callback
            .as_function_data()
            .ok_or_else(|| callback_error(format!("Expected function, got {:?}", callback)))?;
        let chunk = bytecode.get_function(&function.chunk_name).ok_or_else(|| {
            callback_error(format!("Function chunk not found: {}", function.chunk_name))
        })?;
        let captures: FxHashMap<String, Arc<RwLock<Value16>>> = function
            .captures
            .iter()
            .map(|(name, cell)| (name.clone(), Arc::clone(cell)))
            .collect();
        let state = FunctionCallbackSequence {
            operation,
            items,
            callback,
            index,
            accumulator,
            output,
            dst: call_site.dst,
            origin_ip: call_site.origin_ip,
            chunk,
            func_sym: SymId(function.chunk_sym),
            captures,
        };

        let request = state.request(ReturnSink::Discard)?;
        self.schedule_vm_call_with_continuation(
            VmContinuation::FunctionCallbackSequence(state),
            request,
        )?;
        Ok(MethodDispatchOutcome::Deferred)
    }

    pub(crate) fn start_array_callback_builtin(
        &mut self,
        method: &str,
        arg_count: u8,
        first_arg: u8,
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
        let mut args: Vec<Value16> = (0..arg_count as usize)
            .map(|index| self.registers[first_arg as usize + index])
            .collect();
        if args.is_empty() {
            return Err(callback_error(format!(
                "{}() requires at least 1 argument",
                method
            )));
        }
        let receiver = args.remove(0);
        let items = receiver.as_array().cloned().ok_or_else(|| {
            callback_error(format!(
                "{}() requires an array as first argument, got {}",
                method,
                Self::bytecode_value_type_name(&receiver)
            ))
        })?;
        self.start_array_callback_sequence(items, method, &args, bytecode, call_site)
    }

    pub(super) fn resume_array_callback(
        &mut self,
        id: ContinuationId,
        mut state: FunctionCallbackSequence,
        callback_result: Value16,
    ) -> CompileResult<ContinuationResume> {
        let item = state.items.get(state.index).copied().ok_or_else(|| {
            callback_error(format!(
                "array callback index {} is out of bounds for {} items",
                state.index,
                state.items.len()
            ))
        })?;

        match state.operation {
            ArrayCallbackOperation::Map => state.output.push(callback_result),
            ArrayCallbackOperation::Filter => {
                if callback_result.is_truthy() {
                    state.output.push(item);
                }
            }
            ArrayCallbackOperation::Reduce => state.accumulator = Some(callback_result),
            ArrayCallbackOperation::ForEach => {}
            ArrayCallbackOperation::Find if callback_result.is_truthy() => {
                return Ok(ContinuationResume::Complete {
                    dst: state.dst,
                    value: item,
                });
            }
            ArrayCallbackOperation::Some if callback_result.is_truthy() => {
                return Ok(ContinuationResume::Complete {
                    dst: state.dst,
                    value: Value16::boolean(true),
                });
            }
            ArrayCallbackOperation::Every if !callback_result.is_truthy() => {
                return Ok(ContinuationResume::Complete {
                    dst: state.dst,
                    value: Value16::boolean(false),
                });
            }
            ArrayCallbackOperation::Find
            | ArrayCallbackOperation::Some
            | ArrayCallbackOperation::Every => {}
        }

        state.index += 1;
        if state.index >= state.items.len() {
            return Ok(ContinuationResume::Complete {
                dst: state.dst,
                value: state.final_value()?,
            });
        }

        let request = state.request(ReturnSink::Continuation(id))?;
        self.vm_continuations[id.0] = VmContinuation::FunctionCallbackSequence(state);
        Ok(ContinuationResume::Schedule(request))
    }
}
