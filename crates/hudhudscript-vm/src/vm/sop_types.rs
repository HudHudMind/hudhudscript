//! SOP (Subject-Oriented Programming) runtime types.
//!
//! Subject templates are compile-time declarations captured at DeclStore time.
//! Subject instances are live runtime copies created by spawn.

use hudhudscript_bytecode::Value16;
use rustc_hash::FxHashMap;

/// Subject template — compile-time declaration data stored at DeclStore.
#[derive(Debug, Clone)]
pub(crate) struct SubjectTemplate {
    pub name: String,
    pub of_subject: Option<String>,
    pub roles: Vec<String>,
    pub state_defaults: FxHashMap<String, Value16>,
    pub capabilities: Vec<String>,
    pub intents: Vec<String>,
}

/// Live subject instance — runtime copy of a template with mutable state.
#[derive(Debug, Clone)]
pub(crate) struct SubjectInstance {
    pub template_name: String,
    pub instance_id: String,
    pub state: FxHashMap<String, Value16>,
    pub actor_id: String,
    /// Harrison & Ossher: view_name → view_state
    pub views: FxHashMap<String, FxHashMap<String, Value16>>,
}

/// Event schema — field name + type hint pairs.
#[derive(Debug, Clone)]
pub(crate) struct EventSchema {
    pub name: String,
    pub fields: Vec<(String, String)>,
}

/// SOP0007: VM-side composition mode
#[derive(Debug, Clone)]
pub(crate) enum CompositionMode {
    Combine(Vec<String>),
    Override(String),
    Before(String),
    After(String),
}

/// SOP0007: A single composition rule stored in VM
#[derive(Debug, Clone)]
pub(crate) struct CompositionRule {
    pub ability_name: String,
    pub mode: CompositionMode,
}

/// SOP0009: Field correspondence mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldCorrespondence {
    Correspond,
    Separate,
}
