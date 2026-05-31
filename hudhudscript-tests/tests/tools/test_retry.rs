use hudhudscript_tools::registry::RegistryError;
use hudhudscript_tools::retry::{RetryPolicy, ToolCallOutcome};
use std::time::Duration;

#[test]
fn test_retry_policy_defaults() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.backoff_ms, 100);
    assert!(policy.fallback_tool.is_none());
    assert!(policy.exponential);
}

#[test]
fn test_retry_policy_no_retry() {
    let policy = RetryPolicy::no_retry();
    assert_eq!(policy.max_retries, 0);
}

#[test]
fn test_retry_policy_fixed() {
    let policy = RetryPolicy::fixed(2, 50);
    assert_eq!(policy.max_retries, 2);
    assert_eq!(policy.backoff_ms, 50);
    assert!(!policy.exponential);
}

#[test]
fn test_retry_policy_with_fallback() {
    let policy = RetryPolicy::default().with_fallback("backup_tool");
    assert_eq!(policy.fallback_tool.as_deref(), Some("backup_tool"));
}

#[test]
fn test_delay_exponential() {
    let policy = RetryPolicy {
        max_retries: 3,
        backoff_ms: 100,
        fallback_tool: None,
        exponential: true,
    };
    assert_eq!(policy.delay_for(0), Duration::from_millis(100)); // 100 * 1
    assert_eq!(policy.delay_for(1), Duration::from_millis(200)); // 100 * 2
    assert_eq!(policy.delay_for(2), Duration::from_millis(400)); // 100 * 4
}

#[test]
fn test_delay_fixed() {
    let policy = RetryPolicy::fixed(3, 50);
    assert_eq!(policy.delay_for(0), Duration::from_millis(50));
    assert_eq!(policy.delay_for(1), Duration::from_millis(50));
}

#[test]
fn test_delay_zero_backoff() {
    let policy = RetryPolicy::no_retry();
    assert_eq!(policy.delay_for(0), Duration::ZERO);
}

#[test]
fn test_delay_exponential_high_attempt_capped() {
    let policy = RetryPolicy {
        max_retries: 20,
        backoff_ms: 100,
        fallback_tool: None,
        exponential: true,
    };
    // Attempt 10 should be capped at 2^10 = 1024 factor
    let delay_10 = policy.delay_for(10);
    assert_eq!(delay_10, Duration::from_millis(100 * 1024));
    // Attempt 15 should also be capped at 2^10 (min(15, 10) = 10)
    let delay_15 = policy.delay_for(15);
    assert_eq!(delay_15, Duration::from_millis(100 * 1024));
}

#[test]
fn test_retry_policy_with_fallback_chained() {
    let policy = RetryPolicy::fixed(1, 10).with_fallback("backup");
    assert_eq!(policy.max_retries, 1);
    assert_eq!(policy.backoff_ms, 10);
    assert_eq!(policy.fallback_tool.as_deref(), Some("backup"));
    assert!(!policy.exponential);
}

#[test]
fn test_retry_policy_default_values() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.backoff_ms, 100);
    assert!(policy.exponential);
    assert!(policy.fallback_tool.is_none());
}

#[test]
fn test_delay_fixed_consistent() {
    let policy = RetryPolicy::fixed(5, 200);
    for attempt in 0..5 {
        assert_eq!(policy.delay_for(attempt), Duration::from_millis(200));
    }
}

// ---- RetryPolicy debug ----

#[test]
fn test_retry_policy_debug() {
    let policy = RetryPolicy::default();
    let debug = format!("{:?}", policy);
    assert!(debug.contains("RetryPolicy"));
    assert!(debug.contains("max_retries"));
}

// ---- RetryPolicy clone ----

#[test]
fn test_retry_policy_clone() {
    let policy = RetryPolicy::fixed(2, 50).with_fallback("backup");
    let cloned = policy.clone();
    assert_eq!(cloned.max_retries, 2);
    assert_eq!(cloned.backoff_ms, 50);
    assert_eq!(cloned.fallback_tool.as_deref(), Some("backup"));
}

// ---- ToolCallOutcome debug ----

#[test]
fn test_tool_call_outcome_debug() {
    let outcome = ToolCallOutcome::Success(serde_json::json!({"ok": true}));
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Success"));

    let outcome = ToolCallOutcome::FallbackSuccess {
        fallback_tool: "backup".to_string(),
        result: serde_json::json!(null),
    };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("FallbackSuccess"));
    assert!(debug.contains("backup"));

    let outcome = ToolCallOutcome::Failed {
        attempts: 3,
        last_error: RegistryError::CallFailed("timeout".to_string()),
    };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Failed"));
    assert!(debug.contains("3"));
}

// ---- delay_for with exponential: doubling pattern ----

#[test]
fn test_delay_exponential_doubling_pattern() {
    let policy = RetryPolicy {
        max_retries: 5,
        backoff_ms: 100,
        fallback_tool: None,
        exponential: true,
    };
    let d0 = policy.delay_for(0);
    let d1 = policy.delay_for(1);
    let d2 = policy.delay_for(2);
    let d3 = policy.delay_for(3);
    assert_eq!(d0, Duration::from_millis(100));
    assert_eq!(d1, Duration::from_millis(200));
    assert_eq!(d2, Duration::from_millis(400));
    assert_eq!(d3, Duration::from_millis(800));
    // Each delay should be double the previous
    assert_eq!(d1, d0 * 2);
    assert_eq!(d2, d1 * 2);
    assert_eq!(d3, d2 * 2);
}

// ---- with_fallback replaces existing fallback ----

#[test]
fn test_with_fallback_replaces() {
    let policy = RetryPolicy::default()
        .with_fallback("first")
        .with_fallback("second");
    assert_eq!(policy.fallback_tool.as_deref(), Some("second"));
}
