//! Council tests — adapted to current hudhudscript-orchestration API

use hudhudscript_orchestration::*;
use std::sync::Arc;

/// Mock agent executor that always succeeds (for testing council voting logic)
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

fn make_executor() -> CouncilExecutor {
    CouncilExecutor::with_agent_executor(
        Arc::new(EventBus::new()),
        Arc::new(SuccessExecutor),
    )
}

fn members() -> Vec<CouncilMember> {
    vec![
        CouncilMember::new("agent-1", "voter"),
        CouncilMember::new("agent-2", "voter"),
        CouncilMember::new("agent-3", "voter"),
    ]
}

#[tokio::test]
async fn test_parallel_execution() {
    let exec = make_executor();
    exec.register("c1".to_string(), CouncilConfig::default())
        .await;
    let result = exec
        .execute("c1", members(), serde_json::json!({"q": "test"}), None)
        .await
        .unwrap();
    assert_eq!(result.member_results.len(), 3);
    assert_eq!(result.decision, CouncilDecision::Approved);
}

#[tokio::test]
async fn test_sequential_execution() {
    let exec = make_executor();
    exec.register(
        "c2".to_string(),
        CouncilConfig {
            execution: ExecutionStrategy::Sequential,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("c2", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    assert_eq!(result.member_results.len(), 3);
}

#[tokio::test]
async fn test_round_robin_execution() {
    let exec = make_executor();
    exec.register(
        "c3".to_string(),
        CouncilConfig {
            execution: ExecutionStrategy::RoundRobin,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("c3", members(), serde_json::json!({"v": 1}), None)
        .await
        .unwrap();
    assert_eq!(result.member_results.len(), 3);
}

#[tokio::test]
async fn test_competitive_execution_with_quorum() {
    let exec = make_executor();
    exec.register(
        "c4".to_string(),
        CouncilConfig {
            execution: ExecutionStrategy::Competitive,
            quorum: 2,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("c4", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    assert_eq!(result.member_results.len(), 2);
}

#[tokio::test]
async fn test_majority_voting() {
    let exec = make_executor();
    exec.register(
        "c5".to_string(),
        CouncilConfig {
            voting: VotingAlgorithm::Majority,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("c5", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    assert_eq!(result.decision, CouncilDecision::Approved);
    assert_eq!(result.votes_for, 3);
}

#[tokio::test]
async fn test_unanimous_voting() {
    let exec = make_executor();
    exec.register(
        "c6".to_string(),
        CouncilConfig {
            voting: VotingAlgorithm::Unanimous,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("c6", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    assert_eq!(result.decision, CouncilDecision::Approved);
}

#[tokio::test]
async fn test_session_hooks_called() {
    let exec = make_executor();
    exec.register("c7".to_string(), CouncilConfig::default())
        .await;

    let start_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let complete_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let sc = start_called.clone();
    let cc = complete_called.clone();

    let hooks = SessionHooks {
        on_start: Some(Box::new(move |_, _| {
            sc.store(true, std::sync::atomic::Ordering::SeqCst);
        })),
        on_vote: None,
        on_complete: Some(Box::new(move |_, _| {
            cc.store(true, std::sync::atomic::Ordering::SeqCst);
        })),
    };

    exec.execute("c7", members(), serde_json::json!({}), Some(hooks))
        .await
        .unwrap();
    assert!(start_called.load(std::sync::atomic::Ordering::SeqCst));
    assert!(complete_called.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn test_weighted_voting() {
    let exec = make_executor();
    exec.register(
        "cw".to_string(),
        CouncilConfig {
            voting: VotingAlgorithm::Weighted,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("cw", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    // Default stub returns vote=true for all, so weighted should approve
    assert_eq!(result.decision, CouncilDecision::Approved);
}

#[tokio::test]
async fn test_first_wins_voting() {
    let exec = make_executor();
    exec.register(
        "cf".to_string(),
        CouncilConfig {
            voting: VotingAlgorithm::FirstWins,
            ..Default::default()
        },
    )
    .await;
    let result = exec
        .execute("cf", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    // Default stub returns vote=true, so first wins = approved
    assert_eq!(result.decision, CouncilDecision::Approved);
}

#[tokio::test]
async fn test_on_vote_hook_called() {
    let exec = make_executor();
    exec.register("cv".to_string(), CouncilConfig::default())
        .await;

    let vote_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let vc = vote_count.clone();

    let hooks = SessionHooks {
        on_start: None,
        on_vote: Some(Box::new(move |_, _| {
            vc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        })),
        on_complete: None,
    };

    exec.execute("cv", members(), serde_json::json!({}), Some(hooks))
        .await
        .unwrap();
    assert_eq!(vote_count.load(std::sync::atomic::Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_council_config_default() {
    let config = CouncilConfig::default();
    assert_eq!(config.execution, ExecutionStrategy::Parallel);
    assert_eq!(config.voting, VotingAlgorithm::Majority);
    assert_eq!(config.timeout_secs, 60);
    assert_eq!(config.quorum, 0);
}

#[test]
fn test_council_member_new() {
    let m = CouncilMember::new("agent-1", "voter");
    assert_eq!(m.agent_id, "agent-1");
    assert_eq!(m.role, "voter");
    assert_eq!(m.weight, 1.0);
}

#[tokio::test]
async fn test_unregistered_council_uses_defaults() {
    let exec = make_executor();
    // Execute without registering — should use default config
    let result = exec
        .execute("unregistered", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    assert_eq!(result.member_results.len(), 3);
}

#[tokio::test]
async fn test_custom_agent_executor() {
    use hudhudscript_orchestration::agent_executor::AgentTaskResult;


    struct RejectExecutor;

    #[async_trait::async_trait]
    impl AgentExecutor for RejectExecutor {
        async fn execute(&self, agent_id: &str, _task: AgentTask) -> AgentTaskResult {
            AgentTaskResult {
                success: true,
                output: serde_json::json!({"rejected_by": agent_id}),
                confidence: 0.5,
                vote: Some(false),
                error: None,
            }
        }
    }

    let exec =
        CouncilExecutor::with_agent_executor(Arc::new(EventBus::new()), Arc::new(RejectExecutor));
    exec.register("cx".to_string(), CouncilConfig::default())
        .await;
    let result = exec
        .execute("cx", members(), serde_json::json!({}), None)
        .await
        .unwrap();
    assert_eq!(result.decision, CouncilDecision::Rejected);
    assert_eq!(result.votes_against, 3);
    assert_eq!(result.votes_for, 0);
}

#[test]
fn test_council_error_display_all_variants() {
    let e1 = CouncilError::NotFound("c1".to_string());
    assert!(format!("{}", e1).contains("Council not found: c1"));

    let e2 = CouncilError::NoMembers;
    assert!(format!("{}", e2).contains("No members in council"));

    let e3 = CouncilError::ExecutionFailed("oops".to_string());
    assert!(format!("{}", e3).contains("Execution failed: oops"));

    let e4 = CouncilError::Timeout;
    assert!(format!("{}", e4).contains("Timeout"));
}

#[test]
fn test_execution_strategy_default() {
    let s = ExecutionStrategy::default();
    assert_eq!(s, ExecutionStrategy::Parallel);
}

#[test]
fn test_voting_algorithm_default() {
    let v = VotingAlgorithm::default();
    assert_eq!(v, VotingAlgorithm::Majority);
}

#[test]
fn test_council_decision_variants() {
    assert_ne!(CouncilDecision::Approved, CouncilDecision::Rejected);
    assert_ne!(CouncilDecision::Approved, CouncilDecision::Inconclusive);
    assert_ne!(CouncilDecision::Rejected, CouncilDecision::Inconclusive);
}
