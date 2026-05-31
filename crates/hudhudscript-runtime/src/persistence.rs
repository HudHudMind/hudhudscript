//! Agent State Persistence
//!
//! Provides save/load functionality for agent state using JSON serialization.
//! Supports file-based storage with optional Redis backend.

use crate::agent::{AgentId, AgentState, StateValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Persistence errors
#[derive(Debug)]
pub enum PersistenceError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    NotFound(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            PersistenceError::Io(e) => write!(f, "IO error: {}", e),
            PersistenceError::Serialization(e) => write!(f, "Serialization error: {}", e),
            PersistenceError::NotFound(id) => write!(f, "State not found for agent: {}", id),
        }
    }
}

impl std::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PersistenceError::Io(e) => Some(e),
            PersistenceError::Serialization(e) => Some(e),
            PersistenceError::NotFound(_) => None,
        }
    }
}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::Io(e)
    }
}

impl From<serde_json::Error> for PersistenceError {
    fn from(e: serde_json::Error) -> Self {
        PersistenceError::Serialization(e)
    }
}

/// Serializable snapshot of agent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub agent_id: AgentId,
    pub variables: HashMap<String, serde_json::Value>,
    pub version: u64,
    pub saved_at: u64, // Unix timestamp (secs)
}

impl StateSnapshot {
    /// Convert AgentState → StateSnapshot
    pub fn from_state(state: &AgentState) -> Self {
        let variables = state
            .variables
            .iter()
            .map(|(k, v)| (k.clone(), state_value_to_json(v)))
            .collect();

        let saved_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            agent_id: state.agent_id.clone(),
            variables,
            version: state.version,
            saved_at,
        }
    }

    /// Convert StateSnapshot → AgentState
    pub fn into_state(self) -> AgentState {
        let variables = self
            .variables
            .into_iter()
            .map(|(k, v)| (k, json_to_state_value(v)))
            .collect();

        AgentState {
            agent_id: self.agent_id,
            variables,
            version: self.version,
            updated_at: std::time::SystemTime::now(),
        }
    }
}

/// File-based state persistence backend
pub struct FilePersistence {
    base_dir: PathBuf,
}

impl FilePersistence {
    /// Create a new file persistence backend
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self, PersistenceError> {
        let base_dir = base_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    /// Save agent state to disk
    pub fn save(&self, state: &AgentState) -> Result<(), PersistenceError> {
        let snapshot = StateSnapshot::from_state(state);
        let path = self.state_path(&state.agent_id);
        let json = serde_json::to_string_pretty(&snapshot)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load agent state from disk
    pub fn load(&self, agent_id: &str) -> Result<AgentState, PersistenceError> {
        let path = self.state_path(agent_id);
        if !path.exists() {
            return Err(PersistenceError::NotFound(agent_id.to_string()));
        }
        let json = std::fs::read_to_string(path)?;
        let snapshot: StateSnapshot = serde_json::from_str(&json)?;
        Ok(snapshot.into_state())
    }

    /// Delete persisted state for an agent
    pub fn delete(&self, agent_id: &str) -> Result<(), PersistenceError> {
        let path = self.state_path(agent_id);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Check if persisted state exists for an agent
    pub fn exists(&self, agent_id: &str) -> bool {
        self.state_path(agent_id).exists()
    }

    fn state_path(&self, agent_id: &str) -> PathBuf {
        // Sanitize agent_id for use as filename
        let safe_id = agent_id.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.base_dir.join(format!("{}.state.json", safe_id))
    }
}

// ── Value conversion helpers ──────────────────────────────────────────────────

fn state_value_to_json(value: &StateValue) -> serde_json::Value {
    match value {
        StateValue::String(s) => serde_json::Value::String(s.clone()),
        StateValue::Number(n) => serde_json::Value::Number(
            serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
        ),
        StateValue::Boolean(b) => serde_json::Value::Bool(*b),
        StateValue::Null => serde_json::Value::Null,
        StateValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(state_value_to_json).collect())
        }
        StateValue::Object(obj) => {
            let map = obj
                .iter()
                .map(|(k, v)| (k.clone(), state_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

fn json_to_state_value(value: serde_json::Value) -> StateValue {
    match value {
        serde_json::Value::String(s) => StateValue::String(s),
        serde_json::Value::Number(n) => StateValue::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::Bool(b) => StateValue::Boolean(b),
        serde_json::Value::Null => StateValue::Null,
        serde_json::Value::Array(arr) => {
            StateValue::Array(arr.into_iter().map(json_to_state_value).collect())
        }
        serde_json::Value::Object(obj) => StateValue::Object(
            obj.into_iter()
                .map(|(k, v)| (k, json_to_state_value(v)))
                .collect(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl PersistenceError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            PersistenceError::Io(..) => hudhudscript_errors::ErrorCode::PersistenceIo,
            PersistenceError::NotFound(..) => hudhudscript_errors::ErrorCode::PersistenceNotFound,
            PersistenceError::Serialization(..) => {
                hudhudscript_errors::ErrorCode::PersistenceSerialization
            }
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<PersistenceError> for hudhudscript_errors::Error {
    fn from(e: PersistenceError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
