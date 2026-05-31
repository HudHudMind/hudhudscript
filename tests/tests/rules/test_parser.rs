//! External tests for hudhudscript_rules::parser

use hudhudscript_rules::parser::{SkillParseError, SkillParser};
use hudhudscript_rules::skill::SkillTrigger;

const VALID_YAML: &str = r#"
name: auto-backup
description: Backs up changed files
triggers:
  - event: "file.changed"
    pattern: "/home/*/documents/*"
conditions:
  - "file.size > 0"
actions:
  - tool: backup
    args:
      source: "{{file.path}}"
"#;

#[test]
fn test_parse_valid_skill() {
    let skill = SkillParser::parse(VALID_YAML).expect("should parse");
    assert_eq!(skill.name, "auto-backup");
    assert_eq!(skill.description, "Backs up changed files");
    assert_eq!(skill.triggers.len(), 1);
    assert_eq!(skill.conditions, vec!["file.size > 0"]);
    assert_eq!(skill.actions.len(), 1);
    assert_eq!(skill.actions[0].tool, "backup");
    assert_eq!(
        skill.actions[0].args.get("source").unwrap(),
        "{{file.path}}"
    );
}

#[test]
fn test_parse_minimal_skill() {
    let yaml = r#"
name: ping
triggers:
  - event: "health.check"
actions:
  - tool: echo
    args:
      msg: pong
"#;
    let skill = SkillParser::parse(yaml).expect("should parse");
    assert_eq!(skill.name, "ping");
    assert!(skill.conditions.is_empty());
    assert!(skill.description.is_empty());
}

#[test]
fn test_parse_cron_trigger() {
    let yaml = r#"
name: scheduled
triggers:
  - cron: "0 * * * *"
actions:
  - tool: cleanup
"#;
    let skill = SkillParser::parse(yaml).expect("should parse");
    if let SkillTrigger::Cron { cron } = &skill.triggers[0] {
        assert_eq!(cron, "0 * * * *");
    } else {
        panic!("expected Cron trigger");
    }
}

#[test]
fn test_parse_manual_trigger() {
    let yaml = r#"
name: deploy
triggers:
  - manual: true
actions:
  - tool: deploy
"#;
    let skill = SkillParser::parse(yaml).expect("should parse");
    assert!(matches!(
        skill.triggers[0],
        SkillTrigger::Manual { manual: true }
    ));
}

#[test]
fn test_validation_empty_name() {
    let yaml = r#"
name: ""
triggers:
  - event: "x"
actions:
  - tool: y
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(matches!(err, SkillParseError::ValidationError(_)));
}

#[test]
fn test_validation_no_triggers() {
    let yaml = r#"
name: bad
triggers: []
actions:
  - tool: y
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(matches!(err, SkillParseError::ValidationError(_)));
}

#[test]
fn test_validation_no_actions() {
    let yaml = r#"
name: bad
triggers:
  - event: "x"
actions: []
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(matches!(err, SkillParseError::ValidationError(_)));
}

#[test]
fn test_validation_empty_tool() {
    let yaml = r#"
name: bad
triggers:
  - event: "x"
actions:
  - tool: ""
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(matches!(err, SkillParseError::ValidationError(_)));
}

#[test]
fn test_invalid_yaml() {
    let err = SkillParser::parse("{{{{invalid yaml").unwrap_err();
    assert!(matches!(err, SkillParseError::YamlError(_)));
}

#[test]
fn test_parse_many() {
    let yaml = r#"
- name: a
  triggers:
    - event: "x"
  actions:
    - tool: t1
- name: b
  triggers:
    - event: "y"
  actions:
    - tool: t2
"#;
    let skills = SkillParser::parse_many(yaml).expect("should parse");
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0].name, "a");
    assert_eq!(skills[1].name, "b");
}

#[test]
fn test_parse_with_timeout() {
    let yaml = r#"
name: slow
triggers:
  - event: "build.started"
actions:
  - tool: build
    timeout: 300
"#;
    let skill = SkillParser::parse(yaml).expect("should parse");
    assert_eq!(skill.actions[0].timeout, Some(300));
}

#[test]
fn test_validation_empty_event_trigger() {
    let yaml = r#"
name: bad-trigger
triggers:
  - event: ""
actions:
  - tool: y
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(
        matches!(err, SkillParseError::ValidationError(ref msg) if msg.contains("non-empty event name"))
    );
}

#[test]
fn test_validation_empty_cron_trigger() {
    let yaml = r#"
name: bad-cron
triggers:
  - cron: ""
actions:
  - tool: y
"#;
    let err = SkillParser::parse(yaml).unwrap_err();
    assert!(
        matches!(err, SkillParseError::ValidationError(ref msg) if msg.contains("non-empty cron expression"))
    );
}

#[test]
fn test_skill_parse_error_display_yaml() {
    let err = SkillParseError::YamlError("bad yaml".to_string());
    let msg = format!("{}", err);
    assert_eq!(msg, "YAML parse error: bad yaml");
}

#[test]
fn test_skill_parse_error_display_validation() {
    let err = SkillParseError::ValidationError("empty name".to_string());
    let msg = format!("{}", err);
    assert_eq!(msg, "Validation error: empty name");
}

#[test]
fn test_skill_parse_error_is_std_error() {
    let err = SkillParseError::YamlError("test".to_string());
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_parse_many_validation_fails() {
    let yaml = r#"
- name: ""
  triggers:
    - event: "x"
  actions:
    - tool: y
"#;
    let err = SkillParser::parse_many(yaml).unwrap_err();
    assert!(matches!(err, SkillParseError::ValidationError(_)));
}
