//! External tests for hudhudscript_rules::engine

use hudhudscript_rules::action::NoopExecutor;
use hudhudscript_rules::engine::SkillEngine;
use hudhudscript_rules::trigger::BusEvent;

const BACKUP_SKILL_YAML: &str = r#"
name: auto-backup
triggers:
  - event: "file.changed"
    pattern: "/home/*/documents/*"
conditions:
  - "file.size > 0"
actions:
  - tool: backup
    args:
      source: "{{event.payload}}"
"#;

#[test]
fn test_engine_load_yaml() {
    let mut engine = SkillEngine::new();
    engine.load_yaml(BACKUP_SKILL_YAML).expect("should load");
    assert_eq!(engine.len(), 1);
    assert!(engine.get("auto-backup").is_some());
}

#[test]
fn test_engine_load_yaml_many() {
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
    let mut engine = SkillEngine::new();
    let count = engine.load_yaml_many(yaml).expect("should load");
    assert_eq!(count, 2);
    assert_eq!(engine.len(), 2);
}

#[test]
fn test_engine_register_and_unregister() {
    let mut engine = SkillEngine::new();
    engine.load_yaml(BACKUP_SKILL_YAML).unwrap();
    assert!(engine.get("auto-backup").is_some());

    let removed = engine.unregister("auto-backup");
    assert!(removed.is_some());
    assert!(engine.is_empty());
}

#[test]
fn test_engine_skill_names() {
    let mut engine = SkillEngine::new();
    engine.load_yaml(BACKUP_SKILL_YAML).unwrap();

    let names = engine.skill_names();
    assert!(names.contains(&"auto-backup"));
}

#[test]
fn test_engine_process_event_matching() {
    let mut engine = SkillEngine::new();
    engine.load_yaml(BACKUP_SKILL_YAML).unwrap();

    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: Some("/home/alice/documents/report.txt".to_string()),
    };

    let executor = NoopExecutor;
    let reports = engine.process_event(&event, &executor);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].skill_name, "auto-backup");
    assert_eq!(reports[0].action_results.len(), 1);
    assert_eq!(reports[0].action_results[0].tool_name, "backup");
}

#[test]
fn test_engine_process_event_no_match() {
    let mut engine = SkillEngine::new();
    engine.load_yaml(BACKUP_SKILL_YAML).unwrap();

    let event = BusEvent {
        event_type: "file.deleted".to_string(),
        payload: Some("/home/alice/documents/report.txt".to_string()),
    };

    let executor = NoopExecutor;
    let reports = engine.process_event(&event, &executor);
    assert!(reports.is_empty());
}

#[test]
fn test_engine_process_event_pattern_no_match() {
    let mut engine = SkillEngine::new();
    engine.load_yaml(BACKUP_SKILL_YAML).unwrap();

    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: Some("/var/log/syslog".to_string()),
    };

    let executor = NoopExecutor;
    let reports = engine.process_event(&event, &executor);
    assert!(reports.is_empty());
}

#[test]
fn test_engine_process_event_output_piping() {
    let yaml = r#"
name: pipe-test
triggers:
  - event: "build"
actions:
  - tool: step1
    args:
      out: "hello"
  - tool: step2
    args:
      input: "{{out}}"
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();

    let event = BusEvent {
        event_type: "build".to_string(),
        payload: None,
    };

    let executor = NoopExecutor;
    let reports = engine.process_event(&event, &executor);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].action_results.len(), 2);
    assert_eq!(reports[0].action_results[1].tool_name, "step2");
}

#[test]
fn test_engine_default() {
    let engine = SkillEngine::default();
    assert!(engine.is_empty());
}

#[test]
fn test_engine_process_event_no_payload() {
    let yaml = r#"
name: no-pattern
triggers:
  - event: "ping"
actions:
  - tool: pong
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml).unwrap();

    let event = BusEvent {
        event_type: "ping".to_string(),
        payload: None,
    };

    let executor = NoopExecutor;
    let reports = engine.process_event(&event, &executor);

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].skill_name, "no-pattern");
}

#[test]
fn test_engine_unregister_nonexistent() {
    let mut engine = SkillEngine::new();
    let removed = engine.unregister("nope");
    assert!(removed.is_none());
}

#[test]
fn test_engine_overwrite_skill() {
    let yaml1 = r#"
name: dup
triggers:
  - event: "a"
actions:
  - tool: t1
"#;
    let yaml2 = r#"
name: dup
triggers:
  - event: "b"
actions:
  - tool: t2
"#;
    let mut engine = SkillEngine::new();
    engine.load_yaml(yaml1).unwrap();
    engine.load_yaml(yaml2).unwrap();
    assert_eq!(engine.len(), 1);
    let skill = engine.get("dup").unwrap();
    assert_eq!(skill.actions[0].tool, "t2");
}
