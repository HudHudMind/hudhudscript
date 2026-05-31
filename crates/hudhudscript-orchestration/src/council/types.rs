use serde::{Deserialize, Serialize};

/// Execution strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum ExecutionStrategy {
    #[default]
    Parallel,
    Sequential,
    RoundRobin,
    Competitive,
}

/// Voting algorithm
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum VotingAlgorithm {
    /// Majority (>50%)
    #[default]
    Majority,
    /// Unanimous
    Unanimous,
    /// Weighted vote
    Weighted,
    /// First wins
    FirstWins,
}

/// Council member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilMember {
    pub agent_id: String,
    pub role: String,
    /// Weight used in weighted voting (0.0–1.0)
    pub weight: f64,
}

impl CouncilMember {
    pub fn new(agent_id: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            role: role.into(),
            weight: 1.0,
        }
    }
}

/// Member execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResult {
    pub agent_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub vote: Option<bool>,
    pub error: Option<String>,
}

impl MemberResult {
    /// Build a `MemberResult` from an `AgentTaskResult`.
    pub(crate) fn from_agent_result(
        agent_id: String,
        r: crate::agent_executor::AgentTaskResult,
    ) -> Self {
        Self {
            agent_id,
            success: r.success,
            output: r.output,
            vote: r.vote,
            error: r.error,
        }
    }
}

/// Council execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilResult {
    pub council_id: String,
    pub member_results: Vec<MemberResult>,
    pub decision: CouncilDecision,
    pub votes_for: usize,
    pub votes_against: usize,
    pub final_output: serde_json::Value,
}

/// Council decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CouncilDecision {
    Approved,
    Rejected,
    Inconclusive,
}

/// Council configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilConfig {
    pub execution: ExecutionStrategy,
    pub voting: VotingAlgorithm,
    pub timeout_secs: u64,
    /// In competitive mode, how many results to wait for (0 = all)
    pub quorum: usize,
}

impl Default for CouncilConfig {
    fn default() -> Self {
        Self {
            execution: ExecutionStrategy::Parallel,
            voting: VotingAlgorithm::Majority,
            timeout_secs: 60,
            quorum: 0,
        }
    }
}
