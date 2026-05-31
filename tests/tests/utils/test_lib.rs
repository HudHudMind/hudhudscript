//! Inline tests extracted from hudhudscript-utils crates
//! Sources: debounce.rs, rate_limiter.rs, retry.rs, throttle.rs

use hudhudscript_utils::{
    retry, retry_sync, Debouncer, RateLimiter, RetryConfig, RetryError, Throttle,
};
use std::sync::Arc;
use std::time::Duration;

// ── debounce.rs tests ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_debounce_fires_after_delay() {
    let debouncer = Debouncer::new(Duration::from_millis(50));
    debouncer.trigger();

    let gen = tokio::time::timeout(Duration::from_millis(200), debouncer.wait())
        .await
        .expect("should settle within timeout");
    assert_eq!(gen, 1);
}

#[tokio::test]
async fn test_debounce_resets_on_retrigger() {
    let debouncer = Debouncer::new(Duration::from_millis(100));
    debouncer.trigger(); // gen 1

    tokio::time::sleep(Duration::from_millis(50)).await;
    debouncer.trigger(); // gen 2 — resets the timer

    let gen = tokio::time::timeout(Duration::from_millis(300), debouncer.wait())
        .await
        .expect("should settle within timeout");
    assert_eq!(gen, 2);
}

#[test]
fn test_debounce_trigger_increments_generation() {
    let debouncer = Debouncer::new(Duration::from_millis(100));
    assert_eq!(debouncer.trigger(), 1);
    assert_eq!(debouncer.trigger(), 2);
    assert_eq!(debouncer.trigger(), 3);
}

#[tokio::test]
async fn test_spawn_once_executes_callback_after_settle() {
    use std::sync::atomic::{AtomicU64, Ordering};

    let debouncer = Arc::new(Debouncer::new(Duration::from_millis(50)));
    let counter = Arc::new(AtomicU64::new(0));
    let counter_clone = counter.clone();

    debouncer.trigger();
    let handle = debouncer.spawn_once(move || async move {
        counter_clone.store(42, Ordering::SeqCst);
    });

    tokio::time::timeout(Duration::from_millis(300), handle)
        .await
        .expect("should complete within timeout")
        .expect("task should not panic");

    assert_eq!(counter.load(Ordering::SeqCst), 42);
}

// ── rate_limiter.rs tests ─────────────────────────────────────────────────────

#[test]
fn test_rate_limiter_basic() {
    let limiter = RateLimiter::per_second(5);
    // Should allow up to 5 immediately
    for _ in 0..5 {
        assert!(limiter.try_acquire());
    }
    // 6th should be denied
    assert!(!limiter.try_acquire());
}

#[test]
fn test_rate_limiter_available() {
    let limiter = RateLimiter::per_second(10);
    assert_eq!(limiter.available(), 10);
    limiter.try_acquire();
    assert_eq!(limiter.available(), 9);
}

#[test]
fn test_rate_limiter_refill() {
    let limiter = RateLimiter::new(2, 2.0, Duration::from_millis(50));
    // Drain all tokens
    assert!(limiter.try_acquire());
    assert!(limiter.try_acquire());
    assert!(!limiter.try_acquire());

    // Wait for refill
    std::thread::sleep(Duration::from_millis(60));
    assert!(limiter.try_acquire());
}

#[tokio::test]
async fn test_rate_limiter_acquire_async() {
    let limiter = RateLimiter::new(1, 100.0, Duration::from_millis(50));
    limiter.try_acquire(); // drain
                           // acquire should eventually succeed after refill
    tokio::time::timeout(Duration::from_millis(200), limiter.acquire())
        .await
        .expect("acquire should succeed within timeout");
}

// ── retry.rs tests ────────────────────────────────────────────────────────────

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert!(config.jitter);
}

#[test]
fn test_delay_for_attempt_no_jitter() {
    let config = RetryConfig {
        max_retries: 5,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
        jitter: false,
    };
    assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
    assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
    assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
    assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
}

#[test]
fn test_delay_capped_at_max() {
    let config = RetryConfig {
        max_retries: 10,
        base_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(5),
        multiplier: 10.0,
        jitter: false,
    };
    // 1 * 10^2 = 100s, should be capped to 5s
    assert_eq!(config.delay_for_attempt(2), Duration::from_secs(5));
}

#[test]
fn test_delay_with_jitter_is_bounded() {
    let config = RetryConfig {
        max_retries: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
        jitter: true,
    };
    for _ in 0..100 {
        let delay = config.delay_for_attempt(0);
        // 100ms ± 50% => [50ms, 150ms]
        assert!(delay >= Duration::from_millis(50));
        assert!(delay <= Duration::from_millis(150));
    }
}

#[tokio::test]
async fn test_retry_succeeds_first_try() {
    let config = RetryConfig::new(3, Duration::from_millis(1));
    let result: Result<&str, RetryError<&str>> = retry(&config, || async { Ok("ok") }).await;
    assert_eq!(result.unwrap(), "ok");
}

#[tokio::test]
async fn test_retry_succeeds_after_failures() {
    let config = RetryConfig {
        max_retries: 3,
        base_delay: Duration::from_millis(1),
        jitter: false,
        ..Default::default()
    };
    let attempt = std::sync::atomic::AtomicU32::new(0);
    let result: Result<&str, RetryError<String>> = retry(&config, || {
        let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        async move {
            if n < 2 {
                Err(format!("fail #{}", n))
            } else {
                Ok("recovered")
            }
        }
    })
    .await;
    assert_eq!(result.unwrap(), "recovered");
}

#[tokio::test]
async fn test_retry_exhausted() {
    let config = RetryConfig {
        max_retries: 2,
        base_delay: Duration::from_millis(1),
        jitter: false,
        ..Default::default()
    };
    let result: Result<(), RetryError<&str>> =
        retry(&config, || async { Err("always fails") }).await;
    let err = result.unwrap_err();
    assert_eq!(err.attempts, 3); // initial + 2 retries
    assert_eq!(err.last_error, "always fails");
}

#[test]
fn test_retry_sync_succeeds() {
    let config = RetryConfig {
        max_retries: 2,
        base_delay: Duration::from_millis(1),
        jitter: false,
        ..Default::default()
    };
    let result: Result<&str, RetryError<&str>> = retry_sync(&config, || Ok("sync ok"));
    assert_eq!(result.unwrap(), "sync ok");
}

#[test]
fn test_retry_sync_exhausted() {
    let config = RetryConfig {
        max_retries: 1,
        base_delay: Duration::from_millis(1),
        jitter: false,
        ..Default::default()
    };
    let result: Result<(), RetryError<&str>> = retry_sync(&config, || Err("sync fail"));
    let err = result.unwrap_err();
    assert_eq!(err.attempts, 2);
    assert_eq!(err.last_error, "sync fail");
}

#[test]
fn test_retry_sync_succeeds_after_failures() {
    let config = RetryConfig {
        max_retries: 3,
        base_delay: Duration::from_millis(1),
        jitter: false,
        ..Default::default()
    };
    let mut count = 0u32;
    let result: Result<&str, RetryError<String>> = retry_sync(&config, || {
        count += 1;
        if count < 3 {
            Err(format!("attempt {}", count))
        } else {
            Ok("recovered sync")
        }
    });
    assert_eq!(result.unwrap(), "recovered sync");
}

#[test]
fn test_retry_error_display() {
    let err = RetryError {
        attempts: 4,
        last_error: "connection timeout",
    };
    let msg = format!("{}", err);
    assert_eq!(msg, "all 4 retry attempts exhausted: connection timeout");
}

#[test]
fn test_retry_config_new_preserves_defaults() {
    let config = RetryConfig::new(5, Duration::from_millis(200));
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.base_delay, Duration::from_millis(200));
    assert_eq!(config.max_delay, Duration::from_secs(30));
    assert_eq!(config.multiplier, 2.0);
    assert!(config.jitter);
}

// ── throttle.rs tests ─────────────────────────────────────────────────────────

#[test]
fn test_throttle_allows_first_call() {
    let throttle = Throttle::new(Duration::from_millis(100));
    assert!(throttle.try_run());
}

#[test]
fn test_throttle_blocks_rapid_calls() {
    let throttle = Throttle::new(Duration::from_millis(100));
    assert!(throttle.try_run());
    assert!(!throttle.try_run());
    assert!(!throttle.try_run());
}

#[test]
fn test_throttle_allows_after_interval() {
    let throttle = Throttle::new(Duration::from_millis(50));
    assert!(throttle.try_run());
    std::thread::sleep(Duration::from_millis(60));
    assert!(throttle.try_run());
}

#[test]
fn test_throttle_reset() {
    let throttle = Throttle::new(Duration::from_millis(1000));
    assert!(throttle.try_run());
    assert!(!throttle.try_run());
    throttle.reset();
    assert!(throttle.try_run());
}

#[tokio::test]
async fn test_throttle_async_run() {
    let throttle = Throttle::new(Duration::from_millis(50));
    throttle.try_run(); // first call
                        // async run should wait and succeed
    tokio::time::timeout(Duration::from_millis(200), throttle.run())
        .await
        .expect("should succeed within timeout");
}

#[tokio::test]
async fn test_throttle_async_run_first_call() {
    // When no previous call, run() should succeed immediately
    let throttle = Throttle::new(Duration::from_millis(1000));
    tokio::time::timeout(Duration::from_millis(50), throttle.run())
        .await
        .expect("first async run should succeed immediately");
}
