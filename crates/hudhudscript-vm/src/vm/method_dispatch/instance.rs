use crate::vm::call_state::{DeferredCallSite, MethodDispatchOutcome, ReceiverContext};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, SymId, Value16};
use rustc_hash::FxHashMap;

#[cold]
#[inline(never)]
fn vtable_not_packed() -> hudhudscript_errors::Error {
    compile_codes::runtime_error("Compiler invariant: vtable value not packed int".to_string())
}

#[cold]
#[inline(never)]
pub(crate) fn function_index_missing(index: u32) -> hudhudscript_errors::Error {
    compile_codes::runtime_error(format!(
        "Compiler invariant: function idx {} missing",
        index
    ))
}

impl VM {
    pub(crate) fn dispatch_instance_method(
        &mut self,
        receiver: &Value16,
        method: &str,
        method_sym: SymId,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
        let instance = receiver.as_instance_data().ok_or_else(|| {
            compile_codes::runtime_error("Instance dispatch received a non-instance".to_string())
        })?;
        let class_name = instance.class_name.clone();

        if let Some(class_data) = instance.class.as_class_data() {
            if let Some(method_value) = class_data
                .vtable
                .get(&method_sym)
                .or_else(|| class_data.methods.get(&method_sym))
            {
                let packed = method_value.as_int().ok_or_else(vtable_not_packed)?;
                let function_index = (packed >> 32) as u32;
                let function_sym = SymId(packed as u32);
                let chunk = bytecode
                    .get_function_by_index(function_index)
                    .ok_or_else(|| function_index_missing(function_index))?;
                let class_sym = SymId(hudhudscript_bytecode::interner::intern(&class_name).0);

                if let Some(access) = class_data.method_access.get(&method_sym) {
                    if *access == 1 && self.class_context_stack.last().copied() != Some(class_sym) {
                        return Err(compile_codes::runtime_error(format!(
                            "Cannot call private method '{}' on class '{}' from outside the class",
                            method, class_name
                        )));
                    }
                    if *access == 2 {
                        let current = self.class_context_stack.last().copied();
                        let current_name = current.map(|symbol| {
                            hudhudscript_bytecode::interner::resolve(
                                hudhudscript_bytecode::interner::SymbolId(symbol.0),
                            )
                        });
                        let allowed = current == Some(class_sym)
                            || self.is_subclass_of(current_name.as_deref(), &class_name);
                        if !allowed {
                            return Err(compile_codes::runtime_error(format!(
                                "Cannot call protected method '{}' on class '{}' from unrelated class",
                                method, class_name
                            )));
                        }
                    }
                }

                let context = ReceiverContext::new(*receiver, Some(class_sym), false);
                return self.schedule_deferred_chunk_call(
                    chunk,
                    function_sym,
                    args.to_vec(),
                    FxHashMap::default(),
                    Some(context),
                    call_site,
                );
            }
        }

        if let Some(chunk_name) = instance
            .fields
            .get(&method_sym)
            .and_then(|value| value.as_string())
        {
            if let Some(chunk) = bytecode.get_function(&chunk_name) {
                let function_sym = SymId(hudhudscript_bytecode::interner::intern(&chunk_name).0);
                let context = ReceiverContext::new(*receiver, None, false);
                return self.schedule_deferred_chunk_call(
                    chunk,
                    function_sym,
                    args.to_vec(),
                    FxHashMap::default(),
                    Some(context),
                    call_site,
                );
            }
        }

        match crate::vm::builtin_method::lookup_method(method_sym) {
            Some(crate::vm::builtin_method::BuiltinMethod::Keys) => {
                let mut keys: Vec<Value16> = instance
                    .fields
                    .keys()
                    .map(|key| Value16::string(key.to_string()))
                    .collect();
                keys.sort_by_cached_key(|value| value.as_string().unwrap_or_default());
                Ok(MethodDispatchOutcome::Immediate(Value16::array(keys)))
            }
            Some(crate::vm::builtin_method::BuiltinMethod::Values) => {
                let mut pairs: Vec<(String, Value16)> = instance
                    .fields
                    .iter()
                    .map(|(key, value)| (key.to_string(), *value))
                    .collect();
                pairs.sort_by(|left, right| left.0.cmp(&right.0));
                Ok(MethodDispatchOutcome::Immediate(Value16::array(
                    pairs.into_iter().map(|(_, value)| value).collect(),
                )))
            }
            Some(crate::vm::builtin_method::BuiltinMethod::Length) => Ok(
                MethodDispatchOutcome::Immediate(Value16::int(instance.fields.len() as i64)),
            ),
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown method '{}' on instance of {}",
                method, class_name
            ))),
        }
    }

    pub(crate) fn dispatch_object_user_method(
        &mut self,
        receiver: &Value16,
        method: &str,
        method_sym: SymId,
        args: &[Value16],
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
        let object = receiver.as_object().ok_or_else(|| {
            compile_codes::runtime_error("Object dispatch received a non-object".to_string())
        })?;

        if let Some(chunk_name) = object.get(&method_sym).and_then(|value| value.as_string()) {
            if let Some(chunk) = bytecode.get_function(&chunk_name) {
                let function_sym = SymId(hudhudscript_bytecode::interner::intern(&chunk_name).0);
                let context = ReceiverContext::new(*receiver, None, false);
                return self.schedule_deferred_chunk_call(
                    chunk,
                    function_sym,
                    args.to_vec(),
                    FxHashMap::default(),
                    Some(context),
                    call_site,
                );
            }
        }
        if let Some(function) = Self::property_function_value(object.get(&method_sym)) {
            return self.call_property_function(
                receiver,
                function,
                args.to_vec(),
                bytecode,
                call_site,
            );
        }

        match crate::vm::builtin_method::lookup_method(method_sym) {
            Some(crate::vm::builtin_method::BuiltinMethod::Keys) => {
                let mut keys: Vec<Value16> = object
                    .keys()
                    .map(|key| Value16::string(key.to_string()))
                    .collect();
                keys.sort_by_cached_key(|value| value.as_string().unwrap_or_default());
                Ok(MethodDispatchOutcome::Immediate(Value16::array(keys)))
            }
            Some(crate::vm::builtin_method::BuiltinMethod::Values) => {
                let mut pairs: Vec<(String, Value16)> = object
                    .iter()
                    .map(|(key, value)| (key.to_string(), *value))
                    .collect();
                pairs.sort_by(|left, right| left.0.cmp(&right.0));
                Ok(MethodDispatchOutcome::Immediate(Value16::array(
                    pairs.into_iter().map(|(_, value)| value).collect(),
                )))
            }
            Some(crate::vm::builtin_method::BuiltinMethod::Length) => Ok(
                MethodDispatchOutcome::Immediate(Value16::int(object.len() as i64)),
            ),
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown method '{}' on object",
                method
            ))),
        }
    }
}
