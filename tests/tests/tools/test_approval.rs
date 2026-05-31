use hudhudscript_tools::approval::is_valid_transition;
use hudhudscript_tools::approval::{ApprovalError, ApprovalGate, ApprovalRegistry, ApprovalState};
use serde_json::json;

#[test]
fn test_submit_and_get() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("delete_file", json!({"path": "/tmp/test.txt"}));
    let req = registry.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Pending);
    assert_eq!(req.tool_name, "delete_file");
}

#[test]
fn test_approve_and_execute() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("write_file", json!({}));

    registry.approve(&id, Some("looks safe".into())).unwrap();
    assert_eq!(registry.get(&id).unwrap().state, ApprovalState::Approved);
    assert_eq!(
        registry.get(&id).unwrap().reason.as_deref(),
        Some("looks safe")
    );

    registry.mark_executed(&id).unwrap();
    assert_eq!(registry.get(&id).unwrap().state, ApprovalState::Executed);
}

#[test]
fn test_deny_and_skip() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("dangerous_op", json!({}));

    registry.deny(&id, Some("too risky".into())).unwrap();
    assert_eq!(registry.get(&id).unwrap().state, ApprovalState::Denied);

    registry.mark_skipped(&id).unwrap();
    assert_eq!(registry.get(&id).unwrap().state, ApprovalState::Skipped);
}

#[test]
fn test_invalid_transition() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("tool", json!({}));

    // Cannot go directly from Pending to Executed
    let err = registry.mark_executed(&id).unwrap_err();
    assert!(matches!(err, ApprovalError::InvalidTransition { .. }));
}

#[test]
fn test_pending_list() {
    let registry = ApprovalRegistry::new();
    let id1 = registry.submit("tool_a", json!({}));
    let id2 = registry.submit("tool_b", json!({}));

    let pending = registry.pending();
    assert_eq!(pending.len(), 2);

    registry.approve(&id1, None).unwrap();
    let pending = registry.pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, id2);
}

#[test]
fn test_approval_gate_needs_approval() {
    let registry = ApprovalRegistry::new();
    let mut gate = ApprovalGate::new(registry);
    gate.require_approval_for("delete_file");

    assert!(gate.needs_approval("delete_file"));
    assert!(!gate.needs_approval("read_file"));
}

#[test]
fn test_not_found_error() {
    let registry = ApprovalRegistry::new();
    let err = registry.approve("nonexistent-id", None).unwrap_err();
    assert!(matches!(err, ApprovalError::NotFound(_)));
}

#[test]
fn test_invalid_transition_approved_to_approved() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("tool", json!({}));
    registry.approve(&id, None).unwrap();
    let err = registry.approve(&id, None).unwrap_err();
    assert!(matches!(err, ApprovalError::InvalidTransition { .. }));
}

#[test]
fn test_invalid_transition_denied_to_executed() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("tool", json!({}));
    registry.deny(&id, None).unwrap();
    let err = registry.mark_executed(&id).unwrap_err();
    assert!(matches!(err, ApprovalError::InvalidTransition { .. }));
}

#[test]
fn test_invalid_transition_approved_to_skipped() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("tool", json!({}));
    registry.approve(&id, None).unwrap();
    let err = registry.mark_skipped(&id).unwrap_err();
    assert!(matches!(err, ApprovalError::InvalidTransition { .. }));
}

#[test]
fn test_approval_state_display() {
    assert_eq!(ApprovalState::Pending.to_string(), "Pending");
    assert_eq!(ApprovalState::Approved.to_string(), "Approved");
    assert_eq!(ApprovalState::Denied.to_string(), "Denied");
    assert_eq!(ApprovalState::Executed.to_string(), "Executed");
    assert_eq!(ApprovalState::Skipped.to_string(), "Skipped");
}

#[test]
fn test_approval_error_display() {
    let err = ApprovalError::NotFound("id123".to_string());
    assert!(err.to_string().contains("id123"));

    let err = ApprovalError::InvalidTransition {
        id: "id456".to_string(),
        from: ApprovalState::Pending,
        to: ApprovalState::Executed,
    };
    let msg = err.to_string();
    assert!(msg.contains("Pending"));
    assert!(msg.contains("Executed"));
    assert!(msg.contains("id456"));
}

#[test]
fn test_approval_gate_request_for_non_gated_tool() {
    let registry = ApprovalRegistry::new();
    let gate = ApprovalGate::new(registry);
    // Should still create a request even if not gated
    let id = gate.request_approval("ungated_tool", json!({}));
    assert!(!id.is_empty());
    let req = gate.registry().get(&id).unwrap();
    assert_eq!(req.tool_name, "ungated_tool");
}

#[test]
fn test_approval_gate_dedup() {
    let registry = ApprovalRegistry::new();
    let mut gate = ApprovalGate::new(registry);
    gate.require_approval_for("delete_file");
    gate.require_approval_for("delete_file"); // duplicate
    assert!(gate.needs_approval("delete_file"));
}

#[test]
fn test_approval_registry_default() {
    let registry = ApprovalRegistry::default();
    assert!(registry.pending().is_empty());
}

#[test]
fn test_deny_without_reason() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("tool", json!({}));
    registry.deny(&id, None).unwrap();
    let req = registry.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Denied);
    assert!(req.reason.is_none());
}

#[test]
fn test_approve_without_reason() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("tool", json!({}));
    registry.approve(&id, None).unwrap();
    let req = registry.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Approved);
    assert!(req.reason.is_none());
}

#[test]
fn test_get_nonexistent_returns_none() {
    let registry = ApprovalRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_approval_state_serde_roundtrip() {
    let states = vec![
        ApprovalState::Pending,
        ApprovalState::Approved,
        ApprovalState::Denied,
        ApprovalState::Executed,
        ApprovalState::Skipped,
    ];
    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ApprovalState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, state);
    }
}

#[test]
fn test_is_valid_transition_exhaustive() {
    // Valid transitions
    assert!(is_valid_transition(
        &ApprovalState::Pending,
        &ApprovalState::Approved
    ));
    assert!(is_valid_transition(
        &ApprovalState::Pending,
        &ApprovalState::Denied
    ));
    assert!(is_valid_transition(
        &ApprovalState::Approved,
        &ApprovalState::Executed
    ));
    assert!(is_valid_transition(
        &ApprovalState::Denied,
        &ApprovalState::Skipped
    ));

    // Invalid transitions
    assert!(!is_valid_transition(
        &ApprovalState::Pending,
        &ApprovalState::Executed
    ));
    assert!(!is_valid_transition(
        &ApprovalState::Pending,
        &ApprovalState::Skipped
    ));
    assert!(!is_valid_transition(
        &ApprovalState::Approved,
        &ApprovalState::Denied
    ));
    assert!(!is_valid_transition(
        &ApprovalState::Approved,
        &ApprovalState::Skipped
    ));
    assert!(!is_valid_transition(
        &ApprovalState::Denied,
        &ApprovalState::Approved
    ));
    assert!(!is_valid_transition(
        &ApprovalState::Denied,
        &ApprovalState::Executed
    ));
    assert!(!is_valid_transition(
        &ApprovalState::Executed,
        &ApprovalState::Pending
    ));
    assert!(!is_valid_transition(
        &ApprovalState::Skipped,
        &ApprovalState::Pending
    ));
}
