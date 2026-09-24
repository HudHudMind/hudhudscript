//! VM Promise combinators across every supported resolver transport.
//!
//! Async HudHudScript functions return heap-independent `DetachedGraph`
//! payloads, while embedders may register direct `Value16` receivers.  The
//! combinators must wait on both transports concurrently and attach detached
//! values only on the owning VM thread.

use crate::vm::VM;
use hudhudscript_bytecode::gc_detach::{self, DetachedGraph};
use hudhudscript_bytecode::{PromiseState16, Value16};
use std::sync::mpsc::{self, Receiver, Sender};

enum PendingSource {
    Detached(Receiver<Result<DetachedGraph, String>>),
    Registered(Receiver<Result<Value16, String>>),
}

enum PendingValue {
    Detached(DetachedGraph),
    Registered(Value16),
}

enum AsyncSource {
    Settled(Result<Value16, String>),
    Pending(PendingSource),
}

type AggregateMessage = (usize, Result<PendingValue, String>);

impl PendingSource {
    fn forward(self, index: usize, id: String, sender: Sender<AggregateMessage>) {
        std::thread::spawn(move || {
            let result = match self {
                Self::Detached(receiver) => match receiver.recv() {
                    Ok(Ok(graph)) => Ok(PendingValue::Detached(graph)),
                    Ok(Err(message)) => Err(message),
                    Err(_) => Err(sender_dropped(&id)),
                },
                Self::Registered(receiver) => match receiver.recv() {
                    Ok(Ok(value)) => Ok(PendingValue::Registered(value)),
                    Ok(Err(message)) => Err(message),
                    Err(_) => Err(sender_dropped(&id)),
                },
            };
            let _ = sender.send((index, result));
        });
    }
}

fn sender_dropped(id: &str) -> String {
    format!("Promise {} sender was dropped before resolution", id)
}

fn into_vm_value(value: PendingValue) -> Value16 {
    match value {
        PendingValue::Detached(graph) => gc_detach::attach(&graph),
        PendingValue::Registered(value) => value,
    }
}

impl VM {
    fn take_async_source(&mut self, id: &str) -> Result<AsyncSource, String> {
        if let Some(receiver) = self.detached_promises.remove(id) {
            return Ok(AsyncSource::Pending(PendingSource::Detached(receiver)));
        }
        if let Some(result) = self.promise_registry.take_cached_result(id) {
            return Ok(AsyncSource::Settled(result));
        }
        if let Some(receiver) = self.promise_registry.take_receiver(id) {
            return Ok(AsyncSource::Pending(PendingSource::Registered(receiver)));
        }
        Err(format!("Promise {} has no registered resolver", id))
    }

    /// Resolve all inputs concurrently and preserve their input order.
    pub(crate) fn resolve_promise_all(
        &mut self,
        promises: Vec<Value16>,
    ) -> Result<Vec<Value16>, String> {
        let mut slots: Vec<Option<Value16>> = (0..promises.len()).map(|_| None).collect();
        let mut pending = Vec::new();

        for (index, promise) in promises.into_iter().enumerate() {
            match promise.as_promise_state() {
                Some(PromiseState16::Resolved(value)) => slots[index] = Some(**value),
                Some(PromiseState16::Rejected(message)) => return Err(message.clone()),
                Some(PromiseState16::Pending) => {
                    return Err("Cannot resolve a bare Pending promise".to_string())
                }
                Some(PromiseState16::AsyncPending(id)) => match self.take_async_source(id)? {
                    AsyncSource::Settled(Ok(value)) => slots[index] = Some(value),
                    AsyncSource::Settled(Err(message)) => return Err(message),
                    AsyncSource::Pending(source) => {
                        pending.push((index, id.clone(), source));
                    }
                },
                None => slots[index] = Some(promise),
            }
        }

        let (sender, receiver) = mpsc::channel::<AggregateMessage>();
        let mut remaining = pending.len();
        for (index, id, source) in pending {
            source.forward(index, id, sender.clone());
        }
        drop(sender);

        while remaining > 0 {
            match receiver.recv() {
                Ok((index, Ok(value))) => {
                    slots[index] = Some(into_vm_value(value));
                    remaining -= 1;
                }
                Ok((_index, Err(message))) => return Err(message),
                Err(_) => {
                    return Err("Promise.all aggregate channel closed before settlement".to_string())
                }
            }
        }

        Ok(slots
            .into_iter()
            .map(|slot| slot.unwrap_or(Value16::null()))
            .collect())
    }

    /// Resolve with the first input to settle across detached and registered
    /// Promise transports.
    pub(crate) fn resolve_promise_race(
        &mut self,
        promises: Vec<Value16>,
    ) -> Result<Value16, String> {
        if promises.is_empty() {
            return Err("Promise.race() on empty array".to_string());
        }

        let mut async_ids = Vec::with_capacity(promises.len());
        for promise in &promises {
            match promise.as_promise_state() {
                Some(PromiseState16::Resolved(value)) => return Ok(**value),
                Some(PromiseState16::Rejected(message)) => return Err(message.clone()),
                Some(PromiseState16::Pending) => {
                    return Err("Cannot resolve a bare Pending promise".to_string())
                }
                Some(PromiseState16::AsyncPending(id)) => async_ids.push(id.clone()),
                None => return Ok(*promise),
            }
        }

        // A cached registry settlement is already complete and therefore wins
        // before live receivers. Detached receivers remain owned by the VM.
        for id in &async_ids {
            if !self.detached_promises.contains_key(id) {
                if let Some(result) = self.promise_registry.take_cached_result(id) {
                    return result;
                }
            }
        }

        let (sender, receiver) = mpsc::channel::<AggregateMessage>();
        for (index, id) in async_ids.into_iter().enumerate() {
            match self.take_async_source(&id)? {
                AsyncSource::Settled(result) => return result,
                AsyncSource::Pending(source) => {
                    source.forward(index, id, sender.clone());
                }
            }
        }
        drop(sender);

        match receiver.recv() {
            Ok((_index, Ok(value))) => Ok(into_vm_value(value)),
            Ok((_index, Err(message))) => Err(message),
            Err(_) => Err("Promise.race aggregate channel closed before settlement".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hudhudscript_bytecode::PromiseState16;

    fn async_pending(id: String) -> Value16 {
        Value16::promise(PromiseState16::AsyncPending(id))
    }

    #[test]
    fn all_combines_detached_and_registered_receivers() {
        let mut vm = VM::new();
        let detached = vm.spawn_async_task(|| Ok(Value16::string("detached".to_string())));
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Ok(Value16::string("registered".to_string())))
            .expect("registered receiver must remain open");
        let registered = async_pending(vm.register_promise_owned(receiver));

        let values = vm
            .resolve_promise_all(vec![registered, detached])
            .expect("both resolver transports must settle");

        assert_eq!(values[0].as_string(), Some("registered".to_string()));
        assert_eq!(values[1].as_string(), Some("detached".to_string()));
    }

    #[test]
    fn race_combines_detached_and_registered_receivers() {
        let mut vm = VM::new();
        let detached = vm.spawn_async_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(80));
            Ok(Value16::string("detached".to_string()))
        });
        let (sender, receiver) = mpsc::channel();
        sender
            .send(Ok(Value16::string("registered".to_string())))
            .expect("registered receiver must remain open");
        let registered = async_pending(vm.register_promise_owned(receiver));

        let winner = vm
            .resolve_promise_race(vec![detached, registered])
            .expect("the ready registered receiver must win");

        assert_eq!(winner.as_string(), Some("registered".to_string()));
    }

    #[test]
    fn all_reports_an_unknown_resolver_id() {
        let mut vm = VM::new();
        let missing = async_pending("promise-missing".to_string());

        let error = vm
            .resolve_promise_all(vec![missing])
            .expect_err("an unknown resolver id must fail");

        assert_eq!(
            error,
            "Promise promise-missing has no registered resolver".to_string()
        );
    }
}
