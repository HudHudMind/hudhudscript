//! Swarm tests — adapted to current hudhudscript-orchestration API

use hudhudscript_orchestration::*;
use std::sync::Arc;

/// Mock agent executor that always succeeds (for testing swarm consensus logic)
struct SuccessExecutor;

#[async_trait::async_trait]
impl AgentExecutor for SuccessExecutor {
    async fn execute(&self, agent_id: &str, _task: AgentTask) -> AgentTaskResult {
        AgentTaskResult {
            success: true,
            output: serde_json::json!({"agent": agent_id, "result": "ok"}),
            confidence: 0.9,
            vote: Some(true),
            error: None,
        }
    }
}

fn make_executor() -> SwarmExecutor {
    SwarmExecutor::with_agent_executor(Arc::new(EventBus::new()), Arc::new(SuccessExecutor))
}

fn agents() -> Vec<String> {
    vec!["a1".to_string(), "a2".to_string(), "a3".to_string()]
}

#[tokio::test]
async fn test_swarm_parallel_execution() {
    let exec = make_executor();
    exec.register("s1".to_string(), SwarmConfig::default())
        .await;
    let result = exec
        .execute("s1", agents(), serde_json::json!({"q": "test"}))
        .await
        .unwrap();
    assert_eq!(result.agent_results.len(), 3);
    assert!(result.success);
    assert!(result.failed_agents.is_empty());
}

#[tokio::test]
async fn test_swarm_state_sharing() {
    let exec = make_executor();
    exec.register("s2".to_string(), SwarmConfig::default())
        .await;
    exec.set_state("s2", "shared_key", serde_json::json!("shared_value"))
        .await;

    let val = exec.get_state("s2", "shared_key").await;
    assert_eq!(val, Some(serde_json::json!("shared_value")));
}

#[tokio::test]
async fn test_swarm_state_updated_after_execution() {
    let exec = make_executor();
    exec.register("s3".to_string(), SwarmConfig::default())
        .await;
    exec.execute("s3", agents(), serde_json::json!({}))
        .await
        .unwrap();

    let success_count = exec.get_state("s3", "success_count").await;
    assert_eq!(success_count, Some(serde_json::json!(3)));
}

#[tokio::test]
async fn test_swarm_fault_tolerance() {
    let exec = make_executor();
    exec.register(
        "s4".to_string(),
        SwarmConfig {
            fault_tolerant: true,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("s4", agents(), serde_json::json!({}))
        .await
        .unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_swarm_no_agents_error() {
    let exec = make_executor();
    exec.register("s5".to_string(), SwarmConfig::default())
        .await;
    let result = exec.execute("s5", vec![], serde_json::json!({})).await;
    assert!(matches!(result, Err(SwarmError::NoAgents)));
}

#[tokio::test]
async fn test_consensus_first_success() {
    let exec = make_executor();
    exec.register(
        "s6".to_string(),
        SwarmConfig {
            consensus: ConsensusStrategy::FirstSuccess,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("s6", agents(), serde_json::json!({"v": 42}))
        .await
        .unwrap();
    assert!(!result.consensus_output.is_null());
}

#[tokio::test]
async fn test_consensus_aggregate() {
    let exec = make_executor();
    exec.register(
        "s7".to_string(),
        SwarmConfig {
            consensus: ConsensusStrategy::Aggregate,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("s7", agents(), serde_json::json!({}))
        .await
        .unwrap();
    assert!(result.consensus_output.is_array());
    assert_eq!(result.consensus_output.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_swarm_state_get_nonexistent_swarm() {
    let exec = make_executor();
    let val = exec.get_state("nonexistent", "key").await;
    assert!(val.is_none());
}

#[tokio::test]
async fn test_swarm_state_get_nonexistent_key() {
    let exec = make_executor();
    exec.register("s".to_string(), SwarmConfig::default()).await;
    let val = exec.get_state("s", "nonexistent_key").await;
    assert!(val.is_none());
}

#[tokio::test]
async fn test_swarm_set_state_nonexistent_swarm() {
    let exec = make_executor();
    // Should not panic
    exec.set_state("nonexistent", "key", serde_json::json!("value"))
        .await;
}

#[test]
fn test_swarm_state_set_get() {
    let mut state = SwarmState::default();
    assert!(state.variables.is_empty());
    state.set("key1", serde_json::json!(42));
    assert_eq!(state.get("key1"), Some(&serde_json::json!(42)));
    assert!(state.get("nonexistent").is_none());
}

#[test]
fn test_consensus_strategy_eq() {
    assert_eq!(ConsensusStrategy::Majority, ConsensusStrategy::Majority);
    assert_ne!(ConsensusStrategy::Majority, ConsensusStrategy::Aggregate);
}

#[test]
fn test_swarm_config_default() {
    let config = SwarmConfig::default();
    assert_eq!(config.consensus, ConsensusStrategy::Majority);
    assert_eq!(config.timeout_secs, 60);
    assert_eq!(config.min_success, 1);
    assert!(config.fault_tolerant);
}

#[tokio::test]
async fn test_custom_agent_executor() {
    use hudhudscript_orchestration::agent_executor::AgentTaskResult;

    struct HighConfidenceExecutor;

    #[async_trait::async_trait]
    impl AgentExecutor for HighConfidenceExecutor {
        async fn execute(&self, agent_id: &str, _task: AgentTask) -> AgentTaskResult {
            let confidence = match agent_id {
                "a1" => 0.3,
                "a2" => 0.95,
                "a3" => 0.7,
                _ => 0.5,
            };
            AgentTaskResult {
                success: true,
                output: serde_json::json!({"agent": agent_id, "confidence": confidence}),
                confidence,
                vote: Some(true),
                error: None,
            }
        }
    }

    let exec = SwarmExecutor::with_agent_executor(
        Arc::new(EventBus::new()),
        Arc::new(HighConfidenceExecutor),
    );
    exec.register(
        "sx".to_string(),
        SwarmConfig {
            consensus: ConsensusStrategy::HighestConfidence,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("sx", agents(), serde_json::json!({}))
        .await
        .unwrap();
    // Agent a2 has the highest confidence
    assert_eq!(result.consensus_output["agent"], "a2");
}

#[test]
fn test_swarm_error_display_all_variants() {
    let e1 = SwarmError::NoAgents;
    assert!(format!("{}", e1).contains("No agents in swarm"));

    let e2 = SwarmError::AgentFailed("a1: panicked".to_string());
    assert!(format!("{}", e2).contains("Agent failed: a1: panicked"));

    let e3 = SwarmError::InsufficientSuccess {
        required: 3,
        got: 1,
    };
    assert!(format!("{}", e3).contains("Insufficient success: required 3, got 1"));

    let e4 = SwarmError::Timeout;
    assert!(format!("{}", e4).contains("Timeout"));
}
