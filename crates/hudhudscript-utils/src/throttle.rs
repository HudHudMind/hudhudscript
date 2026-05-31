//! Throttle utility (Issue #673)
//!
//! Limits how often a function can execute. Unlike debounce (which delays until
//! calm), throttle allows the first call and then suppresses subsequent calls
//! within the cooldown window.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A throttle that allows at most one execution per `interval`.
///
/// The first call always executes. Subsequent calls within `interval`
/// are suppressed and return `false`.
pub struct Throttle {
    inner: Mutex<ThrottleInner>,
}

struct ThrottleInner {
    interval: Duration,
    last_allowed: Option<Instant>,
}

impl Throttle {
    /// Create a new throttle with the given minimum interval between executions.
    pub fn new(interval: Duration) -> Self {
        Self {
            inner: Mutex::new(ThrottleInner {
                interval,
                last_allowed: None,
            }),
        }
    }

    /// Check if an execution is allowed right now.
    /// Returns `true` if allowed (and records the timestamp), `false` if throttled.
    pub fn try_run(&self) -> bool {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        match inner.last_allowed {
            None => {
                inner.last_allowed = Some(now);
                true
            }
            Some(last) if now.duration_since(last) >= inner.interval => {
                inner.last_allowed = Some(now);
                true
            }
            _ => false,
        }
    }

    /// Wait until the throttle allows execution, then mark it.
    pub async fn run(&self) {
        loop {
            if self.try_run() {
                return;
            }
            let wait = {
                let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                match inner.last_allowed {
                    Some(last) => {
                        let elapsed = Instant::now().duration_since(last);
                        if elapsed < inner.interval {
                            inner.interval - elapsed
                        } else {
                            Duration::ZERO
                        }
                    }
                    None => Duration::ZERO,
                }
            };
            if wait > Duration::ZERO {
                tokio::time::sleep(wait).await;
            }
        }
    }

    /// Reset the throttle, allowing the next call immediately.
    pub fn reset(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.last_allowed = None;
    }
}
