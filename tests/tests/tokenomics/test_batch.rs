//! Public API tests for tokenomics::batch

use chrono::Utc;
use hudhudscript_tokenomics::batch::*;
use uuid::Uuid;

fn make_request(model: &str) -> BatchRequest {
    BatchRequest {
        id: Uuid::new_v4(),
        model: model.into(),
        provider: "anthropic".into(),
        prompt: "test prompt".into(),
        system_prompt: None,
        max_tokens: Some(1000),
        temperature: None,
        enqueued_at: Utc::now(),
        metadata: serde_json::json!({}),
    }
}

#[test]
fn test_new() {
    let q = BatchQueue::new(100, 30, false);
    assert_eq!(q.queue_len(), 0);
    assert_eq!(q.batch_count(), 0);
}

#[test]
fn test_enqueue() {
    let mut q = BatchQueue::new(100, 30, true);
    q.enqueue(make_request("claude-sonnet-4"));
    assert_eq!(q.queue_len(), 1);
}

#[test]
fn test_flush() {
    let mut q = BatchQueue::new(5, 30, true);
    for _ in 0..3 {
        q.enqueue(make_request("claude-sonnet-4"));
    }
    let batch = q.flush().unwrap();
    assert_eq!(batch.requests.len(), 3);
    assert_eq!(batch.status, BatchStatus::Queued);
    assert_eq!(q.queue_len(), 0);
    assert_eq!(q.batch_count(), 1);
}

#[test]
fn test_empty_flush() {
    let mut q = BatchQueue::new(10, 30, true);
    assert!(q.flush().is_none());
}

#[test]
fn test_partial_flush() {
    let mut q = BatchQueue::new(2, 30, true);
    for _ in 0..5 {
        q.enqueue(make_request("m"));
    }
    let batch = q.flush().unwrap();
    assert_eq!(batch.requests.len(), 2);
    assert_eq!(q.queue_len(), 3);
}

#[test]
fn test_classify_dispatch_stream() {
    let q = BatchQueue::new(100, 30, false);
    assert_eq!(
        q.classify_dispatch(true, false, false),
        DispatchMode::Stream
    );
    assert_eq!(
        q.classify_dispatch(false, true, false),
        DispatchMode::Stream
    );
}

#[test]
fn test_classify_dispatch_batch_auto_promote() {
    let q = BatchQueue::new(100, 30, true);
    assert_eq!(
        q.classify_dispatch(false, false, false),
        DispatchMode::Batch
    );
}

#[test]
fn test_classify_dispatch_batch_cost_optimize() {
    let q = BatchQueue::new(100, 30, false);
    assert_eq!(q.classify_dispatch(false, false, true), DispatchMode::Batch);
}

#[test]
fn test_classify_dispatch_async() {
    let q = BatchQueue::new(100, 30, false);
    assert_eq!(
        q.classify_dispatch(false, false, false),
        DispatchMode::Async
    );
}

#[test]
fn test_should_flush_by_size() {
    let mut q = BatchQueue::new(2, 9999, true);
    q.enqueue(make_request("a"));
    assert!(!q.should_flush());
    q.enqueue(make_request("b"));
    assert!(q.should_flush());
}

#[test]
fn test_pending_batches() {
    let mut q = BatchQueue::new(10, 30, true);
    q.enqueue(make_request("m"));
    q.flush();
    let pending = q.pending_batches();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].status, BatchStatus::Queued);
}

#[test]
fn test_batch_request_fields() {
    let req = make_request("claude-sonnet-4");
    assert_eq!(req.model, "claude-sonnet-4");
    assert_eq!(req.provider, "anthropic");
    assert_eq!(req.prompt, "test prompt");
    assert!(req.system_prompt.is_none());
    assert_eq!(req.max_tokens, Some(1000));
    assert!(req.temperature.is_none());
}

#[test]
fn test_dispatch_mode_eq() {
    assert_eq!(DispatchMode::Stream, DispatchMode::Stream);
    assert_eq!(DispatchMode::Async, DispatchMode::Async);
    assert_eq!(DispatchMode::Batch, DispatchMode::Batch);
    assert_ne!(DispatchMode::Stream, DispatchMode::Batch);
}

#[test]
fn test_batch_status_eq() {
    assert_eq!(BatchStatus::Queued, BatchStatus::Queued);
    assert_eq!(BatchStatus::Completed, BatchStatus::Completed);
    assert_ne!(BatchStatus::Queued, BatchStatus::Completed);
}

#[test]
fn test_total_savings_calculation() {
    let mut q = BatchQueue::new(10, 30, true);
    for _ in 0..3 {
        q.enqueue(make_request("claude-sonnet-4"));
    }
    let mut batch = q.flush().unwrap();

    // Simulate completion with known costs
    batch.status = BatchStatus::Completed;
    batch.results = vec![
        BatchResult {
            request_id: Uuid::new_v4(),
            content: "r1".into(),
            input_tokens: 1000,
            output_tokens: 500,
            cost_usd: 0.10,
            completed_at: Utc::now(),
        },
        BatchResult {
            request_id: Uuid::new_v4(),
            content: "r2".into(),
            input_tokens: 2000,
            output_tokens: 800,
            cost_usd: 0.20,
            completed_at: Utc::now(),
        },
    ];

    // Replace the stored batch with the completed one
    let mut q2 = BatchQueue::new(10, 30, true);
    q2.batches = vec![batch];

    // pricing_fn returns the full price; total_savings applies 50% discount
    let savings = q2.total_savings(|r| (r.input_tokens + r.output_tokens) as f64 * 0.001);
    // result1: (1000+500)*0.001 = 1.5, result2: (2000+800)*0.001 = 2.8
    // savings = (1.5 + 2.8) * 0.50 = 2.15
    let expected = ((1000 + 500) as f64 * 0.001 + (2000 + 800) as f64 * 0.001) * 0.50;
    assert!(
        (savings - expected).abs() < 1e-9,
        "expected savings {}, got {}",
        expected,
        savings,
    );
}

#[test]
fn test_time_based_flush() {
    let mut q = BatchQueue::new(100, 1, true); // 1-second flush interval
    q.enqueue(make_request("m"));
    // Queue has 1 item but max_batch_size=100, so size threshold not met
    assert!(
        !q.should_flush(),
        "should not flush immediately (time not elapsed)"
    );

    // Manipulate last_flush to simulate elapsed time
    q.last_flush = Utc::now() - chrono::Duration::seconds(2);
    assert!(
        q.should_flush(),
        "should flush after flush_interval_seconds has elapsed with non-empty queue",
    );

    // Empty queue should NOT trigger time-based flush
    let batch = q.flush().unwrap();
    assert_eq!(batch.requests.len(), 1);
    assert!(
        !q.should_flush(),
        "empty queue should not trigger flush even after interval"
    );
}
