use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
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
    /// `PromiseRegistry`; the `Await` instruction will later block on
    /// the result through that registry.
    pub fn spawn_async_task<F>(&mut self, task: F) -> Value16
    where
        F: FnOnce() -> Result<Value16, String> + Send + 'static,
    {
        let id = self.promise_registry.spawn_task(task);
        Value16::promise(hudhudscript_bytecode::PromiseState16::AsyncPending(id))
    }

    /// Reduce a `Value::Promise` (in any state) or a non-promise value
    /// to a concrete resolution result, blocking on the shared
    /// `PromiseRegistry` for `AsyncPending` entries.
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
                    match self.promise_registry.await_blocking(id) {
                        Ok(val) => Ok(val),
                        Err(hudhudscript_async::RegistryError::Rejected(msg)) => Err(msg),
                        Err(e) => Err(format!("{}", e)),
                    }
                }
            }
        } else {
            // Non-promise values are treated as already-resolved (JS semantics).
            Ok(value)
        }
    }

    /// Concurrent `Promise.all` implementation, matching the
    /// interpreter's `eval_promise_all_async` semantics (P1-4).
    ///
    /// Behaviour:
    /// * Already-`Resolved` / non-promise entries contribute their value
    ///   immediately.
    /// * Already-`Rejected` entries short-circuit into `Err(msg)`.
    /// * Bare `Pending` entries short-circuit into `Err("Cannot resolve
    ///   a bare Pending promise")` — they can never settle on the VM
    ///   which is tokio-free, so the caller's only option is to reject.
    /// * `AsyncPending` entries are awaited concurrently via
    ///   `PromiseRegistry::await_all_blocking`, so the wall-clock cost
    ///   is `max(duration_i)` rather than `sum(duration_i)`.
    ///
    /// Returns `Ok(values)` with the results in input order, or
    /// `Err(msg)` on the first rejection encountered. Non-rejection
    /// resolution errors from the registry are surfaced via their
    /// `Display` impl.
    pub(crate) fn resolve_promise_all(
        &mut self,
        promises: Vec<Value16>,
    ) -> Result<Vec<Value16>, String> {
        let n = promises.len();
        // Resolved-or-later slots, plus a parallel list of
        // (slot_index, pending_id) for the ids that still need to be
        // awaited concurrently.
        let mut slots: Vec<Option<Value16>> = (0..n).map(|_| None).collect();
        let mut pending: Vec<(usize, String)> = Vec::new();

        for (idx, p) in promises.into_iter().enumerate() {
            if let Some(ps) = p.as_promise_state() {
                match ps {
                    hudhudscript_bytecode::PromiseState16::Resolved(inner) => {
                        slots[idx] = Some(**inner);
                    }
                    hudhudscript_bytecode::PromiseState16::Rejected(msg) => {
                        return Err(msg.clone());
                    }
                    hudhudscript_bytecode::PromiseState16::Pending => {
                        return Err("Cannot resolve a bare Pending promise".to_string());
                    }
                    hudhudscript_bytecode::PromiseState16::AsyncPending(id) => {
                        pending.push((idx, id.clone()));
                    }
                }
            } else {
                // Non-promise values behave as immediately resolved.
                slots[idx] = Some(p);
            }
        }

        if !pending.is_empty() {
            let id_refs: Vec<&str> = pending.iter().map(|(_, id)| id.as_str()).collect();
            match self.promise_registry.await_all_blocking(&id_refs) {
                Ok(values) => {
                    // Zip results back into their original slots.
                    for ((slot_idx, _id), value) in pending.into_iter().zip(values.into_iter()) {
                        slots[slot_idx] = Some(value);
                    }
                }
                Err(hudhudscript_async::RegistryError::Rejected(msg)) => {
                    return Err(msg);
                }
                Err(e) => {
                    return Err(format!("{}", e));
                }
            }
        }

        Ok(slots
            .into_iter()
            .map(|s| s.unwrap_or(Value16::null()))
            .collect())
    }

    /// Concurrent `Promise.race` implementation, matching the
    /// interpreter's `eval_promise_race_async` semantics (P1-3).
    ///
    /// Behaviour:
    /// * If any entry is already `Resolved`, `Rejected`, `Pending`, or a
    ///   non-promise value, the *first* such entry in input order wins
    ///   immediately — matching JS's "first settled" rule as applied to
    ///   synchronously-known states.
    /// * Otherwise every entry is an `AsyncPending`. They are raced
    ///   concurrently via `PromiseRegistry::await_race_blocking`; the
    ///   earliest completion (resolve or reject) is returned.
    /// * Empty input surfaces `Err("Promise.race() on empty array")`.
    ///
    /// Returns `Ok(value)` on resolution or `Err(msg)` on rejection.
    pub(crate) fn resolve_promise_race(
        &mut self,
        promises: Vec<Value16>,
    ) -> Result<Value16, String> {
        if promises.is_empty() {
            return Err("Promise.race() on empty array".to_string());
        }

        // First pass: any synchronously-known settlement wins, in input order.
        let mut async_ids: Vec<String> = Vec::new();
        for p in &promises {
            if let Some(ps) = p.as_promise_state() {
                match ps {
                    hudhudscript_bytecode::PromiseState16::Resolved(inner) => {
                        return Ok(**inner);
                    }
                    hudhudscript_bytecode::PromiseState16::Rejected(msg) => {
                        return Err(msg.clone());
                    }
                    hudhudscript_bytecode::PromiseState16::Pending => {
                        // Bare Pending never settles on the VM; treat it as
                        // immediate rejection so the race does not hang.
                        return Err("Cannot resolve a bare Pending promise".to_string());
                    }
                    hudhudscript_bytecode::PromiseState16::AsyncPending(id) => {
                        async_ids.push(id.clone());
                    }
                }
            } else {
                // Non-promise values are "already resolved" in JS. First
                // such value wins.
                return Ok(*p);
            }
        }

        // All entries were AsyncPending — race them concurrently.
        let id_refs: Vec<&str> = async_ids.iter().map(|s| s.as_str()).collect();
        match self.promise_registry.await_race_blocking(&id_refs) {
            Ok((_idx, val)) => Ok(val),
            Err(hudhudscript_async::RegistryError::Rejected(msg)) => Err(msg),
            Err(e) => Err(format!("{}", e)),
        }
    }

    /// Spawn an async function chunk on a separate thread, returning an
    /// `AsyncPending` promise. A fresh VM is created for the spawned task,
    /// inheriting the caller's global scope, classes, and declarations so
    /// that captured variables and class hierarchies remain accessible.
    ///
    /// The result is consumed by the `Await` instruction handler via the
    /// shared `PromiseRegistry` (Kural 7).
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

        // Delegate thread-spawn + channel wiring to the shared registry.
        // The closure owns everything it needs; the spawned VM runs the
        // function body and returns the result (or an error string) to the
        // registry, which relays it to the matching `Await` later.
        let id = self.promise_registry.spawn_task(move || {
            let mut task_vm = VM::new();
            // Overlay the caller's globals onto the task VM's globals so
            // top-level bindings (other functions, constants) are visible.
            for (k, v) in global_scope {
                task_vm.globals.entry(k).or_insert(v);
            }
            task_vm.classes = classes_clone;
            task_vm.declarations = declarations_clone;

            let result = task_vm.call_chunk_with_captures(
                &chunk_arc,
                &params_clone,
                &args_clone,
                &bytecode_clone,
                &name_clone,
                &captures_clone,
            );
            match result {
                Ok(val) => Ok(val),
                Err(e) => Err(format!("{}", e)),
            }
        });

        Value16::promise(hudhudscript_bytecode::PromiseState16::AsyncPending(id))
    }
}
