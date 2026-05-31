//! PerspectiveHolder binds agents to shared state through perspectives.

use crate::agent::{AgentId, AgentState, StateValue};
use crate::perspective::definition::{FieldAccess, Perspective};
use crate::perspective::error::PerspectiveError;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Binds an agent to a set of perspectives on a shared `AgentState`.
///
/// All reads and writes go through the perspective layer, enforcing the SOP
/// subject-view contract.
pub struct PerspectiveHolder {
    /// The agent that owns this holder.
    pub agent_id: AgentId,
    /// The shared state being observed through perspectives.
    state: Arc<RwLock<AgentState>>,
    /// Active perspectives, by name.
    perspectives: Vec<Arc<Perspective>>,
}

impl fmt::Debug for PerspectiveHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PerspectiveHolder")
            .field("agent_id", &self.agent_id)
            .field(
                "perspectives",
                &self
                    .perspectives
                    .iter()
                    .map(|p| &p.name)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl PerspectiveHolder {
    /// Create a holder with no active perspectives.
    pub fn new(agent_id: impl Into<String>, state: Arc<RwLock<AgentState>>) -> Self {
        Self {
            agent_id: agent_id.into(),
            state,
            perspectives: Vec::new(),
        }
    }

    /// Add a perspective to this holder.
    pub fn add_perspective(&mut self, perspective: Arc<Perspective>) {
        self.perspectives.push(perspective);
    }

    /// Returns the *effective* access mode for `field` across all active
    /// perspectives.
    ///
    /// The most permissive mode wins (Hidden < ReadOnly < ReadWrite), modelling
    /// SOP's principle that a subject's view is the union of its perspectives.
    pub fn effective_access(&self, field: &str) -> FieldAccess {
        let mut best = FieldAccess::Hidden;
        for p in &self.perspectives {
            match p.access_for(field) {
                FieldAccess::ReadWrite => return FieldAccess::ReadWrite, // can't improve
                FieldAccess::ReadOnly if best == FieldAccess::Hidden => {
                    best = FieldAccess::ReadOnly;
                }
                _ => {}
            }
        }
        best
    }

    /// Read a field value through the perspective lens.
    ///
    /// Returns `None` if the field is hidden or does not exist in the state.
    pub async fn read(&self, field: &str) -> Option<StateValue> {
        if self.effective_access(field) == FieldAccess::Hidden {
            return None;
        }

        let state = self.state.read().await;
        let raw = state.get(field)?.clone();

        // Apply the first perspective that has a projection for this field.
        for p in &self.perspectives {
            if p.projections.contains_key(field) {
                return Some(p.project(field, &raw));
            }
        }
        Some(raw)
    }

    /// Write a field value through the perspective lens.
    ///
    /// Returns an error if no active perspective grants write access to `field`.
    pub async fn write(
        &self,
        field: impl Into<String>,
        value: StateValue,
    ) -> Result<(), PerspectiveError> {
        let field = field.into();
        if self.effective_access(&field) != FieldAccess::ReadWrite {
            return Err(PerspectiveError::WriteAccessDenied {
                agent: self.agent_id.clone(),
                field,
            });
        }

        let mut state = self.state.write().await;
        state.set(field, value);
        Ok(())
    }

    /// Return a filtered snapshot of the state containing only visible fields,
    /// with projections applied.
    pub async fn snapshot(&self) -> HashMap<String, StateValue> {
        let state = self.state.read().await;
        let mut result = HashMap::new();

        for (field, value) in &state.variables {
            if self.effective_access(field) != FieldAccess::Hidden {
                // Apply projection if any.
                let projected = {
                    let mut projected_value = value.clone();
                    for p in &self.perspectives {
                        if p.projections.contains_key(field.as_str()) {
                            projected_value = p.project(field, value);
                            break;
                        }
                    }
                    projected_value
                };
                result.insert(field.clone(), projected);
            }
        }

        result
    }

    /// List all field names visible to this holder (union of all perspectives).
    pub async fn visible_fields(&self) -> std::collections::HashSet<String> {
        let state = self.state.read().await;
        state
            .variables
            .keys()
            .filter(|f| self.effective_access(f) != FieldAccess::Hidden)
            .cloned()
            .collect()
    }
}
