//! Token-bucket rate limiter (Issue #673)

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A token-bucket rate limiter.
///
/// Allows up to `capacity` operations, refilling at `refill_rate` tokens per
/// `refill_interval`. Thread-safe via internal `Mutex`.
pub struct RateLimiter {
    inner: Mutex<RateLimiterInner>,
}

struct RateLimiterInner {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    refill_interval: Duration,
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// - `capacity`: maximum burst size (tokens).
    /// - `refill_rate`: tokens added per `refill_interval`.
    /// - `refill_interval`: time period for refilling `refill_rate` tokens.
    pub fn new(capacity: u32, refill_rate: f64, refill_interval: Duration) -> Self {
        Self {
            inner: Mutex::new(RateLimiterInner {
                capacity,
                tokens: capacity as f64,
                refill_rate,
                refill_interval,
                last_refill: Instant::now(),
            }),
        }
    }

    /// Create a rate limiter that allows `ops_per_second` operations per second.
    pub fn per_second(ops_per_second: u32) -> Self {
        Self::new(
            ops_per_second,
            ops_per_second as f64,
            Duration::from_secs(1),
        )
    }

    /// Try to acquire a token. Returns `true` if allowed, `false` if rate-limited.
    pub fn try_acquire(&self) -> bool {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.refill();
        if inner.tokens >= 1.0 {
            inner.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Acquire a token, blocking (async) until one is available.
    pub async fn acquire(&self) {
        loop {
            if self.try_acquire() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Returns the number of available tokens (approximate).
    pub fn available(&self) -> u32 {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.refill();
        inner.tokens as u32
    }
}

impl RateLimiterInner {
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        if elapsed >= self.refill_interval {
            let periods = elapsed.as_secs_f64() / self.refill_interval.as_secs_f64();
            self.tokens = (self.tokens + periods * self.refill_rate).min(self.capacity as f64);
            self.last_refill = now;
        }
    }
}
