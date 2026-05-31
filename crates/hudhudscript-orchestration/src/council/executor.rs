use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agent_executor::{default_agent_executor, AgentExecutor, AgentTask};
use crate::events::{AgentEvent, EventBus};

use super::{
    CouncilConfig, CouncilDecision, CouncilError, CouncilMember, CouncilResult, MemberResult,
    SessionHooks, VotingAlgorithm,
};

/// Council executor
pub struct CouncilExecutor {
    event_bus: Arc<EventBus>,
    /// Registered councils
    councils: Arc<RwLock<HashMap<String, CouncilConfig>>>,
    /// Agent executor used to dispatch work to individual agents
    agent_executor: Arc<dyn AgentExecutor>,
}

impl CouncilExecutor {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            councils: Arc::new(RwLock::new(HashMap::new())),
            agent_executor: default_agent_executor(),
        }
    }

    /// Create a council executor with a custom agent executor.
    pub fn with_agent_executor(
        event_bus: Arc<EventBus>,
        agent_executor: Arc<dyn AgentExecutor>,
    ) -> Self {
        Self {
            event_bus,
            councils: Arc::new(RwLock::new(HashMap::new())),
            agent_executor,
        }
    }

    /// Register a council
    pub async fn register(&self, council_id: String, config: CouncilConfig) {
        self.councils.write().await.insert(council_id, config);
    }

    /// Execute a council
    pub async fn execute(
        &self,
        council_id: &str,
        members: Vec<CouncilMember>,
        task: serde_json::Value,
        session_hooks: Option<SessionHooks>,
    ) -> Result<CouncilResult, CouncilError> {
        let config = {
            let councils = self.councils.read().await;
            councils.get(council_id).cloned().unwrap_or_default()
        };

        // onStart hook
        if let Some(hooks) = &session_hooks {
            if let Some(ref on_start) = hooks.on_start {
                on_start(council_id, &members);
            }
        }

        let timeout_duration = std::time::Duration::from_secs(config.timeout_secs);

        let executor = self.agent_executor.clone();
        let dispatch = async {
            match config.execution {
                super::ExecutionStrategy::Parallel => {
                    Self::execute_parallel(&executor, &members, &task).await
                }
                super::ExecutionStrategy::Sequential => {
                    Self::execute_sequential(&executor, &members, &task).await
                }
                super::ExecutionStrategy::RoundRobin => {
                    Self::execute_round_robin(&executor, &members, &task).await
                }
                super::ExecutionStrategy::Competitive => {
                    Self::execute_competitive(&executor, &members, &task, config.quorum).await
                }
            }
        };

        let member_results = tokio::time::timeout(timeout_duration, dispatch)
            .await
            .map_err(|_| CouncilError::Timeout)?;

        // onVote hook
        if let Some(hooks) = &session_hooks {
            if let Some(ref on_vote) = hooks.on_vote {
                for result in &member_results {
                    on_vote(&result.agent_id, result.vote);
                }
            }
        }

        let (decision, votes_for, votes_against) =
            Self::apply_voting(&member_results, &config.voting, &members);

        let final_output = serde_json::json!({
            "decision": format!("{:?}", decision),
            "votes_for": votes_for,
            "votes_against": votes_against,
            "results": member_results.iter().map(|r| &r.output).collect::<Vec<_>>(),
        });

        let _ = self
            .event_bus
            .emit(AgentEvent::VoteCompleted {
                council_id: council_id.to_string(),
                decision: format!("{:?}", decision),
                votes_for,
                votes_against,
            })
            .await;

        // onComplete hook
        if let Some(hooks) = &session_hooks {
            if let Some(ref on_complete) = hooks.on_complete {
                on_complete(council_id, &decision);
            }
        }

        Ok(CouncilResult {
            council_id: council_id.to_string(),
            member_results,
            decision,
            votes_for,
            votes_against,
            final_output,
        })
    }

    async fn execute_parallel(
        executor: &Arc<dyn AgentExecutor>,
        members: &[CouncilMember],
        task: &serde_json::Value,
    ) -> Vec<MemberResult> {
        let handles: Vec<_> = members
            .iter()
            .map(|m| {
                let agent_id = m.agent_id.clone();
                let task_data = task.clone();
                let exec = executor.clone();
                tokio::spawn(async move {
                    let agent_task = AgentTask {
                        data: task_data,
                        metadata: HashMap::new(),
                    };
                    let result = exec.execute(&agent_id, agent_task).await;
                    MemberResult::from_agent_result(agent_id, result)
                })
            })
            .collect();

        futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect()
    }

    async fn execute_sequential(
        executor: &Arc<dyn AgentExecutor>,
        members: &[CouncilMember],
        task: &serde_json::Value,
    ) -> Vec<MemberResult> {
        let mut results = Vec::new();
        for m in members {
            let agent_task = AgentTask {
                data: task.clone(),
                metadata: HashMap::new(),
            };
            let r = executor.execute(&m.agent_id, agent_task).await;
            results.push(MemberResult::from_agent_result(m.agent_id.clone(), r));
        }
        results
    }

    async fn execute_round_robin(
        executor: &Arc<dyn AgentExecutor>,
        members: &[CouncilMember],
        task: &serde_json::Value,
    ) -> Vec<MemberResult> {
        let mut results = Vec::new();
        let mut current_task = task.clone();
        for m in members {
            let agent_task = AgentTask {
                data: current_task.clone(),
                metadata: HashMap::new(),
            };
            let r = executor.execute(&m.agent_id, agent_task).await;
            current_task = r.output.clone();
            results.push(MemberResult::from_agent_result(m.agent_id.clone(), r));
        }
        results
    }

    async fn execute_competitive(
        executor: &Arc<dyn AgentExecutor>,
        members: &[CouncilMember],
        task: &serde_json::Value,
        quorum: usize,
    ) -> Vec<MemberResult> {
        let handles: Vec<_> = members
            .iter()
            .map(|m| {
                let agent_id = m.agent_id.clone();
                let task_data = task.clone();
                let exec = executor.clone();
                tokio::spawn(async move {
                    let agent_task = AgentTask {
                        data: task_data,
                        metadata: HashMap::new(),
                    };
                    let result = exec.execute(&agent_id, agent_task).await;
                    MemberResult::from_agent_result(agent_id, result)
                })
            })
            .collect();

        let all: Vec<MemberResult> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        if quorum > 0 && quorum < all.len() {
            all.into_iter().take(quorum).collect()
        } else {
            all
        }
    }

    fn apply_voting(
        results: &[MemberResult],
        algorithm: &VotingAlgorithm,
        members: &[CouncilMember],
    ) -> (CouncilDecision, usize, usize) {
        let votes_for = results.iter().filter(|r| r.vote == Some(true)).count();
        let votes_against = results.iter().filter(|r| r.vote == Some(false)).count();
        let total = results.len();

        let decision = match algorithm {
            VotingAlgorithm::Majority => {
                if votes_for * 2 > total {
                    CouncilDecision::Approved
                } else if votes_against * 2 > total {
                    CouncilDecision::Rejected
                } else {
                    CouncilDecision::Inconclusive
                }
            }
            VotingAlgorithm::Unanimous => {
                if votes_for == total {
                    CouncilDecision::Approved
                } else {
                    CouncilDecision::Rejected
                }
            }
            VotingAlgorithm::Weighted => {
                let weight_for: f64 = results
                    .iter()
                    .filter(|r| r.vote == Some(true))
                    .map(|r| {
                        members
                            .iter()
                            .find(|m| m.agent_id == r.agent_id)
                            .map(|m| m.weight)
                            .unwrap_or(1.0)
                    })
                    .sum();
                let total_weight: f64 = members.iter().map(|m| m.weight).sum();
                if weight_for / total_weight > 0.5 {
                    CouncilDecision::Approved
                } else {
                    CouncilDecision::Rejected
                }
            }
            VotingAlgorithm::FirstWins => {
                if let Some(first) = results.first() {
                    if first.vote == Some(true) {
                        CouncilDecision::Approved
                    } else {
                        CouncilDecision::Rejected
                    }
                } else {
                    CouncilDecision::Inconclusive
                }
            }
        };

        (decision, votes_for, votes_against)
    }
}
