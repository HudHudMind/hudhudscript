use chrono::Duration;
use hudhudscript_governance::audit::{AuditEvent, AuditEventData, AuditEventType, AuditLogger};
use hudhudscript_governance::*;

#[test]
fn test_audit_logger_creation() {
    let logger = AuditLogger::new();
    assert_eq!(logger.count(), 0);
}

#[test]
fn test_log_enforcement_decision() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Data processing action".to_string(),
        true,
        vec![],
        vec![],
    );

    assert_eq!(logger.count(), 1);

    let events = logger.get_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::EnforcementDecision);

    match &events[0].data {
        AuditEventData::Enforcement {
            constitution_id,
            action_description,
            allowed,
            violations,
            advisory_violations,
        } => {
            assert_eq!(constitution_id, "cons.1");
            assert_eq!(action_description, "Data processing action");
            assert!(allowed);
            assert_eq!(violations.len(), 0);
            assert_eq!(advisory_violations.len(), 0);
        }
        other => unreachable!("Expected Enforcement event data, got: {:?}", other),
    }
}

#[test]
fn test_log_enforcement_decision_with_violations() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Unauthorized access".to_string(),
        false,
        vec!["cons1.law1".to_string(), "cons1.law2".to_string()],
        vec!["cons1.law3".to_string()],
    );

    let events = logger.get_events();
    assert_eq!(events.len(), 1);

    match &events[0].data {
        AuditEventData::Enforcement {
            allowed,
            violations,
            advisory_violations,
            ..
        } => {
            assert!(!allowed);
            assert_eq!(violations.len(), 2);
            assert_eq!(advisory_violations.len(), 1);
        }
        other => unreachable!("Expected Enforcement event data, got: {:?}", other),
    }
}

#[test]
fn test_log_constitution_modification() {
    let logger = AuditLogger::new();

    logger.log_constitution_modification(
        "cons.1".to_string(),
        "law_added".to_string(),
        1,
        2,
        "Added new data protection law".to_string(),
    );

    assert_eq!(logger.count(), 1);

    let events = logger.get_events();
    assert_eq!(events[0].event_type, AuditEventType::ConstitutionModified);

    match &events[0].data {
        AuditEventData::ConstitutionModification {
            constitution_id,
            modification_type,
            old_version,
            new_version,
            description,
        } => {
            assert_eq!(constitution_id, "cons.1");
            assert_eq!(modification_type, "law_added");
            assert_eq!(*old_version, 1);
            assert_eq!(*new_version, 2);
            assert_eq!(description, "Added new data protection law");
        }
        other => unreachable!(
            "Expected ConstitutionModification event data, got: {:?}",
            other
        ),
    }
}

#[test]
fn test_log_agent_role_change() {
    let logger = AuditLogger::new();

    logger.log_agent_role_change(
        "agent1".to_string(),
        Some(AgentRole::Member),
        AgentRole::Judge,
        "council1".to_string(),
    );

    assert_eq!(logger.count(), 1);

    let events = logger.get_events();
    assert_eq!(events[0].event_type, AuditEventType::AgentRoleChanged);

    match &events[0].data {
        AuditEventData::RoleChange {
            agent_id,
            old_role,
            new_role,
            council_id,
        } => {
            assert_eq!(agent_id, "agent1");
            assert_eq!(old_role, &Some(AgentRole::Member));
            assert_eq!(new_role, &AgentRole::Judge);
            assert_eq!(council_id, "council1");
        }
        other => unreachable!("Expected RoleChange event data, got: {:?}", other),
    }
}

#[test]
fn test_log_agent_role_change_new_member() {
    let logger = AuditLogger::new();

    logger.log_agent_role_change(
        "agent2".to_string(),
        None,
        AgentRole::Prosecutor,
        "council1".to_string(),
    );

    let events = logger.get_events();
    match &events[0].data {
        AuditEventData::RoleChange { old_role, .. } => {
            assert_eq!(old_role, &None);
        }
        other => unreachable!("Expected RoleChange event data, got: {:?}", other),
    }
}

#[test]
fn test_log_cache_access_hit() {
    let logger = AuditLogger::new();

    logger.log_cache_access("resolve".to_string(), "cons.1".to_string(), true);

    assert_eq!(logger.count(), 1);

    let events = logger.get_events();
    assert_eq!(events[0].event_type, AuditEventType::CacheAccess);

    match &events[0].data {
        AuditEventData::CacheAccess {
            operation,
            cache_key,
            hit,
        } => {
            assert_eq!(operation, "resolve");
            assert_eq!(cache_key, "cons.1");
            assert!(hit);
        }
        other => unreachable!("Expected CacheAccess event data, got: {:?}", other),
    }
}

#[test]
fn test_log_cache_access_miss() {
    let logger = AuditLogger::new();

    logger.log_cache_access("resolve".to_string(), "cons.999".to_string(), false);

    let events = logger.get_events();
    match &events[0].data {
        AuditEventData::CacheAccess { hit, .. } => {
            assert!(!hit);
        }
        other => unreachable!("Expected CacheAccess event data, got: {:?}", other),
    }
}

#[test]
fn test_multiple_events() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Action 1".to_string(),
        true,
        vec![],
        vec![],
    );

    logger.log_constitution_modification(
        "cons.1".to_string(),
        "update".to_string(),
        1,
        2,
        "Updated".to_string(),
    );

    logger.log_agent_role_change(
        "agent1".to_string(),
        None,
        AgentRole::Member,
        "council1".to_string(),
    );

    logger.log_cache_access("store".to_string(), "cons.1".to_string(), true);

    assert_eq!(logger.count(), 4);

    let events = logger.get_events();
    assert_eq!(events.len(), 4);
}

#[test]
fn test_get_events_by_type() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Action 1".to_string(),
        true,
        vec![],
        vec![],
    );

    logger.log_enforcement_decision(
        "cons.2".to_string(),
        "Action 2".to_string(),
        false,
        vec!["cons2.law1".to_string()],
        vec![],
    );

    logger.log_constitution_modification(
        "cons.1".to_string(),
        "update".to_string(),
        1,
        2,
        "Updated".to_string(),
    );

    let enforcement_events = logger.get_events_by_type(AuditEventType::EnforcementDecision);
    assert_eq!(enforcement_events.len(), 2);

    let modification_events = logger.get_events_by_type(AuditEventType::ConstitutionModified);
    assert_eq!(modification_events.len(), 1);

    let role_events = logger.get_events_by_type(AuditEventType::AgentRoleChanged);
    assert_eq!(role_events.len(), 0);
}

#[test]
fn test_get_events_in_range() {
    use chrono::Utc;

    let logger = AuditLogger::new();

    let start = Utc::now();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Action 1".to_string(),
        true,
        vec![],
        vec![],
    );

    std::thread::sleep(std::time::Duration::from_millis(10));

    logger.log_constitution_modification(
        "cons.1".to_string(),
        "update".to_string(),
        1,
        2,
        "Updated".to_string(),
    );

    let end = Utc::now();

    let events = logger.get_events_in_range(start, end);
    assert_eq!(events.len(), 2);

    // Test range that excludes events
    let future_start = Utc::now() + Duration::hours(1);
    let future_end = Utc::now() + Duration::hours(2);
    let future_events = logger.get_events_in_range(future_start, future_end);
    assert_eq!(future_events.len(), 0);
}

#[test]
fn test_export_to_json() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Test action".to_string(),
        true,
        vec![],
        vec![],
    );

    let json = logger.export_to_json().unwrap();
    assert!(json.contains("cons.1"));
    assert!(json.contains("Test action"));
    assert!(json.contains("EnforcementDecision"));

    // Verify it's valid JSON
    let parsed: Vec<AuditEvent> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 1);
}

#[test]
fn test_export_to_json_compact() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Test action".to_string(),
        true,
        vec![],
        vec![],
    );

    let json = logger.export_to_json_compact().unwrap();
    assert!(json.contains("cons.1"));

    // Compact JSON should not have pretty formatting
    assert!(!json.contains("\n  "));

    // Verify it's valid JSON
    let parsed: Vec<AuditEvent> = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.len(), 1);
}

#[test]
fn test_clear() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Action 1".to_string(),
        true,
        vec![],
        vec![],
    );

    logger.log_constitution_modification(
        "cons.1".to_string(),
        "update".to_string(),
        1,
        2,
        "Updated".to_string(),
    );

    assert_eq!(logger.count(), 2);

    logger.clear();

    assert_eq!(logger.count(), 0);
    assert_eq!(logger.get_events().len(), 0);
}

#[test]
fn test_event_id_generation() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Action 1".to_string(),
        true,
        vec![],
        vec![],
    );

    logger.log_enforcement_decision(
        "cons.2".to_string(),
        "Action 2".to_string(),
        true,
        vec![],
        vec![],
    );

    let events = logger.get_events();
    assert_eq!(events[0].id, "audit.1");
    assert_eq!(events[1].id, "audit.2");
}

#[test]
fn test_timestamp_ordering() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Action 1".to_string(),
        true,
        vec![],
        vec![],
    );

    std::thread::sleep(std::time::Duration::from_millis(10));

    logger.log_enforcement_decision(
        "cons.2".to_string(),
        "Action 2".to_string(),
        true,
        vec![],
        vec![],
    );

    let events = logger.get_events();
    assert!(events[0].timestamp <= events[1].timestamp);
}

#[test]
fn test_custom_role_in_audit() {
    let logger = AuditLogger::new();

    logger.log_agent_role_change(
        "agent1".to_string(),
        Some(AgentRole::Member),
        AgentRole::Custom("DataValidator".to_string()),
        "council1".to_string(),
    );

    let events = logger.get_events();
    match &events[0].data {
        AuditEventData::RoleChange { new_role, .. } => {
            assert_eq!(new_role, &AgentRole::Custom("DataValidator".to_string()));
        }
        other => unreachable!("Expected RoleChange event data, got: {:?}", other),
    }
}

#[test]
fn test_serialization_roundtrip() {
    let logger = AuditLogger::new();

    logger.log_enforcement_decision(
        "cons.1".to_string(),
        "Test action".to_string(),
        true,
        vec!["cons1.law1".to_string()],
        vec!["cons1.law2".to_string()],
    );

    logger.log_constitution_modification(
        "cons.1".to_string(),
        "law_added".to_string(),
        1,
        2,
        "Added law".to_string(),
    );

    logger.log_agent_role_change(
        "agent1".to_string(),
        None,
        AgentRole::Judge,
        "council1".to_string(),
    );

    logger.log_cache_access("resolve".to_string(), "cons.1".to_string(), true);

    // Export to JSON
    let json = logger.export_to_json().unwrap();

    // Parse back
    let parsed: Vec<AuditEvent> = serde_json::from_str(&json).unwrap();

    // Verify all events are preserved
    assert_eq!(parsed.len(), 4);
    assert_eq!(parsed[0].event_type, AuditEventType::EnforcementDecision);
    assert_eq!(parsed[1].event_type, AuditEventType::ConstitutionModified);
    assert_eq!(parsed[2].event_type, AuditEventType::AgentRoleChanged);
    assert_eq!(parsed[3].event_type, AuditEventType::CacheAccess);
}

#[test]
fn test_concurrent_logging() {
    use std::sync::Arc;
    use std::thread;

    let logger = Arc::new(AuditLogger::new());
    let mut handles = vec![];

    // Spawn multiple threads logging concurrently
    for i in 0..10 {
        let logger_clone = Arc::clone(&logger);
        let handle = thread::spawn(move || {
            logger_clone.log_enforcement_decision(
                format!("cons.{}", i),
                format!("Action {}", i),
                true,
                vec![],
                vec![],
            );
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all events were logged
    assert_eq!(logger.count(), 10);
}
