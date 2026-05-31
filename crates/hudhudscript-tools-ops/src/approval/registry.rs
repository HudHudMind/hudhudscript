use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tracing::{debug, info};

use super::{ApprovalError, ApprovalId, ApprovalRequest, ApprovalState};

/// Registry of approval requests; thread-safe via `Arc<RwLock<_>>`.
#[derive(Clone)]
pub struct ApprovalRegistry {
    pub(crate) requests: Arc<RwLock<HashMap<ApprovalId, ApprovalRequest>>>,
}

impl Default for ApprovalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalRegistry {
    /// Create a new, empty registry.
    pub fn new() -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submit a new approval request for `tool_name`.
    ///
    /// Returns the generated [`ApprovalId`] so callers can poll or resolve it.
    pub fn submit(&self, tool_name: impl Into<String>, arguments: serde_json::Value) -> ApprovalId {
        let id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now();
        let request = ApprovalRequest {
            id: id.clone(),
            tool_name: tool_name.into(),
            arguments,
            state: ApprovalState::Pending,
            created_at: now,
            updated_at: now,
            reason: None,
        };

        info!(
            approval_id = id.as_str(),
            tool = request.tool_name.as_str(),
            "Submitted HitL approval request"
        );
        self.requests.write().unwrap().insert(id.clone(), request);
        id
    }

    /// Retrieve a snapshot of the request.
    pub fn get(&self, id: &str) -> Option<ApprovalRequest> {
        self.requests.read().unwrap().get(id).cloned()
    }

    /// Return all pending requests.
    pub fn pending(&self) -> Vec<ApprovalRequest> {
        self.requests
            .read()
            .unwrap()
            .values()
            .filter(|r| r.state == ApprovalState::Pending)
            .cloned()
            .collect()
    }

    /// Approve a pending request.
    pub fn approve(&self, id: &str, reason: Option<String>) -> Result<(), ApprovalError> {
        self.transition(id, ApprovalState::Approved, reason)
    }

    /// Deny a pending request.
    pub fn deny(&self, id: &str, reason: Option<String>) -> Result<(), ApprovalError> {
        self.transition(id, ApprovalState::Denied, reason)
    }

    /// Mark an approved request as executed.
    pub fn mark_executed(&self, id: &str) -> Result<(), ApprovalError> {
        self.transition(id, ApprovalState::Executed, None)
    }

    /// Mark a denied request as skipped.
    pub fn mark_skipped(&self, id: &str) -> Result<(), ApprovalError> {
        self.transition(id, ApprovalState::Skipped, None)
    }

    /// Internal helper — validates and applies a state transition.
    fn transition(
        &self,
        id: &str,
        target: ApprovalState,
        reason: Option<String>,
    ) -> Result<(), ApprovalError> {
        let mut requests = self.requests.write().unwrap();
        let req = requests
            .get_mut(id)
            .ok_or_else(|| ApprovalError::NotFound(id.to_string()))?;

        if !is_valid_transition(&req.state, &target) {
            return Err(ApprovalError::InvalidTransition {
                id: id.to_string(),
                from: req.state.clone(),
                to: target,
            });
        }

        debug!(
            approval_id = id,
            tool = req.tool_name.as_str(),
            from = %req.state,
            to = %target,
            "HitL state transition"
        );

        req.state = target;
        req.updated_at = SystemTime::now();
        if reason.is_some() {
            req.reason = reason;
        }
        Ok(())
    }
}

/// Defines the allowed state transitions.
pub fn is_valid_transition(from: &ApprovalState, to: &ApprovalState) -> bool {
    matches!(
        (from, to),
        (ApprovalState::Pending, ApprovalState::Approved)
            | (ApprovalState::Pending, ApprovalState::Denied)
            | (ApprovalState::Approved, ApprovalState::Executed)
            | (ApprovalState::Denied, ApprovalState::Skipped)
    )
}
