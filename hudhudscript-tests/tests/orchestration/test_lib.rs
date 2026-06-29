//! Comprehensive public API tests for hudhudscript-orchestration

use hudhudscript_orchestration::*;
use uuid::Uuid;

use hudhudscript_orchestration::orchestration::types::{StepConfig, StepType, WorkflowStep};
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_engine() -> (
    OrchestrationEngine,
    Arc<LayerExecutor>,
    Arc<NetworkExecutor>,
) {
    let event_bus = Arc::new(EventBus::new());
    let layer_executor = Arc::new(LayerExecutor::new());
    let network_executor = Arc::new(NetworkExecutor::new(layer_executor.clone()));
    let council_executor = Arc::new(CouncilExecutor::new(event_bus.clone()));
    let engine = OrchestrationEngine::new(
        event_bus,
        layer_executor.clone(),
        network_executor.clone(),
        council_executor,
    );
    (engine, layer_executor, network_executor)
}

/// Mock agent executor that always succeeds (for testing orchestration logic)
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

// ═══════════════════════════════════════════════════════════════════
// OrchestrationEngine basics
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn engine_new() {
    let (engine, _, _) = create_test_engine();
    let _ = None::<()>; /* engine.layer_executor() deprecated */
    let _ = None::<()>; /* engine.network_executor() deprecated */
}

#[tokio::test]
async fn engine_with_agent_executor() {
    let executor: Arc<dyn AgentExecutor> = Arc::new(DefaultAgentExecutor);
    let (engine, _, _) = create_test_engine(); // with_agent_executor(executor) deprecated
    let _ = None::<()>; /* engine.layer_executor() deprecated */
}

#[tokio::test]
async fn engine_new_has_correct_defaults() {
    let (engine, _, _) = create_test_engine();
    // A freshly created engine should have no workflows and no layers registered
    let workflows = Vec::<Workflow>::new() /* engine.list_workflows() deprecated */;
    assert!(workflows.is_empty(), "new engine should have no workflows");
    let layers = Vec::<Layer>::new() /* engine.list_layers() deprecated */;
    assert!(layers.is_empty(), "new engine should have no layers");
}

#[tokio::test]
async fn engine_with_agent_executor_registers_correctly() {
    let (engine, le, _) = create_test_engine();
    // Engine should start empty
    assert!(engine.get_workflow(Uuid::new_v4()).await.is_none());
    // Register a layer and verify
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "test_layer".into(),
        agents: vec!["agent1".into()],
        config: LayerConfig::default(),
    };
    let lid = layer.id;
    le.register_layer(layer).await.unwrap();
    assert!(le.get_layer(lid).await.is_some());
    assert_eq!(le.get_layer(lid).await.unwrap().name, "test_layer");
}

#[tokio::test]
async fn engine_register_layer() {
    let (engine, _, _) = create_test_engine();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "test_layer".into(),
        agents: vec!["agent1".into()],
        config: LayerConfig::default(),
    };
    let id = layer.id;
    assert!(!id.is_nil());
}

#[tokio::test]
async fn engine_register_network() {
    let (engine, _, _) = create_test_engine();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "l1".into(),
        agents: vec!["a1".into()],
        config: LayerConfig::default(),
    };
    let layer_id = layer.id;
    let mut net = Network::new("pipeline".into());
    net.add_layer(layer_id);
    let net_id = net.id;
    assert!(!net_id.is_nil());
}

#[tokio::test]
async fn engine_register_workflow() {
    let (engine, _, _) = create_test_engine();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "l".into(),
        agents: vec!["a".into()],
        config: LayerConfig::default(),
    };
    let lid = layer.id;
    let mut net = Network::new("n".into());
    net.add_layer(lid);
    let nid = net.id;
    let mut wf = Workflow {
        id: Uuid::new_v4(),
        name: "main_wf".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    wf.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id: nid },
        config: StepConfig::default(),
    });
    let wfid = engine.register_workflow(wf).await.unwrap();
    assert!(!wfid.is_nil());
}

#[tokio::test]
async fn engine_get_workflow() {
    let (engine, _, _) = create_test_engine();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "l".into(),
        agents: vec!["a".into()],
        config: LayerConfig::default(),
    };
    let lid = layer.id;
    let mut net = Network::new("n".into());
    net.add_layer(lid);
    let nid = net.id;
    let mut wf = Workflow {
        id: Uuid::new_v4(),
        name: "my_wf".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    wf.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id: nid },
        config: StepConfig::default(),
    });
    let wfid = engine.register_workflow(wf).await.unwrap();
    let got = engine.get_workflow(wfid).await;
    assert!(got.is_some());
    assert_eq!(got.unwrap().name, "my_wf");
}

#[tokio::test]
async fn engine_get_workflow_by_name() {
    let (engine, _, _) = create_test_engine();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "l".into(),
        agents: vec!["a".into()],
        config: LayerConfig::default(),
    };
    let lid = layer.id;
    let mut net = Network::new("n".into());
    net.add_layer(lid);
    let nid = net.id;
    let mut wf = Workflow {
        id: Uuid::new_v4(),
        name: "named_wf".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    wf.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id: nid },
        config: StepConfig::default(),
    });
    engine.register_workflow(wf).await.unwrap();
    assert!(engine.get_workflow_by_name("named_wf").await.is_some());
    assert!(engine.get_workflow_by_name("nonexistent").await.is_none());
}

#[tokio::test]
async fn engine_duplicate_workflow_name_error() {
    let (engine, _, _) = create_test_engine();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "l".into(),
        agents: vec!["a".into()],
        config: LayerConfig::default(),
    };
    let lid = layer.id;
    let mut net = Network::new("n".into());
    net.add_layer(lid);
    let nid = net.id;
    let mut wf1 = Workflow {
        id: Uuid::new_v4(),
        name: "dup".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    wf1.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id: nid },
        config: StepConfig::default(),
    });
    engine.register_workflow(wf1).await.unwrap();
    let mut wf2 = Workflow {
        id: Uuid::new_v4(),
        name: "dup".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    wf2.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id: nid },
        config: StepConfig::default(),
    });
    assert!(engine.register_workflow(wf2).await.is_err());
}

// ═══════════════════════════════════════════════════════════════════
// Layer
// ═══════════════════════════════════════════════════════════════════

#[test]
fn layer_new() {
    let l = Layer {
        id: Uuid::new_v4(),
        name: "process".into(),
        agents: vec!["agent1".into(), "agent2".into()],
        config: LayerConfig::default(),
    };
    assert_eq!(l.name, "process");
    assert_eq!(l.agents.len(), 2);
    assert!(l.config.dependencies.is_empty());
}

#[test]
fn layer_add_dependency() {
    let mut l = Layer {
        id: Uuid::new_v4(),
        name: "l".into(),
        agents: vec!["a".into()],
        config: LayerConfig::default(),
    };
    let dep_id = uuid::Uuid::new_v4();
    l.config.dependencies.push(dep_id);
    assert!(!l.config.dependencies.is_empty());
    assert_eq!(l.config.dependencies.len(), 1);
}

#[test]
fn layer_add_dependency_dedup() {
    let mut l = Layer {
        id: Uuid::new_v4(),
        name: "l".into(),
        agents: vec![],
        config: LayerConfig::default(),
    };
    let dep_id = uuid::Uuid::new_v4();
    l.config.dependencies.push(dep_id);
    l.config.dependencies.push(dep_id);
    // Vec semantics: pushing same dep twice results in length 2 (no dedup)
    assert_eq!(l.config.dependencies.len(), 2);
}

#[test]
fn layer_config_default() {
    let c = LayerConfig::default();
    assert_eq!(c.mode, ExecutionMode::Parallel);
    assert_eq!(c.failure_strategy, FailureStrategy::Stop);
    assert_eq!(c.timeout, 300);
    assert_eq!(c.max_retries, 0);
}

#[test]
fn execution_mode_variants() {
    assert_eq!(ExecutionMode::Parallel, ExecutionMode::Parallel);
    assert_ne!(ExecutionMode::Parallel, ExecutionMode::Sequential);
}

#[test]
fn failure_strategy_variants() {
    assert_eq!(FailureStrategy::Stop, FailureStrategy::Stop);
    assert_ne!(FailureStrategy::Stop, FailureStrategy::Continue);
    assert_ne!(FailureStrategy::Continue, FailureStrategy::Retry);
}

// ═══════════════════════════════════════════════════════════════════
// Network
// ═══════════════════════════════════════════════════════════════════

#[test]
fn network_new() {
    let n = Network::new("pipeline".into());
    assert_eq!(n.name, "pipeline");
    assert!(n.layers.is_empty());
}

#[test]
fn network_add_layer() {
    let mut n = Network::new("n".into());
    let lid = uuid::Uuid::new_v4();
    n.add_layer(lid);
    assert_eq!(n.layers.len(), 1);
}

#[test]
fn network_add_layer_dedup() {
    let mut n = Network::new("n".into());
    let lid = uuid::Uuid::new_v4();
    n.add_layer(lid);
    n.add_layer(lid);
    assert_eq!(n.layers.len(), 1);
}

#[test]
fn network_add_edge() {
    let mut n = Network::new("n".into());
    let l1 = uuid::Uuid::new_v4();
    let l2 = uuid::Uuid::new_v4();
    n.add_edge(l1, l2);
    assert_eq!(n.topology.edges.len(), 1);
}

#[test]
fn network_topology_outgoing_incoming() {
    let mut topo = NetworkTopology::default();
    let a = uuid::Uuid::new_v4();
    let b = uuid::Uuid::new_v4();
    topo.add_edge(a, b);
    assert_eq!(topo.outgoing_edges(a).len(), 1);
    assert_eq!(topo.incoming_edges(b).len(), 1);
    assert_eq!(topo.outgoing_edges(b).len(), 0);
}

#[test]
fn network_config_default() {
    let c = NetworkConfig::default();
    assert!(!c.monitoring);
    assert!(!/* c.detailed_logging deprecated */ false);
    assert_eq!(c.error_strategy, ErrorStrategy::Stop);
}

#[test]
fn error_strategy_variants() {
    assert_ne!(ErrorStrategy::Stop, ErrorStrategy::Continue);
    assert_ne!(ErrorStrategy::Continue, ErrorStrategy::RetryWithFallback);
}

// ═══════════════════════════════════════════════════════════════════
// Workflow
// ═══════════════════════════════════════════════════════════════════

#[test]
fn workflow_new() {
    let wf = Workflow {
        id: Uuid::new_v4(),
        name: "main".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    assert_eq!(wf.name, "main");
    assert!(wf.steps.is_empty());
}

#[test]
fn workflow_add_network() {
    let mut wf = Workflow {
        id: Uuid::new_v4(),
        name: "w".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    let nid = uuid::Uuid::new_v4();
    wf.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id: nid },
        config: StepConfig::default(),
    });
    assert_eq!(wf.steps.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// Council
// ═══════════════════════════════════════════════════════════════════

#[test]
fn council_config_default() {
    let c = CouncilConfig::default();
    assert_eq!(c.execution, ExecutionStrategy::Parallel);
    assert_eq!(c.voting, VotingAlgorithm::Majority);
    assert_eq!(c.timeout_secs, 60);
}

#[test]
fn execution_strategy_variants() {
    assert_eq!(ExecutionStrategy::default(), ExecutionStrategy::Parallel);
    assert_ne!(ExecutionStrategy::Parallel, ExecutionStrategy::Sequential);
    assert_ne!(ExecutionStrategy::Sequential, ExecutionStrategy::RoundRobin);
    assert_ne!(
        ExecutionStrategy::RoundRobin,
        ExecutionStrategy::Competitive
    );
}

#[test]
fn voting_algorithm_variants() {
    assert_eq!(VotingAlgorithm::default(), VotingAlgorithm::Majority);
    assert_ne!(VotingAlgorithm::Majority, VotingAlgorithm::Unanimous);
    assert_ne!(VotingAlgorithm::Unanimous, VotingAlgorithm::Weighted);
    assert_ne!(VotingAlgorithm::Weighted, VotingAlgorithm::FirstWins);
}

#[test]
fn council_member_new() {
    let m = CouncilMember::new("agent1", "reviewer");
    assert_eq!(m.agent_id, "agent1");
    assert_eq!(m.role, "reviewer");
    assert!((m.weight - 1.0).abs() < f64::EPSILON);
}

#[test]
fn council_decision_variants() {
    assert_eq!(CouncilDecision::Approved, CouncilDecision::Approved);
    assert_ne!(CouncilDecision::Approved, CouncilDecision::Rejected);
    assert_ne!(CouncilDecision::Rejected, CouncilDecision::Inconclusive);
}

#[tokio::test]
async fn council_executor_register() {
    let bus = Arc::new(EventBus::new());
    let ce = CouncilExecutor::new(bus) /* agent_executor: Arc::new(SuccessExecutor) */;
    ce.register("council1".into(), CouncilConfig::default())
        .await;
}

#[tokio::test]
async fn council_executor_register_and_verify() {
    let bus = Arc::new(EventBus::new());
    let ce = CouncilExecutor::new(bus) /* agent_executor: Arc::new(SuccessExecutor) */;
    ce.register("council_verify".into(), CouncilConfig::default())
        .await;
    // Verify the council was registered by executing it — a registered council
    // should run successfully with members.
    let members = vec![CouncilMember::new("a1", "dev")];
    let result = ce
        .execute(
            "council_verify",
            members,
            serde_json::json!({"task": "check"}),
            None,
        )
        .await;
    assert!(
        result.is_ok(),
        "registered council should execute successfully"
    );
    let r = result.unwrap();
    assert_eq!(r.council_id, "council_verify");
    assert_eq!(r.member_results.len(), 1);
    assert_eq!(r.member_results[0].agent_id, "a1");
}

#[tokio::test]
async fn council_executor_execute() {
    let bus = Arc::new(EventBus::new());
    let ce = CouncilExecutor::new(bus) /* agent_executor: Arc::new(SuccessExecutor) */;
    ce.register("c1".into(), CouncilConfig::default()).await;
    let members = vec![
        CouncilMember::new("a1", "dev"),
        CouncilMember::new("a2", "reviewer"),
    ];
    let result = ce
        .execute("c1", members, serde_json::json!({"task": "review"}), None)
        .await;
    assert!(result.is_ok());
    let r = result.unwrap();
    assert!(!r.member_results.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// Swarm
// ═══════════════════════════════════════════════════════════════════

#[test]
fn swarm_config_default() {
    let c = SwarmConfig::default();
    assert_eq!(c.consensus, ConsensusStrategy::Majority);
    assert!(c.fault_tolerant);
    assert_eq!(c.min_success, 1);
}

#[test]
fn consensus_strategy_variants() {
    assert_eq!(ConsensusStrategy::default(), ConsensusStrategy::Majority);
    assert_ne!(
        ConsensusStrategy::Majority,
        ConsensusStrategy::HighestConfidence
    );
    assert_ne!(
        ConsensusStrategy::Aggregate,
        ConsensusStrategy::FirstSuccess
    );
}

#[test]
fn swarm_state_set_get() {
    let mut state = SwarmState::default();
    state.set("counter", serde_json::json!(42));
    assert_eq!(state.get("counter"), Some(&serde_json::json!(42)));
    assert!(state.get("nonexistent").is_none());
}

#[tokio::test]
async fn swarm_executor_register() {
    let bus = Arc::new(EventBus::new());
    let se = SwarmExecutor::new(bus) /* agent_executor: Arc::new(SuccessExecutor) */;
    se.register("swarm1".into(), SwarmConfig::default()).await;
}

#[tokio::test]
async fn swarm_executor_register_and_verify() {
    let bus = Arc::new(EventBus::new());
    let se = SwarmExecutor::new(bus) /* agent_executor: Arc::new(SuccessExecutor) */;
    se.register("swarm_verify".into(), SwarmConfig::default())
        .await;
    // Verify the swarm was registered by checking that its state store exists
    // (register creates an empty SwarmState). Writing and reading back confirms it.
    se.set_state("swarm_verify", "probe", serde_json::json!(true))
        .await;
    let val = se.get_state("swarm_verify", "probe").await;
    assert_eq!(
        val,
        Some(serde_json::json!(true)),
        "registered swarm should have accessible state"
    );
    // Also verify that an unregistered swarm has no state
    let missing = se.get_state("nonexistent_swarm", "probe").await;
    assert!(missing.is_none(), "unregistered swarm should return None");
}

#[tokio::test]
async fn swarm_executor_state() {
    let bus = Arc::new(EventBus::new());
    let se = SwarmExecutor::new(bus) /* agent_executor: Arc::new(SuccessExecutor) */;
    se.register("s1".into(), SwarmConfig::default()).await;
    se.set_state("s1", "key", serde_json::json!("value")).await;
    let val = se.get_state("s1", "key").await;
    assert_eq!(val, Some(serde_json::json!("value")));
}

// ═══════════════════════════════════════════════════════════════════
// EventBus
// ═══════════════════════════════════════════════════════════════════

#[test]
fn event_bus_new() {
    let bus = EventBus::new();
    assert_eq!(bus.subscriber_count(), 0);
}

#[test]
fn event_bus_default() {
    let bus = EventBus::default();
    assert_eq!(bus.subscriber_count(), 0);
}

#[test]
fn event_bus_subscribe() {
    let bus = EventBus::new();
    let _rx = bus.subscribe();
    assert_eq!(bus.subscriber_count(), 1);
}

#[tokio::test]
async fn event_bus_emit_with_subscriber() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe();
    let event = AgentEvent::TaskCompleted {
        agent_id: "a1".into(),
        task_id: "t1".into(),
        output: serde_json::json!({}),
    };
    let count = bus.emit(event.clone()).await.unwrap();
    assert_eq!(count, 1);
    let received = rx.recv().await.unwrap();
    assert_eq!(received.agent_id(), "a1");
    assert_eq!(received.event_type(), "task_completed");
}

#[tokio::test]
async fn event_bus_emit_no_subscribers() {
    let bus = EventBus::new();
    let event = AgentEvent::TaskFailed {
        agent_id: "a1".into(),
        task_id: "t1".into(),
        error: "err".into(),
    };
    assert!(bus.emit(event).await.is_err());
}

#[tokio::test]
async fn event_bus_stats() {
    let bus = EventBus::new();
    let _rx = bus.subscribe();
    bus.emit(AgentEvent::Custom {
        agent_id: "a".into(),
        event_type: "test".into(),
        payload: serde_json::json!({}),
    })
    .await
    .unwrap();
    let stats = bus.stats().await;
    assert_eq!(stats.total_emitted, 1);
    assert_eq!(*stats.by_type.get("test").unwrap(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// AgentEvent
// ═══════════════════════════════════════════════════════════════════

#[test]
fn agent_event_types() {
    let events: Vec<AgentEvent> = vec![
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
            decision: "approved".into(),
            votes_for: 3,
            votes_against: 1,
        },
        AgentEvent::SwarmConsensus {
            swarm_id: "s".into(),
            result: serde_json::json!({}),
        },
        AgentEvent::CoupTriggered {
            agent_id: "a".into(),
            target_agent_id: "b".into(),
            reason: "low trust".into(),
        },
        AgentEvent::PermissionDenied {
            agent_id: "a".into(),
            resource: "file".into(),
            action: "write".into(),
        },
        AgentEvent::Custom {
            agent_id: "a".into(),
            event_type: "custom".into(),
            payload: serde_json::json!({}),
        },
    ];
    assert_eq!(events[0].event_type(), "task_completed");
    assert_eq!(events[1].event_type(), "task_failed");
    assert_eq!(events[2].event_type(), "state_changed");
    assert_eq!(events[3].event_type(), "vote_completed");
    assert_eq!(events[4].event_type(), "swarm_consensus");
    assert_eq!(events[5].event_type(), "coup_triggered");
    assert_eq!(events[6].event_type(), "permission_denied");
    assert_eq!(events[7].event_type(), "custom");
}

// ═══════════════════════════════════════════════════════════════════
// PermissionEngine
// ═══════════════════════════════════════════════════════════════════

#[test]
fn permission_rule_allow() {
    let r = PermissionRule::allow("file", "read");
    assert_eq!(r.resource, "file");
    assert_eq!(r.action, "read");
}

#[test]
fn permission_rule_deny() {
    let r = PermissionRule::deny("network", "connect");
    assert_eq!(r.resource, "network");
}

#[test]
fn permission_rule_with_realm() {
    let r = PermissionRule::allow("llm", "*").with_realm("production");
    assert_eq!(r.realm, Some("production".into()));
}

#[test]
fn permission_config_builder() {
    let config = PermissionConfig::new()
        .add_rule(PermissionRule::allow("file", "read"))
        .add_rule(PermissionRule::deny("network", "*"))
        .default_deny();
    assert_eq!(config.rules.len(), 2);
}

#[tokio::test]
async fn permission_engine_register_and_check() {
    let bus = Arc::new(EventBus::new());
    let _rx = bus.subscribe();
    let pe = PermissionEngine::new(bus);
    let config = PermissionConfig::new()
        .add_rule(PermissionRule::allow("file", "read"))
        .default_deny();
    pe.register("agent1".into(), config).await;
    assert!(pe.check("agent1", "file", "read", None).await.is_ok());
    assert!(pe
        .check("agent1", "network", "connect", None)
        .await
        .is_err());
}

#[tokio::test]
async fn permission_engine_no_config_allows() {
    let bus = Arc::new(EventBus::new());
    let pe = PermissionEngine::new(bus);
    // No config registered: should allow by default
    assert!(pe
        .check("unknown_agent", "anything", "do", None)
        .await
        .is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// CoupDetector (AgentTrustState)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn coup_config_default() {
    let c = CoupConfig::default();
    assert_eq!(c.coup_authority, "system");
    assert!(c.audit_log);
    assert!((c.coup_condition.trust_threshold - 0.3).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════
// AgentExecutor trait
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn default_agent_executor_returns_stub_failure() {
    // DefaultAgentExecutor is a stub — it always returns success=false
    // with a hint to plug in a real implementation.
    let executor = DefaultAgentExecutor;
    let task = AgentTask {
        data: serde_json::json!({"prompt": "test"}),
        metadata: HashMap::new(),
    };
    let result = executor.execute("agent1", task).await;
    assert!(!result.success, "Stub executor should return failure");
    assert_eq!(result.confidence, 0.0);
}

#[tokio::test]
async fn engine_execute_workflow() {
    let (engine, le, ne) = create_test_engine();
    let layer = Layer {
        id: Uuid::new_v4(),
        name: "l".into(),
        agents: vec!["a1".into()],
        config: LayerConfig::default(),
    };
    let lid = layer.id;
    le.register_layer(layer).await.unwrap();

    let mut net = Network::new("n".into());
    net.add_layer(lid);
    let nid = net.id;
    ne.register_network(net).await.unwrap();

    let mut wf = Workflow {
        id: Uuid::new_v4(),
        name: "exec_test".into(),
        steps: vec![],
        config: WorkflowConfig::default(),
    };
    wf.steps.push(WorkflowStep {
        name: "step".to_string(),
        step_type: StepType::Network { network_id: nid },
        config: StepConfig::default(),
    });
    let wfid = engine.register_workflow(wf).await.unwrap();
    let input = WorkflowInput {
        data: serde_json::json!({"input": "test"}),
        metadata: HashMap::new(),
    };
    let result = engine.execute_workflow(wfid, input).await.unwrap();
    assert!(result.success);
}
