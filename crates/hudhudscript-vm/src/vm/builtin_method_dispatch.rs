//! Canonical receiver classification for method calls.
//!
//! Receiver-specific implementations live under `method_dispatch/`. This
//! module owns only their ordering and the public VM dispatch contract.

use crate::vm::call_state::{DeferredCallSite, MethodDispatchOutcome};
use crate::vm::VM;
use hudhudscript_bytecode::error::{compile_codes, CompileResult};
use hudhudscript_bytecode::{Bytecode, SymId, Value16};

impl VM {
    pub(crate) fn call_method_on_value(
        &mut self,
        receiver: &Value16,
        method: &str,
        method_sym: SymId,
        args: Vec<Value16>,
        bytecode: &Bytecode,
        call_site: DeferredCallSite,
    ) -> CompileResult<MethodDispatchOutcome> {
        if let Some(outcome) =
            self.dispatch_sop_method(receiver, method, method_sym, &args, bytecode, call_site)?
        {
            return Ok(outcome);
        }

        if receiver.as_instance_data().is_some() {
            return self.dispatch_instance_method(
                receiver, method, method_sym, &args, bytecode, call_site,
            );
        }

        if receiver.as_array().is_some() {
            return self.call_array_method(*receiver, method, args, bytecode, call_site);
        }

        if let Some(outcome) =
            self.dispatch_native_method(receiver, method, &args, bytecode, call_site)?
        {
            return Ok(outcome);
        }

        if receiver.as_object().is_some() {
            if let Some(outcome) =
                self.dispatch_agent_object_method(receiver, method, &args, bytecode, call_site)?
            {
                return Ok(outcome);
            }
            return self.dispatch_object_user_method(
                receiver, method, method_sym, &args, bytecode, call_site,
            );
        }

        Err(compile_codes::runtime_error(format!(
            "Cannot call method '{}' on {}",
            method,
            self.type_name_of(receiver)
        )))
    }
}
