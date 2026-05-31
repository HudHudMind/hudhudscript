//! Perspective definition and field-access types.

use crate::agent::StateValue;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

/// The access mode for a single field within a perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldAccess {
    /// The field is not visible to this perspective.
    Hidden,
    /// The field is visible but the perspective cannot modify it.
    ReadOnly,
    /// The field is visible and the perspective can modify it.
    ReadWrite,
}

impl fmt::Display for FieldAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldAccess::Hidden => write!(f, "hidden"),
            FieldAccess::ReadOnly => write!(f, "ro"),
            FieldAccess::ReadWrite => write!(f, "rw"),
        }
    }
}

/// A projection function transforms a `StateValue` before it is returned to a
/// perspective holder.
///
/// This enables:
/// - **Redaction**: returning `StateValue::Null` for sensitive fields.
/// - **Masking**: returning `"***"` for a password field.
/// - **Transformation**: returning the logarithm of a numeric field.
///
/// Projections are pure functions (no side-effects).
pub type ProjectionFn = Arc<dyn Fn(&StateValue) -> StateValue + Send + Sync + 'static>;

/// A named perspective that defines what an agent can see and modify.
///
/// A perspective is a *compile-time* description of a view; it is stateless
/// itself.  Runtime state is held by `AgentState` and accessed through a
/// `PerspectiveHolder`.
pub struct Perspective {
    /// Unique name for this perspective (e.g., `"accountant"`, `"auditor"`).
    pub name: String,
    /// Description used in logs and diagnostics.
    pub description: Option<String>,
    /// Per-field access rules.  Fields not listed default to `Hidden`.
    pub field_access: HashMap<String, FieldAccess>,
    /// Per-field projection functions applied on read.
    pub projections: HashMap<String, ProjectionFn>,
}

impl fmt::Debug for Perspective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Perspective")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("field_access", &self.field_access)
            .finish()
    }
}

impl Perspective {
    /// Create a new perspective with no field access by default.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            field_access: HashMap::new(),
            projections: HashMap::new(),
        }
    }

    /// Add a description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Grant read-only access to a field.
    pub fn readable(mut self, field: impl Into<String>) -> Self {
        self.field_access
            .insert(field.into(), FieldAccess::ReadOnly);
        self
    }

    /// Grant read-write access to a field.
    pub fn writable(mut self, field: impl Into<String>) -> Self {
        self.field_access
            .insert(field.into(), FieldAccess::ReadWrite);
        self
    }

    /// Attach a projection function to a field.
    pub fn with_projection(
        mut self,
        field: impl Into<String>,
        projection: impl Fn(&StateValue) -> StateValue + Send + Sync + 'static,
    ) -> Self {
        self.projections.insert(field.into(), Arc::new(projection));
        self
    }

    /// Returns the access mode for `field`.
    pub fn access_for(&self, field: &str) -> FieldAccess {
        self.field_access
            .get(field)
            .copied()
            .unwrap_or(FieldAccess::Hidden)
    }

    /// Returns the set of all fields that are visible (not hidden).
    pub fn visible_fields(&self) -> HashSet<&str> {
        self.field_access
            .iter()
            .filter(|(_, &mode)| mode != FieldAccess::Hidden)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Apply the projection for `field` to `value`, or return the value
    /// unchanged if no projection is registered.
    pub fn project(&self, field: &str, value: &StateValue) -> StateValue {
        if let Some(proj) = self.projections.get(field) {
            proj(value)
        } else {
            value.clone()
        }
    }
}
