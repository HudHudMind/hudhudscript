use crate::vm::VM;
use hudhudscript_bytecode::Value16;
use hudhudscript_bytecode::{Bytecode, FunctionChunk};
use std::collections::HashMap;
use std::sync::Arc;

impl crate::vm::VM {
    pub fn register_promise(
        &mut self,
        id: String,
        receiver: std::sync::mpsc::Receiver<Result<Value16, String>>,
    ) {
        self.promise_registry
            .register_external_with_id(id, receiver);
    }

    /// Register an externally-owned receiver and return the id it was
    /// filed under in the shared registry. Prefer this over the legacy
    /// `register_promise` wrapper.
    pub fn register_promise_owned(
        &mut self,
        receiver: std::sync::mpsc::Receiver<Result<Value16, String>>,
    ) -> String {
        self.promise_registry.register_external(receiver)
    }

    /// Store a pre-resolved promise result under an explicit id. Useful
    /// when the async work has already completed before the VM reaches
    /// the `await` point and the id was minted elsewhere.
    pub fn store_promise_result(&mut self, id: String, result: Result<Value16, String>) {
        self.promise_registry.store_result_with_id(id, result);
    }

    /// Spawn an async closure on a background thread and return an
    /// `AsyncPending` promise value. The id is minted by the shared
    /// `PromiseRegistry`; the detached receiver is retained by the VM so
    /// the value can be attached to the owning heap after settlement.
    pub fn spawn_async_task<F>(&mut self, task: F) -> Value16
    where
        F: FnOnce() -> Result<Value16, String> + Send + 'static,
    {
        // H-BLOCKING: detach in thread, attach on await
        use hudhudscript_bytecode::gc_detach;
        let (tx, rx) = std::sync::mpsc::channel::<Result<gc_detach::DetachedGraph, String>>();
        let id = self.promise_registry.next_id();
        self.detached_promises.insert(id.clone(), rx);
        std::thread::spawn(move || {
            let result = match task() {
                Ok(v) => match gc_detach::detach(v) {
                    Ok(tree) => Ok(tree),
                    Err(e) => Err(e.to_string()),
                },
                Err(msg) => Err(msg),
            };
            let _ = tx.send(result);
        });
        Value16::promise(hudhudscript_bytecode::PromiseState16::AsyncPending(id))
    }

    /// Reduce a `Value::Promise` (in any state) or a non-promise value
    /// to a concrete resolution result, blocking on either the VM's detached
    /// receiver transport or the shared registry for `AsyncPending` entries.
    ///
    /// Returns `Ok(value)` on resolution, `Err(message)` on rejection.
    /// Used by the VM's Promise.all / Promise.race / Promise.allSettled
    /// builtins so they never silently fall through to a bare Pending
    /// when handed an AsyncPending array element (Kural 7b).
    pub(crate) fn resolve_promise_value(&mut self, value: Value16) -> Result<Value16, String> {
        if let Some(ps) = value.as_promise_state() {
            match ps {
                hudhudscript_bytecode::PromiseState16::Resolved(inner) => Ok(**inner),
                hudhudscript_bytecode::PromiseState16::Rejected(msg) => Err(msg.clone()),
                hudhudscript_bytecode::PromiseState16::Pending => {
                    Err("Cannot resolve a bare Pending promise".to_string())
                }
                hudhudscript_bytecode::PromiseState16::AsyncPending(id) => {
                    // H-BLOCKING: Check detached promises first (V2-E detach/attach)
                    if let Some(rx) = self.detached_promises.remove(id) {
                        match rx.recv() {
                            Ok(Ok(tree)) => Ok(hudhudscript_bytecode::gc_detach::attach(&tree)),
                            Ok(Err(msg)) => Err(msg),
                            Err(_) => Err("blocking task sender dropped".to_string()),
                        }
                    } else {
                        match self.promise_registry.await_blocking(id) {
                            Ok(val) => Ok(val),
                            Err(hudhudscript_async::RegistryError::Rejected(msg)) => Err(msg),
                            Err(e) => Err(format!("{}", e)),
                        }
                    }
                }
            }
        } else {
            // Non-promise values are treated as already-resolved (JS semantics).
            Ok(value)
        }
    }

    /// Spawn an async function chunk on a separate thread, returning an
    /// `AsyncPending` promise. A fresh VM is created for the spawned task,
    /// inheriting the caller's global scope, classes, and declarations so
    /// that captured variables and class hierarchies remain accessible.
    ///
    /// The result is consumed by the `Await` instruction handler through the
    /// VM's detached receiver transport (Kural 7).
    pub(crate) fn spawn_async_chunk(
        &mut self,
        chunk: Arc<FunctionChunk>,
        params: &[String],
        args: &[Value16],
        bytecode: &Bytecode,
        func_name: &str,
        closure_captures: Option<&HashMap<String, Arc<parking_lot::RwLock<Value16>>>>,
    ) -> Value16 {
        // Prepare data for the spawned task: Arc::clone is O(1) vs deep clone.
        let chunk_arc = Arc::clone(&chunk);
        let params_clone: Vec<String> = params.to_vec();
        let args_clone = args.to_vec();
        let bytecode_clone = bytecode.clone();
        let name_clone = func_name.to_string();
        // Captures use Arc<RwLock<Value16>> — cloning the HashMap only bumps refcounts, no deep copy.
        let captures_clone: HashMap<String, Arc<parking_lot::RwLock<Value16>>> =
            closure_captures.cloned().unwrap_or_default();

        // Snapshot the caller's global namespace so the async function can
        // read top-level bindings (functions, constants, classes, etc.).
        let global_scope = self.globals.clone();
        let classes_clone = self.classes.clone();
        let declarations_clone = self.declarations.clone();

        // H-BLOCKING: Detach değerleri thread heap'inden çıkar, DetachedGraph taşınır.
        // Main thread await'te attach ile kendi heap'ine alır.
        use hudhudscript_bytecode::gc_detach;
        let (tx, rx) = std::sync::mpsc::channel::<Result<gc_detach::DetachedGraph, String>>();
        let id = self.promise_registry.next_id();
        self.detached_promises.insert(id.clone(), rx);

        std::thread::spawn(move || {
            let mut task_vm = VM::new();
            for (k, v) in global_scope {
                task_vm.globals.entry(k).or_insert(v);
            }
            task_vm.classes = classes_clone;
            task_vm.declarations = declarations_clone;
            // run attached chunk, capture result
            let func_sym = hudhudscript_bytecode::SymId(
                hudhudscript_bytecode::interner::intern(&name_clone).0,
            );
            let raw_result = task_vm.call_chunk_with_captures(
                &chunk_arc,
                &params_clone,
                &args_clone,
                &bytecode_clone,
                func_sym,
                &captures_clone,
            );
            let detached = match raw_result {
                Ok(val) => match gc_detach::detach(val) {
                    Ok(tree) => Ok(tree),
                    Err(e) => Err(e.to_string()),
                },
                Err(e) => Err(format!("{}", e)),
            };
            let _ = tx.send(detached);
        });

        Value16::promise(hudhudscript_bytecode::PromiseState16::AsyncPending(id))
    }
}
