use hudhudscript_tools::telemetry::{
    record_tool_telemetry, ExecutionStatus, TelemetryCollector, ToolStats, ToolTelemetryRecord,
};
use std::time::Duration;

fn make_record(tool: &str, ok: bool, dur_ms: u64) -> ToolTelemetryRecord {
    ToolTelemetryRecord {
        tool_name: tool.to_string(),
        duration: Duration::from_millis(dur_ms),
        status: if ok {
            ExecutionStatus::Success
        } else {
            ExecutionStatus::Failure
        },
        input_size_bytes: 100,
        output_size_bytes: if ok { 200 } else { 0 },
        error: if ok { None } else { Some("oops".into()) },
        output_tokens_estimated: if ok { 50 } else { 0 },
    }
}

#[test]
fn test_collector_record_and_retrieve() {
    let col = TelemetryCollector::new(100);
    col.record(make_record("tool_a", true, 50));
    col.record(make_record("tool_a", false, 200));

    assert_eq!(col.record_count(), 2);

    let stats = col.stats_for("tool_a").unwrap();
    assert_eq!(stats.call_count, 2);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failure_count, 1);
}

#[test]
fn test_stats_avg_duration() {
    let col = TelemetryCollector::default();
    col.record(make_record("t", true, 100));
    col.record(make_record("t", true, 200));

    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.avg_duration(), Some(Duration::from_millis(150)));
}

#[test]
fn test_stats_success_rate() {
    let col = TelemetryCollector::default();
    col.record(make_record("t", true, 10));
    col.record(make_record("t", false, 10));
    col.record(make_record("t", false, 10));

    let stats = col.stats_for("t").unwrap();
    assert!((stats.success_rate() - 1.0 / 3.0).abs() < 1e-9);
}

#[test]
fn test_collector_ring_buffer() {
    let col = TelemetryCollector::new(3);
    for _ in 0..5 {
        col.record(make_record("t", true, 10));
    }
    // Should retain at most 3
    assert_eq!(col.record_count(), 3);
}

#[test]
fn test_collector_clear() {
    let col = TelemetryCollector::default();
    col.record(make_record("t", true, 10));
    col.clear();
    assert_eq!(col.record_count(), 0);
    assert!(col.stats_for("t").is_none());
}

#[test]
fn test_all_stats_multiple_tools() {
    let col = TelemetryCollector::default();
    col.record(make_record("tool_a", true, 10));
    col.record(make_record("tool_b", false, 20));

    let all = col.all_stats();
    assert!(all.contains_key("tool_a"));
    assert!(all.contains_key("tool_b"));
}

#[test]
fn test_record_tool_telemetry_convenience() {
    let col = TelemetryCollector::default();
    record_tool_telemetry(
        &col,
        "my_tool",
        Duration::from_millis(75),
        ExecutionStatus::Success,
        50,
        120,
        None,
    );
    let stats = col.stats_for("my_tool").unwrap();
    assert_eq!(stats.call_count, 1);
    assert_eq!(stats.total_input_bytes, 50);
    assert_eq!(stats.total_output_bytes, 120);
}

#[test]
fn test_min_max_duration() {
    let col = TelemetryCollector::default();
    col.record(make_record("t", true, 10));
    col.record(make_record("t", true, 50));
    col.record(make_record("t", true, 30));

    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.min_duration, Some(Duration::from_millis(10)));
    assert_eq!(stats.max_duration, Some(Duration::from_millis(50)));
}

// ---- ToolStats default ----

#[test]
fn test_tool_stats_default() {
    let stats = ToolStats::default();
    assert_eq!(stats.call_count, 0);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.failure_count, 0);
    assert_eq!(stats.total_duration, Duration::ZERO);
    assert!(stats.min_duration.is_none());
    assert!(stats.max_duration.is_none());
    assert_eq!(stats.total_input_bytes, 0);
    assert_eq!(stats.total_output_bytes, 0);
}

// ---- ToolStats avg_duration with zero calls ----

#[test]
fn test_tool_stats_avg_duration_zero_calls() {
    let stats = ToolStats::default();
    assert!(stats.avg_duration().is_none());
}

// ---- ToolStats success_rate with zero calls ----

#[test]
fn test_tool_stats_success_rate_zero_calls() {
    let stats = ToolStats::default();
    assert_eq!(stats.success_rate(), 0.0);
}

// ---- ToolStats 100% success rate ----

#[test]
fn test_tool_stats_all_success() {
    let col = TelemetryCollector::default();
    for _ in 0..10 {
        col.record(make_record("t", true, 10));
    }
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.success_rate(), 1.0);
    assert_eq!(stats.call_count, 10);
    assert_eq!(stats.failure_count, 0);
}

// ---- ToolStats 0% success rate ----

#[test]
fn test_tool_stats_all_failure() {
    let col = TelemetryCollector::default();
    for _ in 0..5 {
        col.record(make_record("t", false, 10));
    }
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.success_rate(), 0.0);
    assert_eq!(stats.failure_count, 5);
}

// ---- ExecutionStatus Display ----

#[test]
fn test_execution_status_display() {
    assert_eq!(ExecutionStatus::Success.to_string(), "success");
    assert_eq!(ExecutionStatus::Failure.to_string(), "failure");
}

// ---- ExecutionStatus serde roundtrip ----

#[test]
fn test_execution_status_serde_roundtrip() {
    let json = serde_json::to_string(&ExecutionStatus::Success).unwrap();
    let deser: ExecutionStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deser, ExecutionStatus::Success);

    let json = serde_json::to_string(&ExecutionStatus::Failure).unwrap();
    let deser: ExecutionStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deser, ExecutionStatus::Failure);
}

// ---- stats_for nonexistent tool ----

#[test]
fn test_stats_for_nonexistent() {
    let col = TelemetryCollector::default();
    assert!(col.stats_for("no_such_tool").is_none());
}

// ---- all_records returns ordered ----

#[test]
fn test_all_records_returns_records() {
    let col = TelemetryCollector::new(100);
    col.record(make_record("a", true, 10));
    col.record(make_record("b", false, 20));
    let records = col.all_records();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].tool_name, "a");
    assert_eq!(records[1].tool_name, "b");
}

// ---- record_tool_telemetry failure path ----

#[test]
fn test_record_tool_telemetry_failure() {
    let col = TelemetryCollector::default();
    record_tool_telemetry(
        &col,
        "fail_tool",
        Duration::from_millis(500),
        ExecutionStatus::Failure,
        100,
        0,
        Some("timeout error".into()),
    );
    let stats = col.stats_for("fail_tool").unwrap();
    assert_eq!(stats.call_count, 1);
    assert_eq!(stats.failure_count, 1);
    assert_eq!(stats.success_count, 0);
}

// ---- TelemetryCollector ring buffer oldest is dropped ----

#[test]
fn test_collector_ring_buffer_drops_oldest() {
    let col = TelemetryCollector::new(2);
    col.record(make_record("first", true, 10));
    col.record(make_record("second", true, 20));
    col.record(make_record("third", true, 30));
    let records = col.all_records();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].tool_name, "second");
    assert_eq!(records[1].tool_name, "third");
}

// ---- ToolStats input/output bytes accumulation ----

#[test]
fn test_tool_stats_byte_accumulation() {
    let col = TelemetryCollector::default();
    col.record(make_record("t", true, 10)); // input=100, output=200
    col.record(make_record("t", true, 20)); // input=100, output=200
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.total_input_bytes, 200);
    assert_eq!(stats.total_output_bytes, 400);
}

// ---- ToolTelemetryRecord serialization ----

#[test]
fn test_tool_telemetry_record_serde() {
    let record = make_record("serde_test", true, 42);
    let json = serde_json::to_string(&record).unwrap();
    let deser: ToolTelemetryRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.tool_name, "serde_test");
    assert_eq!(deser.status, ExecutionStatus::Success);
    assert_eq!(deser.input_size_bytes, 100);
}
