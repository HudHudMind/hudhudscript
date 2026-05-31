//! Async runtime for executing async functions

use crate::promise::{Promise, PromiseId, PromiseResolver};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Async runtime error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncRuntimeError {
    PromiseNotFound(PromiseId),
    TaskSpawnFailed(String),
    RuntimeError(String),
}

impl std::fmt::Display for AsyncRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            AsyncRuntimeError::PromiseNotFound(id) => write!(f, "Promise not found: {}", id),
            AsyncRuntimeError::TaskSpawnFailed(s) => write!(f, "Task spawn failed: {}", s),
            AsyncRuntimeError::RuntimeError(s) => write!(f, "Runtime error: {}", s),
        }
    }
}

impl std::error::Error for AsyncRuntimeError {}

/// Async runtime - manages async operations and promises
pub struct AsyncRuntime<T: Clone + Send + 'static> {
    /// Active promises
    promises: Arc<RwLock<HashMap<PromiseId, Promise<T>>>>,

    /// Active tasks
    tasks: Arc<RwLock<HashMap<PromiseId, JoinHandle<()>>>>,
}

impl<T: Clone + Send + 'static> AsyncRuntime<T> {
    /// Create new async runtime
    pub fn new() -> Self {
        Self {
            promises: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Spawn an async task that returns a promise
    pub async fn spawn_task<F>(&self, task: F) -> Promise<T>
    where
        F: FnOnce(PromiseResolver<T>) -> tokio::task::JoinHandle<()> + Send + 'static,
    {
        let (promise, resolver) = Promise::new();
        let promise_id = promise.id();

        // Store promise
        self.promises
            .write()
            .await
            .insert(promise_id, promise.clone());

        // Spawn task
        let handle = task(resolver);
        self.tasks.write().await.insert(promise_id, handle);

        promise
    }

    /// Get a promise by ID
    pub async fn get_promise(&self, id: PromiseId) -> Option<Promise<T>> {
        self.promises.read().await.get(&id).cloned()
    }

    /// Remove a completed promise
    pub async fn remove_promise(&self, id: PromiseId) {
        self.promises.write().await.remove(&id);

        // Also remove task if exists
        if let Some(handle) = self.tasks.write().await.remove(&id) {
            handle.abort();
        }
    }

    /// Get number of active promises
    pub async fn active_count(&self) -> usize {
        self.promises.read().await.len()
    }

    /// Cancel all pending tasks
    pub async fn cancel_all(&self) {
        let mut tasks = self.tasks.write().await;
        for (_, handle) in tasks.drain() {
            handle.abort();
        }
        self.promises.write().await.clear();
    }
}

impl<T: Clone + Send + 'static> Default for AsyncRuntime<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Tests moved to hudhud-script-tests/tests/async_test_inline.rs

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl AsyncRuntimeError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            AsyncRuntimeError::PromiseNotFound(..) => {
                hudhudscript_errors::ErrorCode::AsyncRuntimePromiseNotFound
            }
            AsyncRuntimeError::RuntimeError(..) => {
                hudhudscript_errors::ErrorCode::AsyncRuntimeRuntimeError
            }
            AsyncRuntimeError::TaskSpawnFailed(..) => {
                hudhudscript_errors::ErrorCode::AsyncRuntimeTaskSpawnFailed
            }
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

impl From<AsyncRuntimeError> for hudhudscript_errors::Error {
    fn from(e: AsyncRuntimeError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
