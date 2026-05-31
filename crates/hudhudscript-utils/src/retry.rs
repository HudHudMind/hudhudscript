//! Retry with exponential backoff and jitter (Issue #673)

use std::future::Future;
use std::time::Duration;
use thiserror::Error;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries, just the initial call).
    pub max_retries: u32,
    /// Base delay between retries (before exponential growth).
    pub base_delay: Duration,
    /// Maximum delay cap (backoff will not exceed this).
    pub max_delay: Duration,
    /// Exponential backoff multiplier (typically 2.0).
    pub multiplier: f64,
    /// Whether to add random jitter (±50% of computed delay).
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new config with the given max retries and base delay.
    pub fn new(max_retries: u32, base_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            ..Default::default()
        }
    }

    /// Compute the delay for a given attempt number (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.base_delay.as_millis() as f64;
        let delay_ms = base_ms * self.multiplier.powi(attempt as i32);
        let capped_ms = delay_ms.min(self.max_delay.as_millis() as f64);

        let final_ms = if self.jitter {
            // Add ±50% jitter
            let jitter_range = capped_ms * 0.5;
            let jitter = (simple_random() * 2.0 - 1.0) * jitter_range;
            (capped_ms + jitter).max(0.0)
        } else {
            capped_ms
        };

        Duration::from_millis(final_ms as u64)
    }
}

/// Error returned when all retry attempts are exhausted.
#[derive(Debug, Error)]
#[error("all {attempts} retry attempts exhausted: {last_error}")]
pub struct RetryError<E: std::fmt::Display + std::fmt::Debug> {
    pub attempts: u32,
    pub last_error: E,
}

/// Retry an async operation with exponential backoff.
///
/// The `operation` closure is called up to `config.max_retries + 1` times.
/// On each failure, the function waits with exponential backoff before retrying.
///
/// # Example
/// ```ignore
/// use hudhudscript_utils::retry::{retry, RetryConfig};
/// use std::time::Duration;
///
/// let config = RetryConfig::new(3, Duration::from_millis(100));
/// let result = retry(&config, || async {
///     // your fallible async operation
///     Ok::<_, String>("success")
/// }).await;
/// ```
pub async fn retry<F, Fut, T, E>(config: &RetryConfig, mut operation: F) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + std::fmt::Debug,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_retries {
                    let delay = config.delay_for_attempt(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(RetryError {
        attempts: config.max_retries + 1,
        last_error: last_error.unwrap(),
    })
}

/// Retry a synchronous operation with exponential backoff.
pub fn retry_sync<F, T, E>(config: &RetryConfig, mut operation: F) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display + std::fmt::Debug,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation() {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_retries {
                    let delay = config.delay_for_attempt(attempt);
                    std::thread::sleep(delay);
                }
            }
        }
    }

    Err(RetryError {
        attempts: config.max_retries + 1,
        last_error: last_error.unwrap(),
    })
}

/// Simple pseudo-random number in [0.0, 1.0) using thread-local state.
/// Not cryptographic — suitable only for jitter.
fn simple_random() -> f64 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = const { Cell::new(0x12345678_9ABCDEF0) };
    }
    STATE.with(|s| {
        // Mix in current time on first call for uniqueness
        let mut x = s.get();
        if x == 0x12345678_9ABCDEF0 {
            x ^= std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
        }
        // xorshift64
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x as f64) / (u64::MAX as f64)
    })
}
