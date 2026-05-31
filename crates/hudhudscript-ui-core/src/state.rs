//! Reactive state management (#546)
//!
//! State containers with change tracking and widget bindings.
//! When state changes, bound widgets receive UIUpdate messages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State scope — where the state lives
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StateScope {
    /// Local to a single screen
    Screen,
    /// Global across all screens
    App,
}

/// A single reactive state value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValue {
    pub name: String,
    pub value: StateData,
    pub scope: StateScope,
    pub dirty: bool,
}

/// State data types (mirrors PropValue but with state semantics)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StateData {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<StateData>),
    Object(HashMap<String, StateData>),
}

/// Binding between a state value and a widget property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub state_name: String,
    pub widget_id: String,
    pub prop_name: String,
}

/// State store — manages all state values and bindings for an app
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateStore {
    values: HashMap<String, StateValue>,
    bindings: Vec<Binding>,
}

impl StateStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a new state value
    pub fn define(&mut self, name: String, value: StateData, scope: StateScope) {
        self.values.insert(
            name.clone(),
            StateValue {
                name,
                value,
                scope,
                dirty: false,
            },
        );
    }

    /// Get a state value
    pub fn get(&self, name: &str) -> Option<&StateData> {
        self.values.get(name).map(|v| &v.value)
    }

    /// Set a state value; marks it dirty and returns affected widget IDs
    pub fn set(&mut self, name: &str, value: StateData) -> Vec<String> {
        if let Some(state) = self.values.get_mut(name) {
            if state.value != value {
                state.value = value;
                state.dirty = true;
                return self.affected_widgets(name);
            }
        }
        vec![]
    }

    /// Bind a state value to a widget property
    pub fn bind(&mut self, state_name: String, widget_id: String, prop_name: String) {
        self.bindings.push(Binding {
            state_name,
            widget_id,
            prop_name,
        });
    }

    /// Get all widget IDs affected by a state change
    fn affected_widgets(&self, state_name: &str) -> Vec<String> {
        self.bindings
            .iter()
            .filter(|b| b.state_name == state_name)
            .map(|b| b.widget_id.clone())
            .collect()
    }

    /// Get all dirty state names and clear dirty flags
    pub fn drain_dirty(&mut self) -> Vec<String> {
        let mut dirty = vec![];
        for (name, state) in &mut self.values {
            if state.dirty {
                dirty.push(name.clone());
                state.dirty = false;
            }
        }
        dirty
    }

    /// Get all bindings for a given state name
    pub fn bindings_for(&self, state_name: &str) -> Vec<&Binding> {
        self.bindings
            .iter()
            .filter(|b| b.state_name == state_name)
            .collect()
    }

    /// Remove all screen-scoped state (when navigating away)
    pub fn clear_screen_state(&mut self) {
        self.values.retain(|_, v| v.scope != StateScope::Screen);
        let screen_states: Vec<String> = self
            .values
            .iter()
            .filter(|(_, v)| v.scope == StateScope::Screen)
            .map(|(k, _)| k.clone())
            .collect();
        self.bindings
            .retain(|b| !screen_states.contains(&b.state_name));
    }
}
