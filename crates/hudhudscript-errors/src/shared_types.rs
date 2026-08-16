//! Shared runtime types used by both the interpreter and VM.
//!
//! These types are generic over the `Value` type so that each backend
//! (interpreter-core, bytecode) can instantiate them with its own `Value`.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::mpsc;

// ---------------------------------------------------------------------------
// PromiseState<V>
// ---------------------------------------------------------------------------

/// State of a promise value.
///
/// Generic over `V` so that both the interpreter (`interpreter_core::Value`)
/// and the VM (`bytecode::Value`) can share the same definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromiseState<V> {
    /// Promise is still pending (synchronous).
    Pending,
    /// Promise resolved with a value.
    Resolved(Box<V>),
    /// Promise was rejected with an error message.
    Rejected(String),
    /// Async pending promise tracked by a string identifier.
    ///
    /// In the interpreter this is `Uuid::to_string()`; in the VM it is an
    /// opaque string ID. Using `String` avoids coupling this crate to the
    /// `uuid` dependency.
    AsyncPending(String),
}

// ---------------------------------------------------------------------------
// GeneratorState<V>
// ---------------------------------------------------------------------------

/// State of a lazy generator (Issue #747).
///
/// The generator body runs on a dedicated OS thread and sends each yielded
/// value through a rendezvous (capacity-0) `SyncSender`, which means the
/// producer naturally blocks after each yield until the consumer calls
/// `next()` (i.e. `receiver.recv()`).
///
/// When the body finishes (or returns early), the sender is dropped and
/// `recv()` returns `Err`, which we surface as `None`.
///
/// Generic over `V` so that both the interpreter and VM can share the same
/// definition with their own `Value` types.
pub struct GeneratorState<V> {
    /// Channel receiver for lazy value production.
    receiver: mpsc::Receiver<V>,
    pub pending: std::collections::VecDeque<V>,
    /// Values already consumed (kept for toArray / serialization).
    pub buffered: Vec<V>,
    /// Whether the generator has been exhausted.
    done: bool,
    /// V2-B: VM-side OwnedTree receiver key (None for precomputed).
    pub yield_id: Option<u64>,
}

impl<V: Clone> GeneratorState<V> {
    /// Create a new lazy generator state from a channel receiver.
    pub fn new(receiver: mpsc::Receiver<V>) -> Self {
        Self {
            receiver,
            pending: std::collections::VecDeque::new(),
            buffered: Vec::new(),
            done: false,
            yield_id: None,
        }
    }

    /// Advance the generator by one step, returning the next value or `None`
    /// if the generator is exhausted.
    pub fn advance(&mut self) -> Option<V> {
        if self.done {
            return None;
        }
        if let Some(val) = self.pending.pop_front() {
            self.buffered.push(val.clone());
            return Some(val);
        }
        match self.receiver.recv() {
            Ok(val) => {
                self.buffered.push(val.clone());
                Some(val)
            }
            Err(_) => {
                self.done = true;
                None
            }
        }
    }

    /// Whether the generator is known to be exhausted.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// Drain all remaining values and return them combined with buffered ones.
    pub fn collect_all(&mut self) -> Vec<V> {
        while let Some(val) = self.pending.pop_front() {
            self.buffered.push(val);
        }
        while !self.done {
            match self.receiver.recv() {
                Ok(val) => self.buffered.push(val),
                Err(_) => {
                    self.done = true;
                }
            }
        }
        self.buffered.clone()
    }
}

impl<V> fmt::Debug for GeneratorState<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GeneratorState {{ buffered: {}, done: {} }}",
            self.buffered.len(),
            self.done
        )
    }
}

/// Create a `GeneratorState` from pre-computed values (for backward compat in
/// the VM path and tests that construct generators from a `Vec<V>`).
impl<V: Send + 'static> From<Vec<V>> for GeneratorState<V> {
    fn from(values: Vec<V>) -> Self {
        let (_tx, rx) = mpsc::sync_channel(0);
        Self {
            receiver: rx,
            pending: values.into(),
            buffered: Vec::new(),
            done: false,
            yield_id: None,
        }
    }
}
