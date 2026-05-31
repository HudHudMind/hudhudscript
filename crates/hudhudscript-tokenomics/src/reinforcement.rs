//! Reinforcement learning for continuous optimisation
//!
//! Implements tabular Q-learning with epsilon-greedy exploration.

use crate::error::Result;
use crate::types::Reward;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct ActionRecord {
    action: String,
    state: String,
    reward: Option<f64>,
}

/// Reinforcement learning agent using tabular Q-learning.
pub struct ReinforcementAgent {
    alpha: f64,
    gamma: f64,
    epsilon: f64,
    q_table: Arc<RwLock<HashMap<(String, String), f64>>>,
    records: Arc<RwLock<HashMap<Uuid, ActionRecord>>>,
}

impl ReinforcementAgent {
    pub fn new(_enabled: bool) -> Self {
        Self {
            alpha: 0.1,
            gamma: 0.9,
            epsilon: 0.1,
            q_table: Arc::new(RwLock::new(HashMap::new())),
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create an agent with custom hyper-parameters.
    pub fn with_params(_enabled: bool, alpha: f64, gamma: f64, epsilon: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.001, 1.0),
            gamma: gamma.clamp(0.0, 0.999),
            epsilon: epsilon.clamp(0.0, 1.0),
            q_table: Arc::new(RwLock::new(HashMap::new())),
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn state_from_context(context: &serde_json::Value) -> String {
        match context.as_object() {
            Some(map) => {
                let mut parts: Vec<String> =
                    map.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                parts.sort();
                parts.join(";")
            }
            None => context.to_string(),
        }
    }

    /// Record action taken
    pub async fn record_action(&self, action: &str, context: serde_json::Value) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let state = Self::state_from_context(&context);
        let record = ActionRecord {
            action: action.to_string(),
            state,
            reward: None,
        };
        self.records.write().await.insert(id, record);
        Ok(id)
    }

    /// Submit reward for action, triggering an immediate Q-value update.
    pub async fn submit_reward(&self, reward: Reward) -> Result<()> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(&reward.action_id) {
            record.reward = Some(reward.reward_value);

            let key = (record.state.clone(), record.action.clone());
            let mut q = self.q_table.write().await;
            let current_q = *q.get(&key).unwrap_or(&0.0);
            let new_q = current_q + self.alpha * (reward.reward_value - current_q);
            q.insert(key, new_q);
        }
        Ok(())
    }

    /// Update policy based on rewards (batch replay).
    pub async fn update_policy(&self) -> Result<()> {
        let records = self.records.read().await;
        let mut q = self.q_table.write().await;

        for record in records.values() {
            if let Some(reward) = record.reward {
                let key = (record.state.clone(), record.action.clone());
                let current_q = *q.get(&key).unwrap_or(&0.0);

                let max_next_q = q
                    .iter()
                    .filter(|((s, _), _)| s == &record.state)
                    .map(|(_, &v)| v)
                    .fold(f64::NEG_INFINITY, f64::max);
                let max_next_q = if max_next_q == f64::NEG_INFINITY {
                    0.0
                } else {
                    max_next_q
                };

                let new_q = current_q + self.alpha * (reward + self.gamma * max_next_q - current_q);
                q.insert(key, new_q);
            }
        }
        Ok(())
    }

    /// Select the best action for a state using epsilon-greedy.
    pub async fn select_action(
        &self,
        context: &serde_json::Value,
        available_actions: &[String],
    ) -> Option<String> {
        if available_actions.is_empty() {
            return None;
        }

        let state = Self::state_from_context(context);

        if rand::random::<f64>() < self.epsilon {
            let idx = rand::random::<usize>() % available_actions.len();
            return Some(available_actions[idx].clone());
        }

        let q = self.q_table.read().await;
        let mut best_action = None;
        let mut best_q = f64::NEG_INFINITY;
        for action in available_actions {
            let key = (state.clone(), action.clone());
            let q_val = *q.get(&key).unwrap_or(&0.0);
            if q_val > best_q {
                best_q = q_val;
                best_action = Some(action.clone());
            }
        }
        best_action
    }
}
