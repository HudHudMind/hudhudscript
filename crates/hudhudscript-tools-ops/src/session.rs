//! Per-Session Permission Memory (Issue #632)
//!
//! Tracks which tools a user has previously approved or set to "always allow"
//! during the current session, avoiding redundant approval prompts.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

// ---------------------------------------------------------------------------
// Permission status
// ---------------------------------------------------------------------------

/// The remembered permission for a tool within a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionStatus {
    /// The user chose "always allow" for this tool during this session.
    AlwaysAllow,
    /// The user explicitly denied this tool and it should stay blocked.
    AlwaysDeny,
}

impl std::fmt::Display for PermissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionStatus::AlwaysAllow => write!(f, "always-allow"),
            PermissionStatus::AlwaysDeny => write!(f, "always-deny"),
        }
    }
}

// ---------------------------------------------------------------------------
// Permission record
// ---------------------------------------------------------------------------

/// A remembered permission decision for a single tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRecord {
    /// Tool name this permission applies to.
    pub tool_name: String,
    /// The remembered status.
    pub status: PermissionStatus,
    /// When this permission was granted.
    pub granted_at: SystemTime,
    /// How many times this permission has been used to skip prompts.
    pub usage_count: u64,
}

// ---------------------------------------------------------------------------
// Session permissions
// ---------------------------------------------------------------------------

/// Per-session permission memory that tracks "always allow" / "always deny"
/// decisions so users are not repeatedly prompted for the same tool.
#[derive(Clone)]
pub struct SessionPermissions {
    session_id: String,
    permissions: Arc<RwLock<HashMap<String, PermissionRecord>>>,
    /// Tools that have been approved at least once (even without "always allow").
    history: Arc<RwLock<HashSet<String>>>,
}

impl SessionPermissions {
    /// Create a new session permission store.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Return the session identifier.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Set a tool to "always allow" for the remainder of this session.
    pub fn set_always_allow(&self, tool_name: impl Into<String>) {
        let name = tool_name.into();
        info!(
            session = self.session_id.as_str(),
            tool = name.as_str(),
            "Permission set to always-allow"
        );
        let mut perms = self.permissions.write().unwrap();
        perms.insert(
            name.clone(),
            PermissionRecord {
                tool_name: name,
                status: PermissionStatus::AlwaysAllow,
                granted_at: SystemTime::now(),
                usage_count: 0,
            },
        );
    }

    /// Set a tool to "always deny" for the remainder of this session.
    pub fn set_always_deny(&self, tool_name: impl Into<String>) {
        let name = tool_name.into();
        info!(
            session = self.session_id.as_str(),
            tool = name.as_str(),
            "Permission set to always-deny"
        );
        let mut perms = self.permissions.write().unwrap();
        perms.insert(
            name.clone(),
            PermissionRecord {
                tool_name: name,
                status: PermissionStatus::AlwaysDeny,
                granted_at: SystemTime::now(),
                usage_count: 0,
            },
        );
    }

    /// Check whether the tool has a remembered permission.
    ///
    /// If `AlwaysAllow` is found the usage counter is incremented and
    /// `Some(AlwaysAllow)` is returned.  If `AlwaysDeny`, `Some(AlwaysDeny)`
    /// is returned.  If no remembered permission exists, `None` is returned
    /// (the caller should prompt the user).
    pub fn check(&self, tool_name: &str) -> Option<PermissionStatus> {
        let mut perms = self.permissions.write().unwrap();
        if let Some(record) = perms.get_mut(tool_name) {
            record.usage_count += 1;
            debug!(
                session = self.session_id.as_str(),
                tool = tool_name,
                status = %record.status,
                usage_count = record.usage_count,
                "Permission cache hit"
            );
            Some(record.status.clone())
        } else {
            None
        }
    }

    /// Record that a tool was approved once (without "always allow").
    ///
    /// This is tracked in the history set for informational purposes.
    pub fn record_one_time_approval(&self, tool_name: impl Into<String>) {
        self.history.write().unwrap().insert(tool_name.into());
    }

    /// Returns `true` if the tool has been approved at least once this session
    /// (either via one-time approval or always-allow).
    pub fn was_approved_before(&self, tool_name: &str) -> bool {
        if let Some(record) = self.permissions.read().unwrap().get(tool_name) {
            if record.status == PermissionStatus::AlwaysAllow {
                return true;
            }
        }
        self.history.read().unwrap().contains(tool_name)
    }

    /// Remove a remembered permission, returning the caller to the interactive prompt.
    pub fn revoke(&self, tool_name: &str) -> bool {
        let removed = self
            .permissions
            .write()
            .unwrap()
            .remove(tool_name)
            .is_some();
        if removed {
            info!(
                session = self.session_id.as_str(),
                tool = tool_name,
                "Permission revoked"
            );
        }
        removed
    }

    /// List all current permissions.
    pub fn all_permissions(&self) -> Vec<PermissionRecord> {
        self.permissions.read().unwrap().values().cloned().collect()
    }

    /// Return the set of tools that have been approved at least once.
    pub fn approval_history(&self) -> Vec<String> {
        self.history.read().unwrap().iter().cloned().collect()
    }

    /// Clear all remembered permissions and history.
    pub fn clear(&self) {
        self.permissions.write().unwrap().clear();
        self.history.write().unwrap().clear();
        debug!(
            session = self.session_id.as_str(),
            "Session permissions cleared"
        );
    }
}
