use crate::error::{err_max_retries_exceeded, err_timeout};
use crate::transaction::Transaction;
use hudhudscript_errors::HudHudResult;

/// Configuration for STM transaction retry behaviour.
#[derive(Debug, Clone)]
pub struct StmConfig {
    /// Maximum number of retries before aborting (default: 1000).
    pub max_retries: usize,
    /// Maximum wall-clock time in milliseconds before aborting (default: 5000).
    pub timeout_ms: u64,
    /// Initial backoff in microseconds (default: 1).
    pub initial_backoff_us: u64,
    /// Maximum backoff in microseconds (default: 1000).
    pub max_backoff_us: u64,
}

impl Default for StmConfig {
    fn default() -> Self {
        Self {
            max_retries: 1000,
            timeout_ms: 5000,
            initial_backoff_us: 1,
            max_backoff_us: 1000,
        }
    }
}

/// Execute `f` atomically with default configuration.
pub fn atomically<V, F, R>(f: F) -> HudHudResult<R>
where
    V: Clone,
    F: FnMut(&mut Transaction<V>) -> HudHudResult<R>,
{
    atomically_with_config(f, StmConfig::default())
}

/// Execute `f` atomically with custom configuration.
///
/// `f` receives a `&mut Transaction<V>` it can use to read/write `TVar<V>`s.
/// If the transaction conflicts, `f` is re-run with a fresh log until it
/// commits, `max_retries` is reached, or the timeout expires.
pub fn atomically_with_config<V, F, R>(mut f: F, config: StmConfig) -> HudHudResult<R>
where
    V: Clone,
    F: FnMut(&mut Transaction<V>) -> HudHudResult<R>,
{
    let start = std::time::Instant::now();
    let mut backoff_us = config.initial_backoff_us;

    for _attempt in 0..config.max_retries {
        let elapsed_ms = start.elapsed().as_millis() as u64;
        if elapsed_ms > config.timeout_ms {
            return Err(err_timeout(config.timeout_ms, elapsed_ms));
        }

        let mut tx: Transaction<V> = Transaction::new();
        let result = f(&mut tx)?;
        if tx.try_commit() {
            return Ok(result);
        }

        if backoff_us < 10 {
            std::thread::yield_now();
        } else {
            std::thread::sleep(std::time::Duration::from_micros(backoff_us));
        }
        backoff_us = (backoff_us * 2).min(config.max_backoff_us);
    }
    Err(err_max_retries_exceeded(config.max_retries))
}
