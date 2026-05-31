//! Tests for tokenomics::streaming — StreamingTokenCounter, StreamAction, TokenReconciliation

use hudhudscript_tokenomics::streaming::*;

// ---------------------------------------------------------------------------
// StreamingTokenCounter::new
// ---------------------------------------------------------------------------

#[test]
fn test_new_initial_count_zero() {
    let counter = StreamingTokenCounter::new(1000);
    assert_eq!(counter.current_count(), 0);
}

#[test]
fn test_new_not_cancelled() {
    let counter = StreamingTokenCounter::new(1000);
    assert!(!counter.is_cancelled());
}

#[test]
fn test_new_usage_percentage_zero() {
    let counter = StreamingTokenCounter::new(1000);
    assert_eq!(counter.usage_percentage(), 0.0);
}

// ---------------------------------------------------------------------------
// process_chunk — Continue
// ---------------------------------------------------------------------------

#[test]
fn test_process_chunk_basic_counting() {
    let mut counter = StreamingTokenCounter::new(1000);
    assert_eq!(counter.process_chunk("hello world"), StreamAction::Continue);
    // "hello world" = 11 chars, ceil(11/4.0) = 3 tokens
    assert_eq!(counter.current_count(), 3);
}

#[test]
fn test_process_chunk_accumulates() {
    let mut counter = StreamingTokenCounter::new(1000);
    counter.process_chunk("aaaa"); // 4 chars = 1 token
    counter.process_chunk("bbbb"); // 4 chars = 1 token
    assert_eq!(counter.current_count(), 2);
}

#[test]
fn test_process_chunk_empty_string() {
    let mut counter = StreamingTokenCounter::new(1000);
    let result = counter.process_chunk("");
    assert_eq!(result, StreamAction::Continue);
    assert_eq!(counter.current_count(), 0);
}

// ---------------------------------------------------------------------------
// process_chunk — Cancel
// ---------------------------------------------------------------------------

#[test]
fn test_process_chunk_cancel_on_exceed() {
    let mut counter = StreamingTokenCounter::new(10);
    let result = counter.process_chunk(&"a".repeat(100)); // 25 tokens > 10
    assert!(matches!(result, StreamAction::Cancel { .. }));
    assert!(counter.is_cancelled());
}

#[test]
fn test_cancel_fields() {
    let mut counter = StreamingTokenCounter::new(10);
    let result = counter.process_chunk(&"a".repeat(100));
    match result {
        StreamAction::Cancel {
            estimated_tokens,
            budget,
        } => {
            assert_eq!(budget, 10);
            assert!(estimated_tokens >= 10);
        }
        _ => panic!("Expected Cancel"),
    }
}

#[test]
fn test_cancelled_stays_cancelled() {
    let mut counter = StreamingTokenCounter::new(5);
    counter.process_chunk(&"a".repeat(100));
    let result = counter.process_chunk("more data");
    assert!(matches!(result, StreamAction::Cancel { .. }));
    assert!(counter.is_cancelled());
}

#[test]
fn test_zero_budget_cancel_immediately() {
    let mut counter = StreamingTokenCounter::new(0);
    let result = counter.process_chunk("a");
    assert_eq!(
        result,
        StreamAction::Cancel {
            estimated_tokens: 1,
            budget: 0
        }
    );
    assert!(counter.is_cancelled());
}

// ---------------------------------------------------------------------------
// process_chunk — Warning
// ---------------------------------------------------------------------------

#[test]
fn test_process_chunk_warning_at_threshold() {
    let mut counter = StreamingTokenCounter::new(100).with_warning_threshold(0.50);
    // 250 chars = ceil(250/4) = 63 tokens, 63% > 50%
    let result = counter.process_chunk(&"a".repeat(250));
    assert!(matches!(result, StreamAction::Warning { .. }));
}

#[test]
fn test_warning_fields() {
    let mut counter = StreamingTokenCounter::new(100).with_warning_threshold(0.50);
    let result = counter.process_chunk(&"a".repeat(250));
    match result {
        StreamAction::Warning {
            estimated_tokens,
            budget,
        } => {
            assert_eq!(budget, 100);
            assert!(estimated_tokens > 0);
        }
        _ => panic!("Expected Warning"),
    }
}

#[test]
fn test_warning_below_threshold_continues() {
    let mut counter = StreamingTokenCounter::new(100).with_warning_threshold(0.80);
    // 100 chars = 25 tokens, 25% < 80%
    let result = counter.process_chunk(&"a".repeat(100));
    assert_eq!(result, StreamAction::Continue);
}

// ---------------------------------------------------------------------------
// with_warning_threshold
// ---------------------------------------------------------------------------

#[test]
fn test_warning_threshold_default() {
    // Default is 0.80
    let mut counter = StreamingTokenCounter::new(100);
    // 320 chars = 80 tokens = 80% = at threshold
    let result = counter.process_chunk(&"a".repeat(320));
    assert!(matches!(result, StreamAction::Warning { .. }));
}

#[test]
fn test_warning_threshold_clamp_low() {
    // Threshold clamped to 0.0 at minimum
    let mut counter = StreamingTokenCounter::new(100).with_warning_threshold(-1.0);
    let result = counter.process_chunk("a"); // 1 token, 1% > 0%
    assert!(matches!(result, StreamAction::Warning { .. }));
}

#[test]
fn test_warning_threshold_clamp_high() {
    // Threshold clamped to 1.0 at maximum
    let mut counter = StreamingTokenCounter::new(100).with_warning_threshold(2.0);
    // 380 chars = 95 tokens, 95% < 100% threshold
    let result = counter.process_chunk(&"a".repeat(380));
    // Should not warn yet (threshold is 1.0)
    assert_eq!(result, StreamAction::Continue);
}

// ---------------------------------------------------------------------------
// reconcile
// ---------------------------------------------------------------------------

#[test]
fn test_reconcile_exact() {
    let mut counter = StreamingTokenCounter::new(1000);
    counter.process_chunk(&"a".repeat(400)); // 100 tokens
    let recon = counter.reconcile(100);
    assert_eq!(recon.estimated, 100);
    assert_eq!(recon.actual, 100);
    assert_eq!(recon.drift_pct, 0.0);
}

#[test]
fn test_reconcile_overestimate() {
    let mut counter = StreamingTokenCounter::new(1000);
    counter.process_chunk(&"a".repeat(400)); // 100 estimated
    let recon = counter.reconcile(90);
    assert_eq!(recon.estimated, 100);
    assert_eq!(recon.actual, 90);
    let expected_drift = (100.0 - 90.0) / 90.0;
    assert!((recon.drift_pct - expected_drift).abs() < 1e-10);
}

#[test]
fn test_reconcile_underestimate() {
    let mut counter = StreamingTokenCounter::new(1000);
    counter.process_chunk(&"a".repeat(400)); // 100 estimated
    let recon = counter.reconcile(120);
    assert_eq!(recon.estimated, 100);
    assert_eq!(recon.actual, 120);
    let expected_drift = (120.0 - 100.0) / 120.0;
    assert!((recon.drift_pct - expected_drift).abs() < 1e-10);
}

#[test]
fn test_reconcile_zero_server_tokens() {
    let mut counter = StreamingTokenCounter::new(1000);
    counter.process_chunk("hello");
    let recon = counter.reconcile(0);
    assert_eq!(recon.drift_pct, 0.0);
    assert_eq!(recon.actual, 0);
}

#[test]
fn test_reconcile_within_20pct() {
    let mut counter = StreamingTokenCounter::new(1000);
    counter.process_chunk(&"a".repeat(400)); // ~100 estimated
    let recon = counter.reconcile(95);
    assert!(recon.drift_pct < 0.20);
}

// ---------------------------------------------------------------------------
// usage_percentage
// ---------------------------------------------------------------------------

#[test]
fn test_usage_percentage_half() {
    let mut counter = StreamingTokenCounter::new(100);
    counter.process_chunk(&"a".repeat(200)); // 50 tokens
    assert_eq!(counter.current_count(), 50);
    assert!((counter.usage_percentage() - 0.50).abs() < f64::EPSILON);
}

#[test]
fn test_usage_percentage_zero_budget() {
    let counter = StreamingTokenCounter::new(0);
    assert_eq!(counter.usage_percentage(), 0.0);
}

// ---------------------------------------------------------------------------
// is_cancelled
// ---------------------------------------------------------------------------

#[test]
fn test_is_cancelled_false_initially() {
    let counter = StreamingTokenCounter::new(100);
    assert!(!counter.is_cancelled());
}

#[test]
fn test_is_cancelled_after_exceed() {
    let mut counter = StreamingTokenCounter::new(1);
    counter.process_chunk("hello world"); // > 1 token
    assert!(counter.is_cancelled());
}
