//! Approval Audit Log (Issue #632)
//!
//! Records every approval decision with full context for traceability.
//! Supports in-memory storage with optional JSON serialisation for persistence.

use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::risk::RiskLevel;

// ---------------------------------------------------------------------------
// Audit entry
// ---------------------------------------------------------------------------

/// The decision taken on an approval request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditDecision {
    /// The operation was approved by the user.
    Approved,
    /// The operation was denied by the user.
    Denied,
    /// The operation was auto-approved (e.g. via always-allow).
    AutoApproved,
    /// The operation was auto-approved because its risk level is safe.
    SafeAutoApproved,
}

impl std::fmt::Display for AuditDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditDecision::Approved => write!(f, "approved"),
            AuditDecision::Denied => write!(f, "denied"),
            AuditDecision::AutoApproved => write!(f, "auto-approved"),
            AuditDecision::SafeAutoApproved => write!(f, "safe-auto-approved"),
        }
    }
}

/// A single entry in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this audit entry.
    pub id: String,
    /// The approval request ID this entry relates to.
    pub approval_id: String,
    /// Name of the tool whose execution was evaluated.
    pub tool_name: String,
    /// Arguments that were submitted (for forensic review).
    pub arguments: serde_json::Value,
    /// The risk level that was assessed.
    pub risk_level: RiskLevel,
    /// The decision that was made.
    pub decision: AuditDecision,
    /// Optional reason provided by the user.
    pub reason: Option<String>,
    /// Session identifier to correlate entries within a session.
    pub session_id: String,
    /// Timestamp of the decision.
    pub timestamp: SystemTime,
}

// ---------------------------------------------------------------------------
// Audit log
// ---------------------------------------------------------------------------

/// Thread-safe in-memory audit log with ring-buffer semantics.
#[derive(Clone)]
pub struct AuditLog {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
    /// Maximum entries retained.
    max_entries: usize,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl AuditLog {
    /// Create a new audit log that retains at most `max_entries`.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            max_entries,
        }
    }

    /// Record a new audit entry.
    pub fn record(&self, entry: AuditEntry) {
        info!(
            audit_id = entry.id.as_str(),
            approval_id = entry.approval_id.as_str(),
            tool = entry.tool_name.as_str(),
            risk = %entry.risk_level,
            decision = %entry.decision,
            "Audit log entry recorded"
        );

        let mut entries = self.entries.write().unwrap();
        if entries.len() >= self.max_entries {
            entries.remove(0);
        }
        entries.push(entry);
    }

    /// Create and record an audit entry from its components, returning the entry ID.
    pub fn log_decision(
        &self,
        approval_id: impl Into<String>,
        tool_name: impl Into<String>,
        arguments: serde_json::Value,
        risk_level: RiskLevel,
        decision: AuditDecision,
        reason: Option<String>,
        session_id: impl Into<String>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let entry = AuditEntry {
            id: id.clone(),
            approval_id: approval_id.into(),
            tool_name: tool_name.into(),
            arguments,
            risk_level,
            decision,
            reason,
            session_id: session_id.into(),
            timestamp: SystemTime::now(),
        };
        self.record(entry);
        id
    }

    /// Return all entries (snapshot).
    pub fn entries(&self) -> Vec<AuditEntry> {
        self.entries.read().unwrap().clone()
    }

    /// Return entries for a specific tool.
    pub fn entries_for_tool(&self, tool_name: &str) -> Vec<AuditEntry> {
        self.entries
            .read()
            .unwrap()
            .iter()
            .filter(|e| e.tool_name == tool_name)
            .cloned()
            .collect()
    }

    /// Return entries for a specific session.
    pub fn entries_for_session(&self, session_id: &str) -> Vec<AuditEntry> {
        self.entries
            .read()
            .unwrap()
            .iter()
            .filter(|e| e.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Return the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Returns `true` if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// Clear all entries.
    pub fn clear(&self) {
        self.entries.write().unwrap().clear();
        debug!("Audit log cleared");
    }

    /// Serialise the entire audit log to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let entries = self.entries.read().unwrap();
        serde_json::to_string_pretty(&*entries)
    }

    /// Deserialise audit entries from JSON and append them to the log.
    pub fn load_from_json(&self, json: &str) -> Result<usize, serde_json::Error> {
        let loaded: Vec<AuditEntry> = serde_json::from_str(json)?;
        let count = loaded.len();
        let mut entries = self.entries.write().unwrap();
        for entry in loaded {
            if entries.len() >= self.max_entries {
                entries.remove(0);
            }
            entries.push(entry);
        }
        debug!(count, "Loaded audit entries from JSON");
        Ok(count)
    }
}
