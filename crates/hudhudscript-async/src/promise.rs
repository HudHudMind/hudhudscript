//! Promise implementation.
//!
//! Audit v3 Finding 11.1 (PERF-27): the internal state is now
//! `Arc<OnceLock<Result<T, String>>>`.  A promise transitions from
//! pending → resolved/rejected exactly once; after that, reads are
//! lock-free atomic loads (vs the previous parking_lot::Mutex hold).
//! The external-facing `PromiseState<T>` enum is reconstructed on
//! demand from the `OnceLock`, so the public API is unchanged and
//! `AsyncPending` is handled by the bytecode layer (it's never a
//! state `Promise<T>` takes internally).

use parking_lot::Mutex;
use std::fmt;
use std::sync::{Arc, OnceLock};
use tokio::sync::oneshot;
use uuid::Uuid;

// Re-export the canonical PromiseState from hudhudscript-errors.
pub use hudhudscript_errors::shared_types::PromiseState;

/// Promise ID
pub type PromiseId = Uuid;

/// Promise error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromiseError {
    AlreadyResolved,
    AlreadyRejected,
    Rejected(String),
    ReceiverDropped,
}

impl std::fmt::Display for PromiseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            PromiseError::AlreadyResolved => write!(f, "Promise already resolved"),
            PromiseError::AlreadyRejected => write!(f, "Promise already rejected"),
            PromiseError::Rejected(s) => write!(f, "Promise was rejected: {}", s),
            PromiseError::ReceiverDropped => write!(f, "Promise receiver dropped"),
        }
    }
}

impl std::error::Error for PromiseError {}

/// Internal shared state — set exactly once.  `Ok(v)` = resolved,
/// `Err(e)` = rejected.  Absence of a value means "still pending".
type FinalState<T> = std::result::Result<T, String>;

/// Translate the monotonic `OnceLock` view into the public
/// `PromiseState<T>` enum.
#[inline]
fn snapshot<T: Clone>(cell: &OnceLock<FinalState<T>>) -> PromiseState<T> {
    match cell.get() {
        None => PromiseState::Pending,
        Some(Ok(v)) => PromiseState::Resolved(Box::new(v.clone())),
        Some(Err(e)) => PromiseState::Rejected(e.clone()),
    }
}

/// Promise - represents an async operation
#[derive(Clone)]
pub struct Promise<T> {
    /// Unique promise ID
    id: PromiseId,

    /// Final state (set once by resolver, lock-free reads thereafter).
    state: Arc<OnceLock<FinalState<T>>>,

    /// Receiver for awaiting result (wrapped in Arc<Mutex> for cloning)
    #[allow(clippy::type_complexity)]
    receiver: Arc<Mutex<Option<oneshot::Receiver<Result<T, String>>>>>,
}

impl<T: Clone> Promise<T> {
    /// Create a new pending promise
    pub fn new() -> (Self, PromiseResolver<T>) {
        let id = Uuid::new_v4();
        let state: Arc<OnceLock<FinalState<T>>> = Arc::new(OnceLock::new());
        let (sender, receiver) = oneshot::channel();

        let promise = Promise {
            id,
            state: state.clone(),
            receiver: Arc::new(Mutex::new(Some(receiver))),
        };

        let resolver = PromiseResolver {
            id,
            state,
            sender: Some(sender),
        };

        (promise, resolver)
    }

    /// Create an already resolved promise
    pub fn resolved(value: T) -> Self {
        let id = Uuid::new_v4();
        let state: Arc<OnceLock<FinalState<T>>> = Arc::new(OnceLock::new());
        // `set` cannot fail on a fresh cell.
        let _ = state.set(Ok(value));

        Promise {
            id,
            state,
            receiver: Arc::new(Mutex::new(None)),
        }
    }

    /// Create an already rejected promise
    pub fn rejected(error: String) -> Self {
        let id = Uuid::new_v4();
        let state: Arc<OnceLock<FinalState<T>>> = Arc::new(OnceLock::new());
        let _ = state.set(Err(error));

        Promise {
            id,
            state,
            receiver: Arc::new(Mutex::new(None)),
        }
    }

    /// Get promise ID
    pub fn id(&self) -> PromiseId {
        self.id
    }

    /// Get current state (non-blocking).  Audit v3 F11.1 / PERF-27:
    /// lock-free atomic load on the shared `OnceLock`.
    pub async fn state(&self) -> PromiseState<T> {
        snapshot(&self.state)
    }

    /// Check if promise is pending
    pub async fn is_pending(&self) -> bool {
        self.state.get().is_none()
    }

    /// Check if promise is resolved
    pub async fn is_resolved(&self) -> bool {
        matches!(self.state.get(), Some(Ok(_)))
    }

    /// Check if promise is rejected
    pub async fn is_rejected(&self) -> bool {
        matches!(self.state.get(), Some(Err(_)))
    }

    /// Await the promise result (blocking until resolved/rejected)
    pub async fn await_result(self) -> Result<T, PromiseError> {
        // Fast path: already final — no lock acquisition at all.
        match self.state.get() {
            Some(Ok(value)) => return Ok(value.clone()),
            Some(Err(error)) => return Err(PromiseError::Rejected(error.clone())),
            None => {}
        }

        // Wait for resolution — take the receiver out of the sync Mutex
        // (brief critical section) before awaiting on it.
        let receiver_opt = self.receiver.lock().take();
        if let Some(receiver) = receiver_opt {
            match receiver.await {
                Ok(Ok(value)) => Ok(value),
                Ok(Err(error)) => Err(PromiseError::Rejected(error)),
                Err(_) => Err(PromiseError::ReceiverDropped),
            }
        } else {
            Err(PromiseError::ReceiverDropped)
        }
    }
}

impl<T> fmt::Debug for Promise<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Promise").field("id", &self.id).finish()
    }
}

/// Promise resolver - used to resolve or reject a promise
pub struct PromiseResolver<T> {
    id: PromiseId,
    state: Arc<OnceLock<FinalState<T>>>,
    sender: Option<oneshot::Sender<Result<T, String>>>,
}

impl<T: Clone> PromiseResolver<T> {
    /// Get promise ID
    pub fn id(&self) -> PromiseId {
        self.id
    }

    /// Resolve the promise with a value
    pub async fn resolve(mut self, value: T) -> Result<(), PromiseError> {
        // `set` returns `Err(rejected_value)` if the cell was already set.
        // Map that to the right PromiseError depending on which terminal
        // state the cell is in.
        match self.state.set(Ok(value.clone())) {
            Ok(()) => {
                if let Some(sender) = self.sender.take() {
                    let _ = sender.send(Ok(value));
                }
                Ok(())
            }
            Err(_) => match self.state.get() {
                Some(Ok(_)) => Err(PromiseError::AlreadyResolved),
                Some(Err(_)) => Err(PromiseError::AlreadyRejected),
                None => unreachable!("set returned Err but cell empty"),
            },
        }
    }

    /// Reject the promise with an error
    pub async fn reject(mut self, error: String) -> Result<(), PromiseError> {
        match self.state.set(Err(error.clone())) {
            Ok(()) => {
                if let Some(sender) = self.sender.take() {
                    let _ = sender.send(Err(error));
                }
                Ok(())
            }
            Err(_) => match self.state.get() {
                Some(Ok(_)) => Err(PromiseError::AlreadyResolved),
                Some(Err(_)) => Err(PromiseError::AlreadyRejected),
                None => unreachable!("set returned Err but cell empty"),
            },
        }
    }
}

impl<T> fmt::Debug for PromiseResolver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PromiseResolver")
            .field("id", &self.id)
            .finish()
    }
}

// Tests moved to hudhud-script-tests/tests/async_test_inline.rs

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl PromiseError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            PromiseError::AlreadyRejected => hudhudscript_errors::ErrorCode::PromiseAlreadyRejected,
            PromiseError::AlreadyResolved => hudhudscript_errors::ErrorCode::PromiseAlreadyResolved,
            PromiseError::ReceiverDropped => hudhudscript_errors::ErrorCode::PromiseReceiverDropped,
            PromiseError::Rejected(..) => hudhudscript_errors::ErrorCode::PromiseRejected,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<PromiseError> for hudhudscript_errors::Error {
    fn from(e: PromiseError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
