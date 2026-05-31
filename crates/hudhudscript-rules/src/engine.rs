//! Skill engine — loads skills, registers triggers, processes events
//!
//! The `SkillEngine` is the central coordinator: it holds a registry of
//! parsed skills, evaluates incoming bus events against registered triggers,
//! and dispatches matching skill action chains through an `ActionExecutor`.

use crate::action::{ActionChain, ActionExecutor, ActionResult};
use crate::parser::{SkillParseError, SkillParser};
use crate::skill::Skill;
use crate::trigger::{BusEvent, TriggerEvaluator};
use std::collections::HashMap;

/// Central skill engine that manages skill lifecycle and event processing
pub struct SkillEngine {
    /// Registered skills keyed by name
    skills: HashMap<String, Skill>,
}

/// Result of processing a single event through the engine
#[derive(Debug)]
pub struct SkillExecutionReport {
    /// Name of the skill that was triggered
    pub skill_name: String,
    /// Results of each action in the skill
    pub action_results: Vec<ActionResult>,
}

impl SkillEngine {
    /// Create an empty engine.
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Load and register a skill from a YAML string.
    pub fn load_yaml(&mut self, yaml: &str) -> Result<(), SkillParseError> {
        let skill = SkillParser::parse(yaml)?;
        self.register(skill);
        Ok(())
    }

    /// Load multiple skills from a YAML sequence string.
    pub fn load_yaml_many(&mut self, yaml: &str) -> Result<usize, SkillParseError> {
        let skills = SkillParser::parse_many(yaml)?;
        let count = skills.len();
        for skill in skills {
            self.register(skill);
        }
        Ok(count)
    }

    /// Register a pre-parsed skill. Overwrites any existing skill with the same name.
    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Unregister a skill by name. Returns the removed skill if it existed.
    pub fn unregister(&mut self, name: &str) -> Option<Skill> {
        self.skills.remove(name)
    }

    /// Get a registered skill by name.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// List all registered skill names.
    pub fn skill_names(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Number of registered skills.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Whether the engine has no registered skills.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Process a bus event: find all matching skills, execute their action chains,
    /// and return a report for each.
    pub fn process_event(
        &self,
        event: &BusEvent,
        executor: &dyn ActionExecutor,
    ) -> Vec<SkillExecutionReport> {
        let mut reports = Vec::new();

        for skill in self.skills.values() {
            let triggered = skill
                .triggers
                .iter()
                .any(|t| TriggerEvaluator::matches(event, t));

            if !triggered {
                continue;
            }

            // Build initial context from event payload
            let mut ctx = HashMap::new();
            ctx.insert("event.type".to_string(), event.event_type.clone());
            if let Some(ref payload) = event.payload {
                ctx.insert("event.payload".to_string(), payload.clone());
            }

            let chain = ActionChain::new(executor);
            let action_results = chain.run(&skill.actions, &ctx, false);

            reports.push(SkillExecutionReport {
                skill_name: skill.name.clone(),
                action_results,
            });
        }

        reports
    }
}

impl Default for SkillEngine {
    fn default() -> Self {
        Self::new()
    }
}
