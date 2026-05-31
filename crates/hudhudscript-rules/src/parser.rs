//! YAML skill parser and schema validator
//!
//! Parses skill definitions from YAML strings and validates required fields.

use crate::skill::{Skill, SkillAction, SkillTrigger};

/// Errors that can occur during skill parsing or validation
#[derive(Debug, Clone, PartialEq)]
pub enum SkillParseError {
    /// YAML syntax error
    YamlError(String),
    /// Schema validation error
    ValidationError(String),
}

impl std::fmt::Display for SkillParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::YamlError(msg) => write!(f, "YAML parse error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for SkillParseError {}

/// Parses and validates YAML skill definitions.
pub struct SkillParser;

impl SkillParser {
    /// Parse a YAML string into a `Skill`.
    pub fn parse(yaml: &str) -> Result<Skill, SkillParseError> {
        let skill: Skill =
            serde_yaml::from_str(yaml).map_err(|e| SkillParseError::YamlError(e.to_string()))?;
        Self::validate(&skill)?;
        Ok(skill)
    }

    /// Parse multiple skills from a YAML string containing a sequence.
    pub fn parse_many(yaml: &str) -> Result<Vec<Skill>, SkillParseError> {
        let skills: Vec<Skill> =
            serde_yaml::from_str(yaml).map_err(|e| SkillParseError::YamlError(e.to_string()))?;
        for skill in &skills {
            Self::validate(skill)?;
        }
        Ok(skills)
    }

    /// Validate a parsed skill against the expected schema constraints.
    pub fn validate(skill: &Skill) -> Result<(), SkillParseError> {
        if skill.name.is_empty() {
            return Err(SkillParseError::ValidationError(
                "skill name must not be empty".to_string(),
            ));
        }
        if skill.triggers.is_empty() {
            return Err(SkillParseError::ValidationError(format!(
                "skill '{}' must have at least one trigger",
                skill.name
            )));
        }
        if skill.actions.is_empty() {
            return Err(SkillParseError::ValidationError(format!(
                "skill '{}' must have at least one action",
                skill.name
            )));
        }
        for trigger in &skill.triggers {
            Self::validate_trigger(&skill.name, trigger)?;
        }
        for (i, action) in skill.actions.iter().enumerate() {
            Self::validate_action(&skill.name, i, action)?;
        }
        Ok(())
    }

    fn validate_trigger(skill_name: &str, trigger: &SkillTrigger) -> Result<(), SkillParseError> {
        match trigger {
            SkillTrigger::Event { event, .. } => {
                if event.is_empty() {
                    return Err(SkillParseError::ValidationError(format!(
                        "skill '{}': event trigger must have a non-empty event name",
                        skill_name
                    )));
                }
            }
            SkillTrigger::Cron { cron } => {
                if cron.is_empty() {
                    return Err(SkillParseError::ValidationError(format!(
                        "skill '{}': cron trigger must have a non-empty cron expression",
                        skill_name
                    )));
                }
            }
            SkillTrigger::Manual { .. } => {}
        }
        Ok(())
    }

    fn validate_action(
        skill_name: &str,
        index: usize,
        action: &SkillAction,
    ) -> Result<(), SkillParseError> {
        if action.tool.is_empty() {
            return Err(SkillParseError::ValidationError(format!(
                "skill '{}': action[{}] must have a non-empty tool name",
                skill_name, index
            )));
        }
        Ok(())
    }
}
