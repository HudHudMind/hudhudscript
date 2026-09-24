//! Embedder-facing promise accessors for the FFI bridge (Flutter/Dart).
//!
//! Additive-only extension: this file opens a NEW `impl VM` block and does
//! not touch any existing code path. It exists because the FFI layer must
//! resolve an `AsyncPending` promise id from OUTSIDE the interpreter loop
//! (e.g. a Dart button handler awaiting a script async function), while the
//! canonical resolver `VM::resolve_promise_value` is `pub(crate)`.
//!
//! Transport parity with the Await instruction is mandatory: detached
//! receivers (async-function bodies spawned on their own thread) are
//! consulted first, then the shared promise registry (externally-registered
//! promises). On timeout the receiver is put back so a later await can
//! still consume it — chunked awaits from Dart rely on this.

use crate::vm::VM;
use hudhudscript_bytecode::gc_detach;
use hudhudscript_bytecode::Value16;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

impl VM {
    /// Block until the promise registered under `id` settles.
    ///
    /// Mirrors the interpreter's Await transport selection: detached
    /// receivers first, then the shared promise registry. On timeout the
    /// receiver is re-registered so the id remains awaitable.
    pub fn await_promise_owned(
        &mut self,
        id: &str,
        timeout: Option<Duration>,
    ) -> Result<Value16, String> {
        if let Some(rx) = self.detached_promises.remove(id) {
            let received = match timeout {
                Some(limit) => match rx.recv_timeout(limit) {
                    Ok(inner) => Ok(inner),
                    Err(RecvTimeoutError::Timeout) => {
                        self.detached_promises.insert(id.to_string(), rx);
                        return Err("promise await timed out".to_string());
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        Err("promise sender dropped".to_string())
                    }
                },
                None => rx.recv().map_err(|_| "promise sender dropped".to_string()),
            };
            return match received {
                Ok(Ok(tree)) => Ok(gc_detach::attach(&tree)),
                Ok(Err(msg)) => Err(msg),
                Err(msg) => Err(msg),
            };
        }

        if let Some(result) = self.promise_registry.take_cached_result(id) {
            return result;
        }
        if let Some(rx) = self.promise_registry.take_receiver(id) {
            let received = match timeout {
                Some(limit) => match rx.recv_timeout(limit) {
                    Ok(inner) => Ok(inner),
                    Err(RecvTimeoutError::Timeout) => {
                        self.promise_registry
                            .register_external_with_id(id.to_string(), rx);
                        return Err("promise await timed out".to_string());
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        Err("promise sender dropped".to_string())
                    }
                },
                None => rx.recv().map_err(|_| "promise sender dropped".to_string()),
            };
            return match received {
                Ok(Ok(val)) => Ok(val),
                Ok(Err(msg)) => Err(msg),
                Err(msg) => Err(msg),
            };
        }
        Err(format!("unregistered promise id: {id}"))
    }

    /// Non-blocking poll: `None` while the promise is still pending,
    /// `Some(result)` once settled. Un-settled receivers are put back so a
    /// later `await_promise_owned` still works.
    pub fn try_take_promise(&mut self, id: &str) -> Option<Result<Value16, String>> {
        if let Some(rx) = self.detached_promises.remove(id) {
            return match rx.try_recv() {
                Ok(Ok(tree)) => Some(Ok(gc_detach::attach(&tree))),
                Ok(Err(msg)) => Some(Err(msg)),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.detached_promises.insert(id.to_string(), rx);
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(Err("promise sender dropped".to_string()))
                }
            };
        }

        if let Some(result) = self.promise_registry.take_cached_result(id) {
            return Some(result);
        }
        if let Some(rx) = self.promise_registry.take_receiver(id) {
            return match rx.try_recv() {
                Ok(inner) => Some(inner),
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.promise_registry
                        .register_external_with_id(id.to_string(), rx);
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    Some(Err("promise sender dropped".to_string()))
                }
            };
        }
        None
    }
}
