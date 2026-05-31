//! Async/Await runtime for HudHudScript
//!
//! This crate provides async runtime infrastructure including:
//! - Promise management
//! - Async function execution
//! - Await expression support
//! - Promise combinators (all, race)

pub mod blocking_registry;
pub mod combinators;
pub mod promise;
pub mod runtime;

pub use blocking_registry::{PromiseRegistry, RegistryError};
pub use combinators::{promise_all, promise_race};
pub use promise::{Promise, PromiseError, PromiseId, PromiseState};
pub use runtime::{AsyncRuntime, AsyncRuntimeError};

// Tests moved to hudhud-script-tests/tests/async_test_inline.rs
