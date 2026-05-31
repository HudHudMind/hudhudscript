use hudhudscript_runtime::agent::{AgentState, StateValue};
use hudhudscript_runtime::persistence::{FilePersistence, PersistenceError};
use tempfile::tempdir;

#[test]
fn test_save_and_load() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();

    let mut state = AgentState::new("agent-1".to_string());
    state.set("counter".to_string(), StateValue::Number(42.0));
    state.set("name".to_string(), StateValue::String("test".to_string()));
    state.set("active".to_string(), StateValue::Boolean(true));
    state.set(
        "tags".to_string(),
        StateValue::Array(vec![
            StateValue::String("a".to_string()),
            StateValue::String("b".to_string()),
        ]),
    );

    persistence.save(&state).unwrap();
    assert!(persistence.exists("agent-1"));

    let loaded = persistence.load("agent-1").unwrap();
    assert_eq!(loaded.agent_id, "agent-1");
    assert_eq!(loaded.version, state.version);

    match loaded.get("counter") {
        Some(StateValue::Number(n)) => assert_eq!(*n, 42.0),
        other => unreachable!("counter not found or wrong type, got: {:?}", other),
    }
    match loaded.get("name") {
        Some(StateValue::String(s)) => assert_eq!(s, "test"),
        other => unreachable!("name not found or wrong type, got: {:?}", other),
    }
    match loaded.get("active") {
        Some(StateValue::Boolean(b)) => assert!(*b),
        other => unreachable!("active not found or wrong type, got: {:?}", other),
    }
}

#[test]
fn test_not_found() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();
    let result = persistence.load("nonexistent");
    assert!(matches!(result, Err(PersistenceError::NotFound(_))));
}

#[test]
fn test_delete() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();

    let state = AgentState::new("agent-del".to_string());
    persistence.save(&state).unwrap();
    assert!(persistence.exists("agent-del"));

    persistence.delete("agent-del").unwrap();
    assert!(!persistence.exists("agent-del"));
}

#[test]
fn test_object_state_value() {
    let dir = tempdir().unwrap();
    let persistence = FilePersistence::new(dir.path()).unwrap();

    let mut state = AgentState::new("agent-obj".to_string());
    let mut obj = std::collections::HashMap::new();
    obj.insert("key".to_string(), StateValue::String("value".to_string()));
    state.set("config".to_string(), StateValue::Object(obj));

    persistence.save(&state).unwrap();
    let loaded = persistence.load("agent-obj").unwrap();

    match loaded.get("config") {
        Some(StateValue::Object(o)) => {
            assert!(o.contains_key("key"));
        }
        other => unreachable!("config not found or wrong type, got: {:?}", other),
    }
}
