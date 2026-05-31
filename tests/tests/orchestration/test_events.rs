//! Tests extracted from hudhudscript-orchestration/src/events.rs

use hudhudscript_orchestration::*;

use tokio::time::{timeout, Duration};
use hudhudscript_orchestration::orchestration::types::{StepConfig, StepType, WorkflowStep};

#[tokio::test]
async fn test_event_bus_emit_and_receive() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe();

    let event = AgentEvent::TaskCompleted {
        agent_id: "agent-1".to_string(),
        task_id: "task-1".to_string(),
        output: serde_json::json!({"result": "ok"}),
    };

    bus.emit(event.clone()).await.unwrap();

    let received = timeout(Duration::from_millis(100), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(received.event_type(), "task_completed");
    assert_eq!(received.agent_id(), "agent-1");
}

#[tokio::test]
async fn test_event_bus_multiple_subscribers() {
    let bus = EventBus::new();
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    bus.emit(AgentEvent::Custom {
        agent_id: "a".to_string(),
        event_type: "ping".to_string(),
        payload: serde_json::json!({}),
    })
    .await
    .unwrap();

    let e1 = timeout(Duration::from_millis(100), rx1.recv())
        .await
        .unwrap()
        .unwrap();
    let e2 = timeout(Duration::from_millis(100), rx2.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(e1.event_type(), "ping");
    assert_eq!(e2.event_type(), "ping");
}

#[tokio::test]
async fn test_event_stats() {
    let bus = EventBus::new();
    let _rx = bus.subscribe();

    bus.emit(AgentEvent::TaskCompleted {
        agent_id: "a".to_string(),
        task_id: "t".to_string(),
        output: serde_json::json!({}),
    })
    .await
    .unwrap();

    bus.emit(AgentEvent::TaskFailed {
        agent_id: "a".to_string(),
        task_id: "t".to_string(),
        error: "oops".to_string(),
    })
    .await
    .unwrap();

    let stats = bus.stats().await;
    assert_eq!(stats.total_emitted, 2);
    assert_eq!(stats.by_type["task_completed"], 1);
    assert_eq!(stats.by_type["task_failed"], 1);
}

#[tokio::test]
async fn test_emit_no_subscribers_returns_error() {
    let bus = EventBus::new();
    // No subscribers registered
    let result = bus
        .emit(AgentEvent::Custom {
            agent_id: "a".to_string(),
            event_type: "test".to_string(),
            payload: serde_json::json!({}),
        })
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EventBusError::NoSubscribers));
}

#[tokio::test]
async fn test_subscriber_count() {
    let bus = EventBus::new();
    assert_eq!(bus.subscriber_count(), 0);
    let _rx1 = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 1);
    let _rx2 = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 2);
}

#[tokio::test]
async fn test_event_bus_default() {
    let bus = EventBus::default();
    let _rx = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 1);
}

#[test]
fn test_agent_event_type_all_variants() {
    let events = vec![
        AgentEvent::TaskCompleted {
            agent_id: "a".into(),
            task_id: "t".into(),
            output: serde_json::json!({}),
        },
        AgentEvent::TaskFailed {
            agent_id: "a".into(),
            task_id: "t".into(),
            error: "e".into(),
        },
        AgentEvent::StateChanged {
            agent_id: "a".into(),
            key: "k".into(),
            value: serde_json::json!(1),
        },
        AgentEvent::VoteCompleted {
            council_id: "c".into(),
            decision: "Approved".into(),
            votes_for: 3,
            votes_against: 0,
        },
        AgentEvent::SwarmConsensus {
            swarm_id: "s".into(),
            result: serde_json::json!({}),
        },
        AgentEvent::CoupTriggered {
            agent_id: "a".into(),
            target_agent_id: "t".into(),
            reason: "trust".into(),
        },
        AgentEvent::PermissionDenied {
            agent_id: "a".into(),
            resource: "file".into(),
            action: "write".into(),
        },
        AgentEvent::Custom {
            agent_id: "a".into(),
            event_type: "custom_type".into(),
            payload: serde_json::json!({}),
        },
    ];

    let expected_types = vec![
        "task_completed",
        "task_failed",
        "state_changed",
        "vote_completed",
        "swarm_consensus",
        "coup_triggered",
        "permission_denied",
        "custom_type",
    ];
    let expected_ids = vec!["a", "a", "a", "c", "s", "a", "a", "a"];

    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.event_type(), expected_types[i]);
        assert_eq!(event.agent_id(), expected_ids[i]);
    }
}

#[tokio::test]
async fn test_coup_event() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe();

    bus.emit(AgentEvent::CoupTriggered {
        agent_id: "authority".to_string(),
        target_agent_id: "failing-agent".to_string(),
        reason: "trust_score_below_threshold".to_string(),
    })
    .await
    .unwrap();

    let e = timeout(Duration::from_millis(100), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(e.event_type(), "coup_triggered");
}

#[test]
fn test_event_bus_error_display() {
    let e1 = EventBusError::NoSubscribers;
    assert!(format!("{}", e1).contains("No active subscribers"));

    let e2 = EventBusError::ChannelClosed;
    assert!(format!("{}", e2).contains("Channel closed"));
}
