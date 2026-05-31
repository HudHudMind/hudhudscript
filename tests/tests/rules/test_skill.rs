//! External tests for hudhudscript_rules::skill

use hudhudscript_rules::skill::{SkillAction, SkillTrigger};
use std::collections::HashMap;

#[test]
fn test_skill_action_default_args() {
    let action = SkillAction {
        tool: "echo".to_string(),
        args: HashMap::new(),
        timeout: None,
    };
    assert!(action.args.is_empty());
    assert!(action.timeout.is_none());
}

#[test]
fn test_skill_trigger_event_variant() {
    let trigger = SkillTrigger::Event {
        event: "file.changed".to_string(),
        pattern: Some("/home/*/documents/*".to_string()),
    };
    if let SkillTrigger::Event { event, pattern } = &trigger {
        assert_eq!(event, "file.changed");
        assert_eq!(pattern.as_deref(), Some("/home/*/documents/*"));
    } else {
        panic!("expected Event variant");
    }
}

#[test]
fn test_skill_trigger_cron_variant() {
    let trigger = SkillTrigger::Cron {
        cron: "0 * * * *".to_string(),
    };
    if let SkillTrigger::Cron { cron } = &trigger {
        assert_eq!(cron, "0 * * * *");
    } else {
        panic!("expected Cron variant");
    }
}

#[test]
fn test_skill_trigger_manual_variant() {
    let trigger = SkillTrigger::Manual { manual: true };
    if let SkillTrigger::Manual { manual } = &trigger {
        assert!(*manual);
    } else {
        panic!("expected Manual variant");
    }
}
