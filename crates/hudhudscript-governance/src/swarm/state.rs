//! Swarm shared state — get, set, remove, clear key-value storage.

use serde_json::Value;

use crate::types::Swarm;

use super::SwarmError;

impl Swarm {
    /// Get a value from shared state
    pub fn get_shared_state(&self, key: &str) -> Option<&Value> {
        self.shared_state.get(key)
    }

    /// Set a value in shared state
    pub fn set_shared_state(&mut self, key: String, value: Value) {
        self.shared_state.insert(key, value);
    }

    /// Remove a value from shared state
    pub fn remove_shared_state(&mut self, key: &str) -> Result<Value, SwarmError> {
        self.shared_state
            .remove(key)
            .ok_or_else(|| SwarmError::StateKeyNotFound(key.to_string()))
    }

    /// Clear all shared state
    pub fn clear_shared_state(&mut self) {
        self.shared_state.clear();
    }
}
