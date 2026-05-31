//! Tests for tokenomics::tool_tracking
//! Extracted from inline #[cfg(test)] module

use hudhudscript_tokenomics::tool_tracking::*;

#[test]
fn test_record_call() {
    let mut tracker = ToolCallTracker::new();
    tracker.record_call(
        ToolCallType::Mcp,
        "github_search",
        Some("github"),
        150,
        100,
        5000,
        true,
    );
    assert_eq!(tracker.total_calls(), 1);
}

#[test]
fn test_start_complete_call() {
    let mut tracker = ToolCallTracker::new();
    let id = tracker.start_call(ToolCallType::Http, "fetch_url", None, 50);
    tracker.complete_call(id, 2000, true);
    let call = &tracker.calls()[0];
    assert!(call.completed_at.is_some());
    assert!(call.success);
    assert_eq!(call.output_size_bytes, 2000);
}

#[test]
fn test_cost_rates() {
    let mut tracker = ToolCallTracker::new();
    tracker.set_cost_rate("premium_api".into(), 0.01);
    tracker.record_call(ToolCallType::Http, "premium_api", None, 100, 50, 200, true);
    assert!((tracker.total_cost() - 0.01).abs() < 0.001);
}

#[test]
fn test_stats_by_type() {
    let mut tracker = ToolCallTracker::new();
    tracker.record_call(ToolCallType::Mcp, "tool1", None, 100, 50, 200, true);
    tracker.record_call(ToolCallType::Mcp, "tool2", None, 200, 50, 300, true);
    tracker.record_call(ToolCallType::Http, "api1", None, 150, 100, 500, false);

    let stats = tracker.stats_by_type();
    assert_eq!(stats[&ToolCallType::Mcp].total_calls, 2);
    assert_eq!(stats[&ToolCallType::Mcp].successful_calls, 2);
    assert_eq!(stats[&ToolCallType::Http].failed_calls, 1);
}

#[test]
fn test_stats_by_name() {
    let mut tracker = ToolCallTracker::new();
    tracker.record_call(ToolCallType::Mcp, "search", None, 100, 50, 200, true);
    tracker.record_call(ToolCallType::Mcp, "search", None, 150, 60, 250, true);
    tracker.record_call(ToolCallType::Mcp, "execute", None, 200, 80, 300, true);

    let stats = tracker.stats_by_name();
    assert_eq!(stats["search"].total_calls, 2);
    assert_eq!(stats["execute"].total_calls, 1);
}

#[test]
fn test_empty_tracker() {
    let tracker = ToolCallTracker::new();
    assert_eq!(tracker.total_calls(), 0);
    assert_eq!(tracker.total_cost(), 0.0);
    // stats_by_type and stats_by_name should be empty, not panic
    assert!(tracker.stats_by_type().is_empty());
    assert!(tracker.stats_by_name().is_empty());
    assert!(tracker.calls().is_empty());
}

#[test]
fn test_cost_rate_applied_correctly() {
    let mut tracker = ToolCallTracker::new();
    tracker.set_cost_rate("expensive_api".into(), 0.05);

    // Record 3 calls to the tool with the cost rate
    for _ in 0..3 {
        tracker.record_call(
            ToolCallType::Http,
            "expensive_api",
            None,
            100,
            50,
            200,
            true,
        );
    }

    // Each call should carry the cost rate; total = 3 * 0.05 = 0.15
    assert_eq!(tracker.total_calls(), 3);
    assert!(
        (tracker.total_cost() - 0.15).abs() < 1e-9,
        "expected total cost 0.15, got {}",
        tracker.total_cost(),
    );

    // Verify each individual call has the correct cost
    for call in tracker.calls() {
        assert!(
            (call.estimated_cost_usd - 0.05).abs() < 1e-9,
            "each call should have cost 0.05, got {}",
            call.estimated_cost_usd,
        );
    }
}

#[test]
fn test_multiple_cost_rates() {
    let mut tracker = ToolCallTracker::new();
    tracker.set_cost_rate("search".into(), 0.001);
    tracker.set_cost_rate("premium_api".into(), 0.10);

    tracker.record_call(ToolCallType::Mcp, "search", None, 100, 50, 200, true);
    tracker.record_call(ToolCallType::Http, "premium_api", None, 100, 50, 200, true);
    // Tool without a cost rate should default to 0.0
    tracker.record_call(
        ToolCallType::FileSystem,
        "read_file",
        None,
        100,
        50,
        200,
        true,
    );

    let by_name = tracker.stats_by_name();
    assert!(
        (by_name["search"].total_cost_usd - 0.001).abs() < 1e-9,
        "search cost should be 0.001, got {}",
        by_name["search"].total_cost_usd,
    );
    assert!(
        (by_name["premium_api"].total_cost_usd - 0.10).abs() < 1e-9,
        "premium_api cost should be 0.10, got {}",
        by_name["premium_api"].total_cost_usd,
    );
    assert_eq!(
        by_name["read_file"].total_cost_usd, 0.0,
        "tool without cost rate should have 0 cost",
    );

    // Total: 0.001 + 0.10 + 0.0 = 0.101
    assert!(
        (tracker.total_cost() - 0.101).abs() < 1e-9,
        "expected total 0.101, got {}",
        tracker.total_cost(),
    );
}

#[test]
fn test_default_impl() {
    // Line 61: Default for ToolCallTracker
    let tracker = ToolCallTracker::default();
    assert_eq!(tracker.total_calls(), 0);
}

#[test]
fn test_stats_by_type_zero_duration_avg() {
    // Line 159: avg_duration_ms when total_calls > 0 (covers the if branch)
    // This is already covered by existing tests, but let's explicitly verify the value
    let mut tracker = ToolCallTracker::new();
    tracker.record_call(ToolCallType::Mcp, "tool1", None, 100, 50, 200, true);
    tracker.record_call(ToolCallType::Mcp, "tool2", None, 200, 50, 300, true);
    let stats = tracker.stats_by_type();
    assert_eq!(stats[&ToolCallType::Mcp].avg_duration_ms, 150.0);
}

#[test]
fn test_stats_by_name_zero_duration_avg() {
    // Line 179: avg_duration_ms when total_calls > 0
    let mut tracker = ToolCallTracker::new();
    tracker.record_call(ToolCallType::Mcp, "search", None, 100, 50, 200, true);
    tracker.record_call(ToolCallType::Mcp, "search", None, 300, 60, 250, true);
    let stats = tracker.stats_by_name();
    assert_eq!(stats["search"].avg_duration_ms, 200.0);
}

// NOTE: tool_tracking.rs lines 159 and 179 contain `else { 0.0 }` branches
// for when total_calls == 0. These are dead code: entries in the HashMap only
// exist because at least one call was iterated, so total_calls is always > 0
// when the avg_duration_ms calculation runs.
