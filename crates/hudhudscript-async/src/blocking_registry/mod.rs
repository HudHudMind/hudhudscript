//! Thread-backed Promise registry — the shared primitive used by the VM's
//! `await` handler (Issue #1079 / Kural 7).
//!
//! # Why a second registry?
//!
//! The crate already exposes [`crate::AsyncRuntime`], a tokio-based runtime
//! used by the interpreter for `async function`s, Promise.all, Promise.race,
//! etc. The VM, however, is intentionally tokio-free — it runs async
//! functions on dedicated OS threads (`std::thread::spawn`) and communicates
//! results back via `std::sync::mpsc`. Prior to this module both the VM
//! carried its own ad-hoc `promise_receivers` / `promise_results` HashMaps
//! and the interpreter carried the tokio path: two parallel lanes.
//!
//! `PromiseRegistry<V>` consolidates the VM's side into a single shared
//! primitive that any non-tokio runtime (the VM today, future WASM-targeted
//! frontends tomorrow) can reuse. It is still thread-based — it does **not**
//! pull tokio into the VM — but it lives in the async crate so there is one
//! canonical `spawn → await → resolve` pipeline, one place to fix bugs,
//! and one API surface for callers.
//!
//! # Semantics
//!
//! * [`PromiseRegistry::spawn_task`] runs a closure on a background OS
//!   thread, returns a string `PromiseId`, and wires an internal
//!   `mpsc::channel` so the result can be retrieved later.
//! * [`PromiseRegistry::register_external`] lets callers that spawn their
//!   own thread register the receiver side of their own channel. The
//!   registry will `recv()` it on behalf of the caller when `await_blocking`
//!   is invoked.
//! * [`PromiseRegistry::store_result`] lets async work that completes
//!   synchronously stash its result for a later `await`.
//! * [`PromiseRegistry::await_blocking`] is the one entry point the `Await`
//!   bytecode instruction uses: it checks the pre-resolved cache first,
//!   then the registered receiver, and finally reports a well-formed error
//!   if neither path is wired. It **never** silently returns a fabricated
//!   value — the VM previously had a silent fake that was eliminated.
//!
//! Both VM and any future thread-based runtime share this type verbatim;
//! there is no per-backend copy.

pub mod combinators;
pub mod error;

pub use error::*;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};

/// Opaque string identifier for a tracked promise.
///
/// The format is `"promise-<N>"` where N is a monotonically increasing
/// counter. Callers must treat this as opaque; nothing inspects the
/// contents except the registry itself.
pub type PromiseId = String;

/// Result carried by an in-flight promise: `Ok(value)` on resolution,
/// `Err(message)` on rejection.
pub type PromiseResult<V> = Result<V, String>;

/// Thread-backed promise registry.
///
/// `V` is the payload type — the runtime's `Value` enum. It must be
/// `Send + 'static` for the thread-spawn path; callers that only use
/// `store_result` / `register_external` can ignore those bounds at the
/// type boundary.
pub struct PromiseRegistry<V> {
    /// Monotonic counter for id generation.
    pub(crate) counter: AtomicU64,
    /// Channel receivers awaiting their first `recv()`.
    pub(crate) receivers: HashMap<PromiseId, Receiver<PromiseResult<V>>>,
    /// Pre-resolved results stashed by `store_result` (used when the
    /// task completed before the VM reached the matching `await`).
    pub(crate) cached: HashMap<PromiseId, PromiseResult<V>>,
}

impl<V> PromiseRegistry<V> {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
            receivers: HashMap::new(),
            cached: HashMap::new(),
        }
    }

    /// Generate a fresh opaque promise id.
    pub fn next_id(&self) -> PromiseId {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        format!("promise-{}", n)
    }

    /// Number of receivers that have not yet been drained.
    pub fn pending_count(&self) -> usize {
        self.receivers.len() + self.cached.len()
    }

    /// Register an externally-created receiver under a freshly minted id.
    ///
    /// Used when the caller spawns its own thread (e.g. the VM's
    /// `spawn_async_chunk`) and already owns the sender half of an
    /// `mpsc::channel`. The registry takes ownership of the receiver and
    /// returns the id that the VM places inside
    /// `PromiseState::AsyncPending`.
    pub fn register_external(&mut self, receiver: Receiver<PromiseResult<V>>) -> PromiseId {
        let id = self.next_id();
        self.receivers.insert(id.clone(), receiver);
        id
    }

    /// Register an externally-created receiver under a caller-supplied
    /// id. Used by embedders (and the VM's backwards-compatible
    /// `register_promise(id, rx)` shim) that already hold a specific id
    /// they need the receiver filed under.
    ///
    /// If an entry already exists for `id` it is overwritten — last
    /// writer wins, matching the pre-registry HashMap semantics.
    pub fn register_external_with_id(
        &mut self,
        id: PromiseId,
        receiver: Receiver<PromiseResult<V>>,
    ) {
        self.receivers.insert(id, receiver);
    }

    /// Store an already-computed result under a freshly minted id.
    ///
    /// Useful when async work resolves synchronously before the VM
    /// reaches the corresponding `await`.
    pub fn store_result(&mut self, result: PromiseResult<V>) -> PromiseId {
        let id = self.next_id();
        self.cached.insert(id.clone(), result);
        id
    }

    /// Overwrite (or install) a cached result under an explicit id.
    ///
    /// Used by callers that already own the id (e.g. an external resolver
    /// settling a promise late).
    pub fn store_result_with_id(&mut self, id: PromiseId, result: PromiseResult<V>) {
        self.cached.insert(id, result);
    }

    /// Block on the given promise id, consuming its receiver or cached
    /// entry exactly once.
    ///
    /// Returns the resolved value on success, a `RegistryError` on any
    /// failure. Callers must map the error into their runtime's native
    /// error type.
    pub fn await_blocking(&mut self, id: &str) -> Result<V, RegistryError> {
        if let Some(result) = self.cached.remove(id) {
            return result.map_err(RegistryError::Rejected);
        }
        if let Some(receiver) = self.receivers.remove(id) {
            return match receiver.recv() {
                Ok(Ok(val)) => Ok(val),
                Ok(Err(msg)) => Err(RegistryError::Rejected(msg)),
                Err(_) => Err(RegistryError::SenderDropped(id.to_string())),
            };
        }
        Err(RegistryError::Unregistered(id.to_string()))
    }

    /// True if the given id can be resolved without blocking (either it
    /// has a cached result or a ready receiver).
    pub fn has_entry(&self, id: &str) -> bool {
        self.cached.contains_key(id) || self.receivers.contains_key(id)
    }

    /// Drop all tracked receivers and cached results. Intended for teardown.
    pub fn clear(&mut self) {
        self.receivers.clear();
        self.cached.clear();
    }
}

impl<V: Clone> PromiseRegistry<V> {
    /// Snapshot resolved cached values for conservative external traversals such
    /// as GC root marking.
    pub fn cached_values(&self) -> Vec<V> {
        self.cached
            .values()
            .filter_map(|result| result.as_ref().ok().cloned())
            .collect()
    }
}

impl<V: Send + 'static> PromiseRegistry<V> {
    /// Spawn `task` on a fresh OS thread, returning the id for the
    /// resulting promise.
    ///
    /// The task's return value is wrapped into `Ok(value)`; callers that
    /// want to reject should return an `Err(msg)` from the inner
    /// computation themselves. `await_blocking` on the returned id yields
    /// the task's result.
    pub fn spawn_task<F>(&mut self, task: F) -> PromiseId
    where
        F: FnOnce() -> PromiseResult<V> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = task();
            let _ = tx.send(result);
        });
        self.register_external(rx)
    }
}

impl<V> Default for PromiseRegistry<V> {
    fn default() -> Self {
        Self::new()
    }
}
