//! RAII (Resource Acquisition Is Initialization) support for agent resources.
//!
//! # Issue #102 — Bjarne Stroustrup Review: Enforce RAII for Agents
//!
//! In C++, RAII guarantees that resources (file handles, connections, memory) are
//! released deterministically when an object goes out of scope via its destructor.
//! HudHudScript agents accumulate resources during their lifetime — open provider
//! connections, file handles, allocated token budgets, event-bus subscriptions —
//! and these must be cleaned up promptly when an agent session ends.
//!
//! ## Design
//!
//! ```text
//! ┌───────────────────────────────────────┐
//! │  AgentSession (owns resources)        │
//! │   ├─ connection handles               │
//! │   ├─ file descriptors                 │
//! │   └─ event subscriptions             │
//! │                                       │
//! │  Drop::drop() triggers               │
//! │   └─ Disposable::dispose()           │
//! │       → close connections            │
//! │       → flush & close files          │
//! │       → deregister subscriptions     │
//! └───────────────────────────────────────┘
//! ```
//!
//! ## Lifecycle
//!
//! 1. Resources are *acquired* inside `AgentSession::new()`.
//! 2. The session is used normally.
//! 3. When the session is dropped (goes out of scope or `dispose()` is called
//!    explicitly), all registered cleanup hooks fire in LIFO order (matching
//!    C++ destructor stack unwinding semantics).

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// A cleanup callback registered with an [`AgentSession`].
///
/// Callbacks fire in reverse registration order (LIFO) when the session is
/// disposed — matching the order in which C++ destructors unwind a call stack.
type CleanupHook = Box<dyn FnOnce() + Send + 'static>;

/// The `Disposable` trait models Stroustrup's RAII destructor contract.
///
/// Any type that holds external resources (network connections, file handles,
/// subscriptions) should implement this trait.  The runtime calls `dispose()`
/// automatically when an `AgentSession` ends.
///
/// Implementors MUST be idempotent: calling `dispose()` multiple times must be
/// safe and must not panic.
pub trait Disposable: Send + Sync {
    /// Release all resources held by this object.
    ///
    /// # Contract
    /// - Must be idempotent (safe to call multiple times).
    /// - Must not panic.
    /// - Should log a warning if called in an already-disposed state.
    fn dispose(&self);

    /// Human-readable name used in log output.
    fn resource_name(&self) -> &str;
}

/// A scoped guard that calls `Disposable::dispose()` when dropped.
///
/// This is analogous to `std::unique_ptr` with a custom deleter in C++.
///
/// # Example
/// ```
/// # use hudhudscript_runtime::raii::{Disposable, ScopedResource};
/// # use std::sync::atomic::{AtomicBool, Ordering};
/// # use std::sync::Arc;
///
/// struct MyConnection { disposed: Arc<AtomicBool> }
/// impl Disposable for MyConnection {
///     fn dispose(&self) { self.disposed.store(true, Ordering::SeqCst); }
///     fn resource_name(&self) -> &str { "MyConnection" }
/// }
///
/// let disposed = Arc::new(AtomicBool::new(false));
/// {
///     let conn = MyConnection { disposed: disposed.clone() };
///     let _guard = ScopedResource::new(conn);
/// } // _guard dropped here → dispose() called automatically
/// assert!(disposed.load(Ordering::SeqCst));
/// ```
pub struct ScopedResource<R: Disposable> {
    resource: Option<R>,
}

impl<R: Disposable> ScopedResource<R> {
    /// Wrap a resource in a scoped guard.
    pub fn new(resource: R) -> Self {
        Self {
            resource: Some(resource),
        }
    }

    /// Explicitly release the resource before the guard is dropped.
    pub fn dispose_now(&mut self) {
        if let Some(r) = self.resource.take() {
            r.dispose();
        }
    }

    /// Access the inner resource without releasing it.
    pub fn get(&self) -> Option<&R> {
        self.resource.as_ref()
    }
}

impl<R: Disposable> Drop for ScopedResource<R> {
    fn drop(&mut self) {
        if let Some(r) = self.resource.take() {
            debug!("RAII: dropping resource '{}'", r.resource_name());
            r.dispose();
        }
    }
}

/// Agent session — the RAII container for all resources acquired on behalf of
/// a single agent during one logical execution lifetime.
///
/// Resources are registered via `register_cleanup` and are released in LIFO
/// order when `dispose()` is called explicitly or when the session is dropped.
///
/// `AgentSession` is intentionally *not* `Clone`: each session owns its
/// resources exclusively, preventing double-free and use-after-free bugs.
pub struct AgentSession {
    agent_id: String,
    disposed: AtomicBool,
    /// Hooks stored behind a Mutex so sessions can be shared across async tasks
    /// before disposal.
    cleanup_hooks: Mutex<Vec<(String, CleanupHook)>>,
    /// Disposable resources (connections, files, subscriptions).
    resources: Mutex<Vec<Arc<dyn Disposable>>>,
}

impl fmt::Debug for AgentSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentSession")
            .field("agent_id", &self.agent_id)
            .field("disposed", &self.disposed.load(Ordering::SeqCst))
            .finish()
    }
}

impl AgentSession {
    /// Create a new session for `agent_id`.
    ///
    /// # RAII contract
    /// The session *acquires* its identity here.  Resources are registered
    /// lazily via [`register_cleanup`] and [`register_resource`].
    pub fn new(agent_id: impl Into<String>) -> Self {
        let id = agent_id.into();
        debug!("RAII: AgentSession created for agent '{}'", id);
        Self {
            agent_id: id,
            disposed: AtomicBool::new(false),
            cleanup_hooks: Mutex::new(Vec::new()),
            resources: Mutex::new(Vec::new()),
        }
    }

    /// Register an ad-hoc cleanup callback (LIFO ordering on disposal).
    ///
    /// Use this for lightweight cleanup that does not warrant a full
    /// [`Disposable`] implementation (e.g., releasing a token from a semaphore).
    pub async fn register_cleanup(
        &self,
        label: impl Into<String>,
        hook: impl FnOnce() + Send + 'static,
    ) {
        if self.disposed.load(Ordering::SeqCst) {
            warn!(
                "RAII: register_cleanup() called on already-disposed session '{}'",
                self.agent_id
            );
            return;
        }
        let mut hooks = self.cleanup_hooks.lock().await;
        hooks.push((label.into(), Box::new(hook)));
    }

    /// Register a [`Disposable`] resource that will be disposed when the
    /// session ends.
    pub async fn register_resource(&self, resource: Arc<dyn Disposable>) {
        if self.disposed.load(Ordering::SeqCst) {
            warn!(
                "RAII: register_resource('{}') called on already-disposed session '{}'",
                resource.resource_name(),
                self.agent_id
            );
            return;
        }
        let mut resources = self.resources.lock().await;
        resources.push(resource);
    }

    /// Dispose the session, releasing all registered resources in LIFO order.
    ///
    /// Safe to call multiple times (idempotent).
    pub async fn dispose(&self) {
        if self
            .disposed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // Already disposed — nothing to do.
            return;
        }

        debug!("RAII: disposing AgentSession for agent '{}'", self.agent_id);

        // Fire cleanup hooks in LIFO order (reverse registration).
        let mut hooks = self.cleanup_hooks.lock().await;
        while let Some((label, hook)) = hooks.pop() {
            debug!(
                "RAII: running cleanup hook '{}' for agent '{}'",
                label, self.agent_id
            );
            hook();
        }

        // Dispose registered resources in LIFO order.
        let mut resources = self.resources.lock().await;
        while let Some(resource) = resources.pop() {
            debug!(
                "RAII: disposing resource '{}' for agent '{}'",
                resource.resource_name(),
                self.agent_id
            );
            resource.dispose();
        }

        debug!(
            "RAII: AgentSession for agent '{}' fully disposed",
            self.agent_id
        );
    }

    /// Returns `true` if this session has been disposed.
    pub fn is_disposed(&self) -> bool {
        self.disposed.load(Ordering::SeqCst)
    }

    /// The agent ID this session belongs to.
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
}

impl Drop for AgentSession {
    /// Synchronous drop — fires a best-effort synchronous cleanup.
    ///
    /// Because Rust's `Drop` is synchronous, async cleanup hooks registered via
    /// `register_cleanup` that need an async runtime should be awaited via
    /// `dispose()` *before* the session is dropped.  The synchronous drop path
    /// handles cleanup hooks that are registered without async dependencies and
    /// logs a warning if async resources were not disposed beforehand.
    fn drop(&mut self) {
        if self
            .disposed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // Already disposed cleanly via dispose() — nothing more to do.
            return;
        }

        warn!(
            "RAII: AgentSession for agent '{}' dropped without explicit dispose() call; \
             attempting synchronous cleanup",
            self.agent_id
        );

        // We can't `.await` here, but we can attempt a try_lock.
        if let Ok(mut hooks) = self.cleanup_hooks.try_lock() {
            while let Some((label, hook)) = hooks.pop() {
                debug!("RAII: sync cleanup hook '{}'", label);
                hook();
            }
        } else {
            warn!(
                "RAII: could not acquire cleanup_hooks lock during synchronous drop for agent '{}'",
                self.agent_id
            );
        }

        if let Ok(mut resources) = self.resources.try_lock() {
            while let Some(resource) = resources.pop() {
                resource.dispose();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in Disposable implementations
// ---------------------------------------------------------------------------

/// A named connection handle with deterministic cleanup via the `Disposable` trait.
///
/// **Status**: This is a generic abstraction layer that lifecycles a logical
/// connection (open → use → dispose) without coupling to a specific transport.
/// To use it with a real connection, supply an `on_close` callback that closes
/// the underlying resource:
///
/// ```rust,ignore
/// let client = reqwest::Client::new();
/// let handle = ConnectionHandle::new("api-client")
///     .with_on_close(move || drop(client.clone()));
/// // ... use handle ...
/// // Drop or call .dispose() to fire the on_close hook.
/// ```
///
/// The handle is intentionally generic — it does not embed a specific
/// connection type so the runtime crate doesn't need to depend on every
/// possible transport (reqwest, sqlx, redis, etc.). Concrete connection
/// types live in the consuming crates which wrap them with this handle.
///
/// (v0.4.47.9 — Issue #814: clarified intent)
pub struct ConnectionHandle {
    name: String,
    disposed: AtomicBool,
    /// Optional synchronous teardown hook.
    on_close: Option<Box<dyn Fn() + Send + Sync + 'static>>,
}

impl fmt::Debug for ConnectionHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionHandle")
            .field("name", &self.name)
            .finish()
    }
}

impl ConnectionHandle {
    /// Create a connection handle with an optional teardown callback.
    pub fn new(name: impl Into<String>, on_close: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            name: name.into(),
            disposed: AtomicBool::new(false),
            on_close: Some(Box::new(on_close)),
        }
    }

    /// Create a connection handle without a teardown callback (logging only).
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            disposed: AtomicBool::new(false),
            on_close: None,
        }
    }
}

impl Disposable for ConnectionHandle {
    fn dispose(&self) {
        if self
            .disposed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return; // idempotent
        }
        debug!("RAII: closing connection '{}'", self.name);
        if let Some(hook) = &self.on_close {
            hook();
        }
    }

    fn resource_name(&self) -> &str {
        &self.name
    }
}
