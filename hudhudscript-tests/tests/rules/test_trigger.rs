//! External tests for hudhudscript_rules::trigger

use hudhudscript_rules::glob_match;
use hudhudscript_rules::skill::SkillTrigger;
use hudhudscript_rules::trigger::{BusEvent, TriggerEvaluator};

#[test]
fn test_exact_event_match() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: None,
    };
    let trigger = SkillTrigger::Event {
        event: "file.changed".to_string(),
        pattern: None,
    };
    assert!(TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn test_event_type_mismatch() {
    let event = BusEvent {
        event_type: "file.deleted".to_string(),
        payload: None,
    };
    let trigger = SkillTrigger::Event {
        event: "file.changed".to_string(),
        pattern: None,
    };
    assert!(!TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn test_event_with_glob_pattern_match() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: Some("/home/alice/documents/report.txt".to_string()),
    };
    let trigger = SkillTrigger::Event {
        event: "file.changed".to_string(),
        pattern: Some("/home/*/documents/*".to_string()),
    };
    assert!(TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn test_event_with_glob_pattern_no_match() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: Some("/var/log/syslog".to_string()),
    };
    let trigger = SkillTrigger::Event {
        event: "file.changed".to_string(),
        pattern: Some("/home/*/documents/*".to_string()),
    };
    assert!(!TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn test_event_pattern_but_no_payload() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: None,
    };
    let trigger = SkillTrigger::Event {
        event: "file.changed".to_string(),
        pattern: Some("/home/*".to_string()),
    };
    assert!(!TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn test_cron_trigger_never_matches_event() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: None,
    };
    let trigger = SkillTrigger::Cron {
        cron: "0 * * * *".to_string(),
    };
    assert!(!TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn test_manual_trigger_never_matches_event() {
    let event = BusEvent {
        event_type: "file.changed".to_string(),
        payload: None,
    };
    let trigger = SkillTrigger::Manual { manual: true };
    assert!(!TriggerEvaluator::matches(&event, &trigger));
}

#[test]
fn test_glob_match_double_star() {
    assert!(glob_match("/home/**", "/home/alice/documents/report.txt"));
    assert!(glob_match(
        "/home/**/report.txt",
        "/home/alice/documents/report.txt"
    ));
}

#[test]
fn test_glob_match_question_mark() {
    assert!(glob_match("/home/alic?", "/home/alice"));
    assert!(!glob_match("/home/alic?", "/home/alices"));
}

#[test]
fn test_glob_exact() {
    assert!(glob_match("/exact/path", "/exact/path"));
    assert!(!glob_match("/exact/path", "/exact/other"));
}

#[test]
fn test_glob_match_special_chars_escaped() {
    // Dots, brackets, etc. should be escaped in glob->regex conversion
    assert!(glob_match("file.txt", "file.txt"));
    assert!(!glob_match("file.txt", "fileatxt"));
}

#[test]
fn test_glob_match_double_star_with_slash() {
    assert!(glob_match("**/file.txt", "/a/b/c/file.txt"));
    assert!(glob_match("/root/**/leaf", "/root/a/b/leaf"));
}

#[test]
fn test_glob_match_empty_pattern_empty_input() {
    assert!(glob_match("", ""));
    assert!(!glob_match("", "notempty"));
}

#[test]
fn test_glob_escaped_special_regex_chars() {
    // Test pattern containing +, (, ), { chars that need escaping
    assert!(glob_match("a+b", "a+b"));
    assert!(glob_match("(test)", "(test)"));
}
