use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::event_bus_ops::{
    event_emit, event_has_listeners, event_matches, event_off, event_on, event_once,
};

#[test]
fn test_event_matches_exact() {
    assert!(event_matches("user.created", "user.created"));
    assert!(!event_matches("user.created", "user.deleted"));
}

#[test]
fn test_event_matches_wildcard() {
    assert!(event_matches("user.created", "user.*"));
    assert!(event_matches("user.created", "*.created"));
    assert!(event_matches("anything", "*"));
}

#[test]
fn test_event_matches_no_partial() {
    assert!(!event_matches("user.created.extra", "user.*"));
}

#[test]
fn test_emit_returns_object() {
    let result = event_emit(&[Value16::string("test.event".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(
            obj.get("event"),
            Some(&Value16::string("test.event".to_string()))
        );
        // No subscribers registered → delivered is false (correct behavior)
        assert_eq!(obj.get("delivered"), Some(&Value16::boolean(false)));
        assert_eq!(obj.get("listener_count"), Some(&Value16::number(0.0)));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_on_returns_subscription() {
    let result = event_on(&[
        Value16::string("user.*".to_string()),
        Value16::string("handle_user".to_string()),
    ])
    .unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(
            obj.get("pattern"),
            Some(&Value16::string("user.*".to_string()))
        );
        assert_eq!(obj.get("active"), Some(&Value16::boolean(true)));
        assert_eq!(obj.get("once"), Some(&Value16::boolean(false)));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_once_returns_subscription_with_once_flag() {
    let result = event_once(&[Value16::string("shutdown".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert_eq!(obj.get("once"), Some(&Value16::boolean(true)));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_off_requires_string() {
    assert!(event_off(&[Value16::string("sub_123".to_string())]).is_ok());
    assert!(event_off(&[Value16::number(42.0)]).is_err());
}

#[test]
fn test_has_listeners() {
    let result = event_has_listeners(&[Value16::string("test".to_string())]).unwrap();
    assert_eq!(result, Value16::boolean(false));
}
