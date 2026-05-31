use crate::vm::VM;
use hudhudscript_bytecode::Bytecode;
use hudhudscript_bytecode::Value16;

/// Adapter that lets `hudhudscript_bytecode::shared_value::CallbackInvoker` drive
/// the VM's closure-dispatch machinery (Kural 7 — shared array callbacks).
///
/// The shared array/set/map callback functions need to invoke a
/// user-supplied script callback against each element of a collection. On
/// the interpreter side `Interpreter` impls `CallbackInvoker` directly and
/// forwards to `call_function`. The VM needs an extra piece of context —
/// the current `Bytecode` — to find the function chunk referenced by the
/// closure, so we bundle `(&mut VM, &Bytecode)` in this short-lived wrapper
/// and impl `CallbackInvoker` on it instead. Constructing the wrapper at
/// the call site re-borrows `self` for the duration of the shared
/// dispatcher call, which is exactly the lifetime scope the borrow checker
/// needs.
pub(crate) struct VmCallbackInvoker<'a> {
    pub(crate) vm: &'a mut VM,
    pub(crate) bytecode: &'a Bytecode,
}

impl<'a> hudhudscript_bytecode::shared_value::CallbackInvoker for VmCallbackInvoker<'a> {
    fn invoke(
        &mut self,
        callback: &Value16,
        args: Vec<Value16>,
    ) -> hudhudscript_errors::HudHudResult<Value16> {
        self.vm
            .call_value_as_function(callback, args, self.bytecode)
    }
}
