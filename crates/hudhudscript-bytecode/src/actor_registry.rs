//! Shared actor registry (Kural 7).
//!
//! The interpreter's `hudhudscript-interpreter::actor` module grew the full
//! supervised actor system first — bounded mailboxes, lifecycle tracking,
//! restart strategies, fire-and-forget sends — and the VM carried a
//! bare-bones `HashMap<String, mpsc::Receiver<Value>>` alongside it. Two
//! implementations of the same concept violated Kural 7 and gave VM users
//! weaker semantics than interpreter users.
//!
//! This module is the single source of truth. Both runtimes now hold an
//! `Arc<SharedActorRegistry<Value>>`; the interpreter exposes it via
//! `hudhudscript_interpreter::actor` type aliases, and the VM wires its
//! `spawn_actor` / `send` / `receive_actor` bytecode handlers directly
//! through it. Identity is a registry-generated `String` id in the form
//! `actor-N`; both runtimes treat it opaquely — no Uuid dep here, no Uuid
//! parse round-trip anywhere in the call path.
//!
//! The generic parameter `V` is the message payload type — both
//! `hudhudscript_interpreter_core::Value` and `hudhudscript_bytecode::Value`
//! satisfy `Clone + Send + 'static`.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};

use parking_lot::Mutex;

use hudhudscript_errors::constants::MAILBOX_CAPACITY;

use crate::shared_value::{runtime_error, SharedResult};

// ── Lifecycle types ─────────────────────────────────────────────────────────

/// Actor lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorState {
    /// The actor is running and processing messages.
    Running,
    /// The actor has been gracefully stopped.
    Stopped,
    /// The actor failed with an error message.
    Failed(String),
}

/// Supervision strategy for actor failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupervisionStrategy {
    /// Restart the actor on failure (up to `max_restarts`).
    Restart { max_restarts: usize },
    /// Mark the actor as failed and stop on failure.
    Stop,
}

/// Default max restart count for the `Restart` strategy.
const DEFAULT_MAX_RESTARTS: usize = 5;

impl Default for SupervisionStrategy {
    fn default() -> Self {
        SupervisionStrategy::Restart {
            max_restarts: DEFAULT_MAX_RESTARTS,
        }
    }
}

/// Metadata tracked per actor for lifecycle management.
#[derive(Debug, Clone)]
pub struct ActorMeta {
    pub state: ActorState,
    pub restart_count: usize,
    pub strategy: SupervisionStrategy,
}

// ── Messages and mailboxes ──────────────────────────────────────────────────

/// A message sent to an actor. The payload is the script-visible value;
/// `reply_to` carries an optional actor id that the handler can target when
/// responding.
#[derive(Debug, Clone)]
pub struct ActorMessage<V: Clone + Send + 'static> {
    pub payload: V,
    pub reply_to: Option<String>,
}

/// The receive-side of an actor's mailbox. Owned by whoever called
/// [`SharedActorRegistry::spawn`] and never stored in the registry.
pub struct ActorMailbox<V: Clone + Send + 'static> {
    rx: Receiver<ActorMessage<V>>,
    pending: Mutex<VecDeque<ActorMessage<V>>>,
}

impl<V: Clone + Send + 'static> ActorMailbox<V> {
    /// Receive the next message, blocking until one arrives.
    pub fn recv(&self) -> SharedResult<ActorMessage<V>> {
        if let Some(msg) = self.pending.lock().pop_front() {
            return Ok(msg);
        }
        self.rx
            .recv()
            .map_err(|_| runtime_error("Actor mailbox disconnected"))
    }

    /// Try to receive a message without blocking.
    pub fn try_recv(&self) -> Option<ActorMessage<V>> {
        if let Some(msg) = self.pending.lock().pop_front() {
            return Some(msg);
        }
        match self.rx.try_recv() {
            Ok(msg) => Some(msg),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Snapshot currently queued messages without consuming them.
    pub fn peek_nonblocking(&self) -> Vec<ActorMessage<V>> {
        let mut pending = self.pending.lock();
        loop {
            match self.rx.try_recv() {
                Ok(msg) => pending.push_back(msg),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        pending.iter().cloned().collect()
    }
}

// ── Actor handles ───────────────────────────────────────────────────────────

/// A cloneable reference (handle) to an actor. Holds the send-side of the
/// mailbox, so any clone can push messages in. The registry also stores an
/// internal copy under the same id for cross-cutting operations like send
/// by id or supervision.
#[derive(Debug, Clone)]
pub struct ActorRef<V: Clone + Send + 'static> {
    pub id: String,
    tx: SyncSender<ActorMessage<V>>,
}

impl<V: Clone + Send + 'static> ActorRef<V> {
    /// Send a fire-and-forget message. Non-blocking: if the mailbox is full
    /// the send fails rather than stalling the caller, matching the
    /// back-pressure semantics documented in Issue #444.
    pub fn send(&self, payload: V) -> SharedResult<()> {
        self.tx
            .try_send(ActorMessage {
                payload,
                reply_to: None,
            })
            .map_err(|e| match e {
                TrySendError::Full(_) => runtime_error(format!(
                    "Actor {} mailbox is full (capacity: {})",
                    self.id, MAILBOX_CAPACITY
                )),
                TrySendError::Disconnected(_) => {
                    runtime_error(format!("Actor {} mailbox is closed", self.id))
                }
            })
    }

    /// Send with a reply-to address the handler can use to target a response.
    pub fn send_with_reply(&self, payload: V, reply_to: String) -> SharedResult<()> {
        self.tx
            .try_send(ActorMessage {
                payload,
                reply_to: Some(reply_to),
            })
            .map_err(|e| match e {
                TrySendError::Full(_) => runtime_error(format!(
                    "Actor {} mailbox is full (capacity: {})",
                    self.id, MAILBOX_CAPACITY
                )),
                TrySendError::Disconnected(_) => {
                    runtime_error(format!("Actor {} mailbox is closed", self.id))
                }
            })
    }
}

// ── Registry ────────────────────────────────────────────────────────────────

/// Monotonic counter used to mint opaque actor ids (`actor-N`). Shared across
/// all `SharedActorRegistry` instances so ids are unique workspace-wide even
/// when both runtimes spawn actors concurrently.
static ACTOR_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Registry of live actors. Holds send-side refs so any caller can push
/// messages by id, plus per-actor lifecycle metadata.
///
/// The receive-side mailbox returned by [`Self::spawn`] is handed to the
/// caller — the registry deliberately does not store mailboxes so that the
/// owner can drive recv / try_recv without contention.
#[derive(Debug, Default)]
pub struct SharedActorRegistry<V: Clone + Send + 'static> {
    refs: Mutex<HashMap<String, ActorRef<V>>>,
    meta: Mutex<HashMap<String, ActorMeta>>,
}

impl<V: Clone + Send + 'static> SharedActorRegistry<V> {
    pub fn new() -> Self {
        Self {
            refs: Mutex::new(HashMap::new()),
            meta: Mutex::new(HashMap::new()),
        }
    }

    /// Allocate a fresh opaque actor id.
    fn alloc_id() -> String {
        let n = ACTOR_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("actor-{}", n)
    }

    /// Spawn a new actor with the default supervision strategy. Returns an
    /// `ActorRef` (stored internally too) and an `ActorMailbox` (the
    /// caller's only way to read messages).
    pub fn spawn(&self) -> (ActorRef<V>, ActorMailbox<V>) {
        self.spawn_with_strategy(SupervisionStrategy::default())
    }

    /// Spawn with an explicit supervision strategy.
    pub fn spawn_with_strategy(
        &self,
        strategy: SupervisionStrategy,
    ) -> (ActorRef<V>, ActorMailbox<V>) {
        let id = Self::alloc_id();
        let (tx, rx) = mpsc::sync_channel::<ActorMessage<V>>(MAILBOX_CAPACITY);

        let actor_ref = ActorRef {
            id: id.clone(),
            tx: tx.clone(),
        };
        let mailbox = ActorMailbox {
            rx,
            pending: Mutex::new(VecDeque::new()),
        };

        // Store a second ref under the id so other code paths can look it
        // up (registry.get) without cloning the caller's handle.
        let lookup_ref = ActorRef { id: id.clone(), tx };
        self.refs.lock().insert(id.clone(), lookup_ref);
        self.meta.lock().insert(
            id,
            ActorMeta {
                state: ActorState::Running,
                restart_count: 0,
                strategy,
            },
        );

        (actor_ref, mailbox)
    }

    /// Look up an actor by id.
    pub fn get(&self, id: &str) -> Option<ActorRef<V>> {
        self.refs.lock().get(id).cloned()
    }

    /// Remove an actor's registry entries. The underlying channel continues
    /// to exist for as long as the `ActorRef` / `ActorMailbox` outlive it —
    /// dropping this entry only hides the actor from subsequent `get`
    /// lookups.
    pub fn deregister(&self, id: &str) {
        self.refs.lock().remove(id);
        self.meta.lock().remove(id);
    }

    /// Gracefully stop an actor by marking its state. Returns an error if
    /// the actor was not registered (or was already deregistered).
    pub fn stop(&self, id: &str) -> SharedResult<()> {
        let mut meta = self.meta.lock();
        if let Some(m) = meta.get_mut(id) {
            m.state = ActorState::Stopped;
            Ok(())
        } else {
            Err(runtime_error(format!("Actor {} not found", id)))
        }
    }

    /// Mark an actor as failed. Returns `true` if the supervision strategy
    /// indicates the caller should restart the actor (the registry handles
    /// the restart_count bookkeeping automatically).
    pub fn mark_failed(&self, id: &str, error: String) -> bool {
        let mut meta = self.meta.lock();
        if let Some(m) = meta.get_mut(id) {
            match &m.strategy {
                SupervisionStrategy::Restart { max_restarts } => {
                    if m.restart_count < *max_restarts {
                        m.restart_count += 1;
                        m.state = ActorState::Running;
                        return true;
                    }
                    m.state = ActorState::Failed(error);
                    false
                }
                SupervisionStrategy::Stop => {
                    m.state = ActorState::Failed(error);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Get the current lifecycle state of an actor.
    pub fn get_state(&self, id: &str) -> Option<ActorState> {
        self.meta.lock().get(id).map(|m| m.state.clone())
    }

    /// Get the restart count (0 if never restarted or unknown id).
    pub fn get_restart_count(&self, id: &str) -> usize {
        self.meta
            .lock()
            .get(id)
            .map(|m| m.restart_count)
            .unwrap_or(0)
    }
}
