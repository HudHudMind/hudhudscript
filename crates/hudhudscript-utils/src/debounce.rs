//! Debounce utility (Issue #673)
//!
//! Delays execution until a specified duration has passed since the last call.
//! If called again before the delay elapses, the timer resets.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Notify;

/// A debouncer that delays execution until `delay` has passed since the last trigger.
///
/// Each call to `trigger()` resets the timer. The callback runs only after
/// the timer expires without being reset.
pub struct Debouncer {
    state: Arc<Mutex<DebouncerState>>,
    notify: Arc<Notify>,
    delay: Duration,
}

struct DebouncerState {
    generation: u64,
}

impl Debouncer {
    /// Create a new debouncer with the given delay.
    pub fn new(delay: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(DebouncerState { generation: 0 })),
            notify: Arc::new(Notify::new()),
            delay,
        }
    }

    /// Trigger the debouncer. Resets the delay timer.
    /// Returns the generation number of this trigger.
    pub fn trigger(&self) -> u64 {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.generation += 1;
        let gen = state.generation;
        self.notify.notify_waiters();
        gen
    }

    /// Wait until the debounce delay expires after the last trigger.
    /// Returns the generation that was settled.
    pub async fn wait(&self) -> u64 {
        loop {
            let gen_before = {
                self.state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .generation
            };

            tokio::time::sleep(self.delay).await;

            let gen_after = {
                self.state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .generation
            };

            if gen_before == gen_after {
                return gen_after;
            }
            // Generation changed during sleep — loop and wait again
        }
    }

    /// Spawn a task that runs `callback` after the debounce delay settles.
    /// Only runs once per settled period.
    pub fn spawn_once<F, Fut>(self: &Arc<Self>, callback: F) -> tokio::task::JoinHandle<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let debouncer = Arc::clone(self);
        tokio::spawn(async move {
            debouncer.wait().await;
            callback().await;
        })
    }
}
