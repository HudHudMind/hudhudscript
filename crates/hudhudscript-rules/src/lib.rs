//! # HudHudScript Rules
//!
//! Rule evaluation engine for the Council & Constitution System.
//! Provides conditional logic evaluation and action execution.
//!
//! ## Skill Engine (Issue #634)
//!
//! YAML-based event-action automation system:
//! - `skill` — Skill definition types
//! - `trigger` — Bus event pattern matching
//! - `action` — Action execution and chaining
//! - `parser` — YAML skill parser and validator
//! - `engine` — Central skill engine

pub mod condition;
pub mod context;
pub mod evaluator;
pub mod rule;

// Skill engine modules
pub mod action;
pub mod engine;
pub mod parser;
pub mod skill;
pub mod trigger;

// Re-export condition evaluation
pub use condition::evaluate_condition;

// Re-export context
pub use context::EvaluationContext;

// Re-export rule evaluation (preferred over evaluator)
pub use rule::{
    evaluate_rule, evaluate_rules_with_priority, execute_actions, ActionExecutionResult, RuleResult,
};

// Re-export skill engine types
pub use action::{
    resolve_templates, ActionChain, ActionExecutor, ActionResult, ActionStatus, NoopExecutor,
};
pub use engine::{SkillEngine, SkillExecutionReport};
pub use parser::{SkillParseError, SkillParser};
pub use skill::{Skill, SkillAction, SkillTrigger};
pub use trigger::{glob_match, BusEvent, TriggerEvaluator};
