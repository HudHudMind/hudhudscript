use hudhudscript_tools_ops::*;
use hudhudscript_tools_schema::registry::RegistryError;
use serde_json::json;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

// =========================================================================
// ApprovalState
// =========================================================================

#[test]
fn approval_state_display_pending() {
    assert_eq!(format!("{}", ApprovalState::Pending), "Pending");
}

#[test]
fn approval_state_display_approved() {
    assert_eq!(format!("{}", ApprovalState::Approved), "Approved");
}

#[test]
fn approval_state_display_denied() {
    assert_eq!(format!("{}", ApprovalState::Denied), "Denied");
}

#[test]
fn approval_state_display_executed() {
    assert_eq!(format!("{}", ApprovalState::Executed), "Executed");
}

#[test]
fn approval_state_display_skipped() {
    assert_eq!(format!("{}", ApprovalState::Skipped), "Skipped");
}

#[test]
fn approval_state_eq() {
    assert_eq!(ApprovalState::Pending, ApprovalState::Pending);
    assert_ne!(ApprovalState::Pending, ApprovalState::Approved);
}

#[test]
fn approval_state_clone() {
    let s = ApprovalState::Approved;
    let s2 = s.clone();
    assert_eq!(s, s2);
}

// =========================================================================
// ApprovalRegistry
// =========================================================================

#[test]
fn registry_new_is_empty() {
    let reg = ApprovalRegistry::new();
    assert!(reg.pending().is_empty());
}

#[test]
fn registry_default_is_empty() {
    let reg = ApprovalRegistry::default();
    assert!(reg.pending().is_empty());
}

#[test]
fn registry_submit_returns_id() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("test_tool", json!({}));
    assert!(!id.is_empty());
}

#[test]
fn registry_submit_creates_pending_request() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool_a", json!({"key": "value"}));
    let req = reg.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Pending);
    assert_eq!(req.tool_name, "tool_a");
}

#[test]
fn registry_get_nonexistent_returns_none() {
    let reg = ApprovalRegistry::new();
    assert!(reg.get("nonexistent").is_none());
}

#[test]
fn registry_pending_returns_only_pending() {
    let reg = ApprovalRegistry::new();
    let id1 = reg.submit("tool_a", json!({}));
    let _id2 = reg.submit("tool_b", json!({}));
    reg.approve(&id1, None).unwrap();
    let pending = reg.pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].tool_name, "tool_b");
}

#[test]
fn registry_approve_pending_succeeds() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool", json!({}));
    assert!(reg.approve(&id, Some("looks good".into())).is_ok());
    let req = reg.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Approved);
    assert_eq!(req.reason, Some("looks good".to_string()));
}

#[test]
fn registry_deny_pending_succeeds() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool", json!({}));
    assert!(reg.deny(&id, None).is_ok());
    let req = reg.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Denied);
}

#[test]
fn registry_mark_executed_after_approve() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool", json!({}));
    reg.approve(&id, None).unwrap();
    assert!(reg.mark_executed(&id).is_ok());
    let req = reg.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Executed);
}

#[test]
fn registry_mark_skipped_after_deny() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool", json!({}));
    reg.deny(&id, None).unwrap();
    assert!(reg.mark_skipped(&id).is_ok());
    let req = reg.get(&id).unwrap();
    assert_eq!(req.state, ApprovalState::Skipped);
}

#[test]
fn registry_invalid_transition_pending_to_executed() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool", json!({}));
    let err = reg.mark_executed(&id).unwrap_err();
    assert!(matches!(err, ApprovalError::InvalidTransition { .. }));
}

#[test]
fn registry_invalid_transition_pending_to_skipped() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool", json!({}));
    assert!(reg.mark_skipped(&id).is_err());
}

#[test]
fn registry_invalid_transition_approved_to_denied() {
    let reg = ApprovalRegistry::new();
    let id = reg.submit("tool", json!({}));
    reg.approve(&id, None).unwrap();
    assert!(reg.deny(&id, None).is_err());
}

#[test]
fn registry_not_found_error() {
    let reg = ApprovalRegistry::new();
    let err = reg.approve("missing", None).unwrap_err();
    assert!(matches!(err, ApprovalError::NotFound(_)));
}

// =========================================================================
// ApprovalGate
// =========================================================================

#[test]
fn approval_gate_needs_approval_default_false() {
    let reg = ApprovalRegistry::new();
    let gate = ApprovalGate::new(reg);
    assert!(!gate.needs_approval("some_tool"));
}

#[test]
fn approval_gate_require_and_check() {
    let reg = ApprovalRegistry::new();
    let mut gate = ApprovalGate::new(reg);
    gate.require_approval_for("dangerous_tool");
    assert!(gate.needs_approval("dangerous_tool"));
    assert!(!gate.needs_approval("safe_tool"));
}

#[test]
fn approval_gate_require_duplicate_no_panic() {
    let reg = ApprovalRegistry::new();
    let mut gate = ApprovalGate::new(reg);
    gate.require_approval_for("tool");
    gate.require_approval_for("tool");
    assert!(gate.needs_approval("tool"));
}

#[test]
fn approval_gate_request_creates_entry() {
    let reg = ApprovalRegistry::new();
    let mut gate = ApprovalGate::new(reg);
    gate.require_approval_for("tool");
    let id = gate.request_approval("tool", json!({}));
    assert!(gate.registry().get(&id).is_some());
}

// =========================================================================
// PromptResponse
// =========================================================================

#[test]
fn prompt_response_display() {
    assert_eq!(format!("{}", PromptResponse::Yes), "yes");
    assert_eq!(format!("{}", PromptResponse::No), "no");
    assert_eq!(format!("{}", PromptResponse::AlwaysAllow), "always-allow");
    assert_eq!(format!("{}", PromptResponse::AlwaysDeny), "always-deny");
}

#[test]
fn prompt_response_eq() {
    assert_eq!(PromptResponse::Yes, PromptResponse::Yes);
    assert_ne!(PromptResponse::Yes, PromptResponse::No);
}

// =========================================================================
// AutoApprovePrompter / AutoDenyPrompter
// =========================================================================

#[test]
fn auto_approve_prompter_always_yes() {
    let prompter = AutoApprovePrompter;
    let resp = prompter.prompt("tool", &json!({}), RiskLevel::Dangerous);
    assert_eq!(resp, PromptResponse::Yes);
}

#[test]
fn auto_deny_prompter_always_no() {
    let prompter = AutoDenyPrompter;
    let resp = prompter.prompt("tool", &json!({}), RiskLevel::Warning);
    assert_eq!(resp, PromptResponse::No);
}

// =========================================================================
// RiskLevel
// =========================================================================

#[test]
fn risk_level_display() {
    assert_eq!(format!("{}", RiskLevel::Safe), "safe");
    assert_eq!(format!("{}", RiskLevel::Warning), "warning");
    assert_eq!(format!("{}", RiskLevel::Dangerous), "dangerous");
}

#[test]
fn risk_level_ord() {
    assert!(RiskLevel::Safe < RiskLevel::Warning);
    assert!(RiskLevel::Warning < RiskLevel::Dangerous);
}

// =========================================================================
// RiskRule
// =========================================================================

#[test]
fn risk_rule_exact_match() {
    let rule = RiskRule::new("format_disk", RiskLevel::Dangerous, "irreversible");
    assert!(rule.matches("format_disk"));
    assert!(!rule.matches("format_disk_extra"));
}

#[test]
fn risk_rule_wildcard_match() {
    let rule = RiskRule::new("delete_*", RiskLevel::Dangerous, "deletion");
    assert!(rule.matches("delete_file"));
    assert!(rule.matches("delete_everything"));
    assert!(!rule.matches("undelete_file"));
}

// =========================================================================
// RiskEngine
// =========================================================================

#[test]
fn risk_engine_default_is_safe() {
    let engine = RiskEngine::new();
    let assessment = engine.assess("unknown_tool");
    assert_eq!(assessment.level, RiskLevel::Safe);
    assert!(!assessment.requires_approval);
}

#[test]
fn risk_engine_with_defaults_dangerous() {
    let engine = RiskEngine::with_defaults();
    let a = engine.assess("delete_user");
    assert_eq!(a.level, RiskLevel::Dangerous);
    assert!(a.requires_approval);
    assert!(a.matched_rule.is_some());
}

#[test]
fn risk_engine_with_defaults_warning() {
    let engine = RiskEngine::with_defaults();
    let a = engine.assess("write_file");
    assert_eq!(a.level, RiskLevel::Warning);
}

#[test]
fn risk_engine_with_defaults_safe() {
    let engine = RiskEngine::with_defaults();
    let a = engine.assess("read_data");
    assert_eq!(a.level, RiskLevel::Safe);
}

#[test]
fn risk_engine_override_takes_priority() {
    let mut engine = RiskEngine::with_defaults();
    engine.set_override("delete_temp", RiskLevel::Safe);
    let a = engine.assess("delete_temp");
    assert_eq!(a.level, RiskLevel::Safe);
    assert!(a.matched_rule.is_none());
}

#[test]
fn risk_engine_remove_override() {
    let mut engine = RiskEngine::new();
    engine.set_override("tool_a", RiskLevel::Dangerous);
    let removed = engine.remove_override("tool_a");
    assert_eq!(removed, Some(RiskLevel::Dangerous));
    assert!(engine.remove_override("tool_a").is_none());
}

#[test]
fn risk_engine_set_default_level() {
    let mut engine = RiskEngine::new();
    engine.set_default_level(RiskLevel::Dangerous);
    let a = engine.assess("any_tool");
    assert_eq!(a.level, RiskLevel::Dangerous);
    assert!(a.requires_approval);
}

#[test]
fn risk_engine_first_match_wins() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new("tool_*", RiskLevel::Warning, "first"));
    engine.add_rule(RiskRule::new("tool_*", RiskLevel::Dangerous, "second"));
    let a = engine.assess("tool_abc");
    assert_eq!(a.level, RiskLevel::Warning);
}

#[test]
fn risk_engine_rules_accessor() {
    let engine = RiskEngine::with_defaults();
    assert!(!engine.rules().is_empty());
}

// =========================================================================
// AuditLog
// =========================================================================

#[test]
fn audit_log_default_is_empty() {
    let log = AuditLog::default();
    assert!(log.is_empty());
    assert_eq!(log.len(), 0);
}

#[test]
fn audit_log_log_decision_and_retrieve() {
    let log = AuditLog::new(100);
    let id = log.log_decision(
        "ap-1",
        "tool_a",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::Approved,
        None,
        "session-1",
    );
    assert!(!id.is_empty());
    assert_eq!(log.len(), 1);
    let entries = log.entries();
    assert_eq!(entries[0].tool_name, "tool_a");
}

#[test]
fn audit_log_entries_for_tool() {
    let log = AuditLog::new(100);
    log.log_decision(
        "",
        "tool_a",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::Approved,
        None,
        "s1",
    );
    log.log_decision(
        "",
        "tool_b",
        json!({}),
        RiskLevel::Warning,
        AuditDecision::Denied,
        None,
        "s1",
    );
    log.log_decision(
        "",
        "tool_a",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::AutoApproved,
        None,
        "s1",
    );
    let a_entries = log.entries_for_tool("tool_a");
    assert_eq!(a_entries.len(), 2);
}

#[test]
fn audit_log_entries_for_session() {
    let log = AuditLog::new(100);
    log.log_decision(
        "",
        "t",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::Approved,
        None,
        "s1",
    );
    log.log_decision(
        "",
        "t",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::Approved,
        None,
        "s2",
    );
    assert_eq!(log.entries_for_session("s1").len(), 1);
    assert_eq!(log.entries_for_session("s2").len(), 1);
}

#[test]
fn audit_log_ring_buffer() {
    let log = AuditLog::new(3);
    for i in 0..5 {
        log.log_decision(
            "",
            &format!("tool_{i}"),
            json!({}),
            RiskLevel::Safe,
            AuditDecision::Approved,
            None,
            "s",
        );
    }
    assert_eq!(log.len(), 3);
    let entries = log.entries();
    assert_eq!(entries[0].tool_name, "tool_2");
}

#[test]
fn audit_log_clear() {
    let log = AuditLog::new(100);
    log.log_decision(
        "",
        "t",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::Approved,
        None,
        "s",
    );
    log.clear();
    assert!(log.is_empty());
}

#[test]
fn audit_log_to_json_and_load() {
    let log = AuditLog::new(100);
    log.log_decision(
        "ap1",
        "tool_x",
        json!({"a":1}),
        RiskLevel::Warning,
        AuditDecision::Denied,
        Some("reason".into()),
        "sess",
    );
    let json_str = log.to_json().unwrap();
    assert!(json_str.contains("tool_x"));

    let log2 = AuditLog::new(100);
    let loaded = log2.load_from_json(&json_str).unwrap();
    assert_eq!(loaded, 1);
    assert_eq!(log2.len(), 1);
}

#[test]
fn audit_decision_display() {
    assert_eq!(format!("{}", AuditDecision::Approved), "approved");
    assert_eq!(format!("{}", AuditDecision::Denied), "denied");
    assert_eq!(format!("{}", AuditDecision::AutoApproved), "auto-approved");
    assert_eq!(
        format!("{}", AuditDecision::SafeAutoApproved),
        "safe-auto-approved"
    );
}

// =========================================================================
// SessionPermissions
// =========================================================================

#[test]
fn session_permissions_new() {
    let sp = SessionPermissions::new("session-1");
    assert_eq!(sp.session_id(), "session-1");
}

#[test]
fn session_check_returns_none_initially() {
    let sp = SessionPermissions::new("s1");
    assert!(sp.check("tool_a").is_none());
}

#[test]
fn session_always_allow() {
    let sp = SessionPermissions::new("s1");
    sp.set_always_allow("tool_a");
    assert_eq!(sp.check("tool_a"), Some(PermissionStatus::AlwaysAllow));
}

#[test]
fn session_always_deny() {
    let sp = SessionPermissions::new("s1");
    sp.set_always_deny("tool_b");
    assert_eq!(sp.check("tool_b"), Some(PermissionStatus::AlwaysDeny));
}

#[test]
fn session_check_increments_usage_count() {
    let sp = SessionPermissions::new("s1");
    sp.set_always_allow("tool_a");
    sp.check("tool_a");
    sp.check("tool_a");
    let perms = sp.all_permissions();
    let record = perms.iter().find(|p| p.tool_name == "tool_a").unwrap();
    assert_eq!(record.usage_count, 2);
}

#[test]
fn session_record_one_time_approval() {
    let sp = SessionPermissions::new("s1");
    sp.record_one_time_approval("tool_c");
    assert!(sp.was_approved_before("tool_c"));
}

#[test]
fn session_was_approved_before_with_always_allow() {
    let sp = SessionPermissions::new("s1");
    sp.set_always_allow("tool_d");
    assert!(sp.was_approved_before("tool_d"));
}

#[test]
fn session_was_approved_before_false() {
    let sp = SessionPermissions::new("s1");
    assert!(!sp.was_approved_before("tool_x"));
}

#[test]
fn session_revoke() {
    let sp = SessionPermissions::new("s1");
    sp.set_always_allow("tool_a");
    assert!(sp.revoke("tool_a"));
    assert!(sp.check("tool_a").is_none());
}

#[test]
fn session_revoke_nonexistent() {
    let sp = SessionPermissions::new("s1");
    assert!(!sp.revoke("nonexistent"));
}

#[test]
fn session_all_permissions() {
    let sp = SessionPermissions::new("s1");
    sp.set_always_allow("tool_a");
    sp.set_always_deny("tool_b");
    assert_eq!(sp.all_permissions().len(), 2);
}

#[test]
fn session_approval_history() {
    let sp = SessionPermissions::new("s1");
    sp.record_one_time_approval("tool_a");
    sp.record_one_time_approval("tool_b");
    let history = sp.approval_history();
    assert_eq!(history.len(), 2);
}

#[test]
fn session_clear() {
    let sp = SessionPermissions::new("s1");
    sp.set_always_allow("tool_a");
    sp.record_one_time_approval("tool_b");
    sp.clear();
    assert!(sp.all_permissions().is_empty());
    assert!(sp.approval_history().is_empty());
}

#[test]
fn permission_status_display() {
    assert_eq!(format!("{}", PermissionStatus::AlwaysAllow), "always-allow");
    assert_eq!(format!("{}", PermissionStatus::AlwaysDeny), "always-deny");
}

// =========================================================================
// TelemetryCollector
// =========================================================================

#[test]
fn telemetry_collector_default() {
    let c = TelemetryCollector::default();
    assert_eq!(c.record_count(), 0);
    assert!(c.all_records().is_empty());
}

#[test]
fn telemetry_collector_record_and_retrieve() {
    let c = TelemetryCollector::new(100);
    record_tool_telemetry(
        &c,
        "tool_a",
        Duration::from_millis(50),
        ExecutionStatus::Success,
        100,
        200,
        None,
    );
    assert_eq!(c.record_count(), 1);
    let records = c.all_records();
    assert_eq!(records[0].tool_name, "tool_a");
    assert_eq!(records[0].output_tokens_estimated, 50); // 200/4
}

#[test]
fn telemetry_collector_stats_for() {
    let c = TelemetryCollector::new(100);
    record_tool_telemetry(
        &c,
        "tool_a",
        Duration::from_millis(100),
        ExecutionStatus::Success,
        10,
        20,
        None,
    );
    record_tool_telemetry(
        &c,
        "tool_a",
        Duration::from_millis(200),
        ExecutionStatus::Failure,
        10,
        0,
        Some("err".into()),
    );
    let stats = c.stats_for("tool_a").unwrap();
    assert_eq!(stats.call_count, 2);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failure_count, 1);
    assert_eq!(stats.success_rate(), 0.5);
}

#[test]
fn telemetry_collector_stats_none_for_unknown() {
    let c = TelemetryCollector::new(100);
    assert!(c.stats_for("unknown").is_none());
}

#[test]
fn telemetry_collector_all_stats() {
    let c = TelemetryCollector::new(100);
    record_tool_telemetry(
        &c,
        "a",
        Duration::from_millis(10),
        ExecutionStatus::Success,
        1,
        1,
        None,
    );
    record_tool_telemetry(
        &c,
        "b",
        Duration::from_millis(10),
        ExecutionStatus::Success,
        1,
        1,
        None,
    );
    assert_eq!(c.all_stats().len(), 2);
}

#[test]
fn telemetry_collector_ring_buffer() {
    let c = TelemetryCollector::new(2);
    for i in 0..5 {
        record_tool_telemetry(
            &c,
            &format!("tool_{i}"),
            Duration::from_millis(1),
            ExecutionStatus::Success,
            1,
            1,
            None,
        );
    }
    assert_eq!(c.record_count(), 2);
}

#[test]
fn telemetry_collector_clear() {
    let c = TelemetryCollector::new(100);
    record_tool_telemetry(
        &c,
        "a",
        Duration::from_millis(1),
        ExecutionStatus::Success,
        1,
        1,
        None,
    );
    c.clear();
    assert_eq!(c.record_count(), 0);
    assert!(c.all_stats().is_empty());
}

// =========================================================================
// ToolStats
// =========================================================================

#[test]
fn tool_stats_avg_duration_empty() {
    let stats = ToolStats::default();
    assert!(stats.avg_duration().is_none());
}

#[test]
fn tool_stats_success_rate_empty() {
    let stats = ToolStats::default();
    assert_eq!(stats.success_rate(), 0.0);
}

#[test]
fn tool_stats_min_max_duration() {
    let c = TelemetryCollector::new(100);
    record_tool_telemetry(
        &c,
        "t",
        Duration::from_millis(10),
        ExecutionStatus::Success,
        1,
        1,
        None,
    );
    record_tool_telemetry(
        &c,
        "t",
        Duration::from_millis(50),
        ExecutionStatus::Success,
        1,
        1,
        None,
    );
    let stats = c.stats_for("t").unwrap();
    assert_eq!(stats.min_duration, Some(Duration::from_millis(10)));
    assert_eq!(stats.max_duration, Some(Duration::from_millis(50)));
}

#[test]
fn execution_status_display() {
    assert_eq!(format!("{}", ExecutionStatus::Success), "success");
    assert_eq!(format!("{}", ExecutionStatus::Failure), "failure");
}

// =========================================================================
// RetryPolicy
// =========================================================================

#[test]
fn retry_policy_default() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.backoff_ms, 100);
    assert!(policy.exponential);
    assert!(policy.fallback_tool.is_none());
}

#[test]
fn retry_policy_no_retry() {
    let policy = RetryPolicy::no_retry();
    assert_eq!(policy.max_retries, 0);
    assert!(!policy.exponential);
}

#[test]
fn retry_policy_fixed() {
    let policy = RetryPolicy::fixed(5, 200);
    assert_eq!(policy.max_retries, 5);
    assert_eq!(policy.backoff_ms, 200);
    assert!(!policy.exponential);
}

#[test]
fn retry_policy_with_fallback() {
    let policy = RetryPolicy::fixed(2, 100).with_fallback("backup_tool");
    assert_eq!(policy.fallback_tool, Some("backup_tool".to_string()));
}

// =========================================================================
// ConfirmationGate
// =========================================================================

#[test]
fn confirmation_gate_auto_approve_safe_tool() {
    let gate = ConfirmationGate::auto_approve("session-1");
    let outcome = gate.confirm("read_data", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
}

#[test]
fn confirmation_gate_auto_approve_dangerous_tool() {
    let gate = ConfirmationGate::auto_approve("session-1");
    let outcome = gate.confirm("delete_everything", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
}

#[test]
fn confirmation_gate_auto_deny_dangerous() {
    let engine = RiskEngine::with_defaults();
    let session = SessionPermissions::new("s1");
    let audit = AuditLog::default();
    let registry = ApprovalRegistry::new();
    let gate = ConfirmationGate::new(engine, session, audit, registry, Box::new(AutoDenyPrompter));
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
}

#[test]
fn confirmation_gate_session_always_allow_remembered() {
    let gate = ConfirmationGate::auto_approve("s1");
    gate.confirm("delete_user", &json!({}));
    gate.session().set_always_allow("delete_user");
    let outcome = gate.confirm("delete_user", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
}

#[test]
fn confirmation_gate_session_always_deny_remembered() {
    let gate = ConfirmationGate::auto_approve("s1");
    gate.session().set_always_deny("exec_cmd");
    let outcome = gate.confirm("exec_cmd", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
}

#[test]
fn confirmation_gate_audit_log_populated() {
    let gate = ConfirmationGate::auto_approve("s1");
    gate.confirm("read_file", &json!({}));
    assert_eq!(gate.audit_log().len(), 1);
}

#[test]
fn confirmation_gate_accessors() {
    let gate = ConfirmationGate::auto_approve("s1");
    let _ = gate.risk_engine().rules();
    assert_eq!(gate.session().session_id(), "s1");
    let _ = gate.registry();
}

#[test]
fn confirmation_gate_risk_engine_mut() {
    let mut gate = ConfirmationGate::auto_approve("s1");
    gate.risk_engine_mut()
        .add_rule(RiskRule::new("custom_*", RiskLevel::Warning, "custom"));
    let a = gate.risk_engine().assess("custom_tool");
    assert_eq!(a.level, RiskLevel::Warning);
}

#[test]
fn confirmation_outcome_eq() {
    assert_eq!(ConfirmationOutcome::Allowed, ConfirmationOutcome::Allowed);
    assert_ne!(ConfirmationOutcome::Allowed, ConfirmationOutcome::Blocked);
}

// =========================================================================
// Moved from inline tests — approval.rs
// =========================================================================

/// Custom prompter that records calls and returns a fixed response.
struct RecordingPrompter {
    response: PromptResponse,
    call_count: Arc<RwLock<u32>>,
}

impl RecordingPrompter {
    fn new(response: PromptResponse) -> Self {
        Self {
            response,
            call_count: Arc::new(RwLock::new(0)),
        }
    }
}

impl ApprovalPrompter for RecordingPrompter {
    fn prompt(
        &self,
        _tool_name: &str,
        _arguments: &serde_json::Value,
        _risk_level: RiskLevel,
    ) -> PromptResponse {
        *self.call_count.write().unwrap() += 1;
        self.response.clone()
    }
}

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
fn test_prompt_response_display() {
    assert_eq!(PromptResponse::Yes.to_string(), "yes");
    assert_eq!(PromptResponse::No.to_string(), "no");
    assert_eq!(PromptResponse::AlwaysAllow.to_string(), "always-allow");
    assert_eq!(PromptResponse::AlwaysDeny.to_string(), "always-deny");
}

#[test]
fn test_confirmation_gate_safe_auto_approve() {
    let gate = ConfirmationGate::auto_approve("test-session");
    let outcome = gate.confirm("read_file", &json!({"path": "/tmp/test"}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
    assert_eq!(gate.audit_log().len(), 1);
}

#[test]
fn test_confirmation_gate_dangerous_with_auto_approve_prompter() {
    let gate = ConfirmationGate::auto_approve("test-session");
    let outcome = gate.confirm("delete_file", &json!({"path": "/tmp/test"}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
}

#[test]
fn test_confirmation_gate_dangerous_with_auto_deny_prompter() {
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(AutoDenyPrompter),
    );
    let outcome = gate.confirm("delete_file", &json!({"path": "/tmp/test"}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
}

#[test]
fn test_confirmation_gate_session_always_allow() {
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(AutoDenyPrompter),
    );
    gate.session().set_always_allow("delete_file");
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
}

#[test]
fn test_confirmation_gate_session_always_deny() {
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(AutoApprovePrompter),
    );
    gate.session().set_always_deny("delete_file");
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
}

#[test]
fn test_confirmation_gate_audit_trail() {
    let gate = ConfirmationGate::auto_approve("test-session");
    gate.confirm("read_file", &json!({}));
    gate.confirm("delete_file", &json!({}));
    gate.confirm("write_config", &json!({}));
    assert_eq!(gate.audit_log().len(), 3);
    let entries = gate.audit_log().entries_for_session("test-session");
    assert_eq!(entries.len(), 3);
}

#[test]
fn test_always_allow_skips_future_prompts() {
    let prompter = RecordingPrompter::new(PromptResponse::AlwaysAllow);
    let call_count = prompter.call_count.clone();
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(prompter),
    );
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
    assert_eq!(*call_count.read().unwrap(), 1);
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
    assert_eq!(*call_count.read().unwrap(), 1); // still 1
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
fn test_approval_registry_default() {
    let registry = ApprovalRegistry::default();
    assert_eq!(registry.pending().len(), 0);
}

#[test]
fn test_approval_gate_request_approval() {
    let registry = ApprovalRegistry::new();
    let mut gate = ApprovalGate::new(registry);
    gate.require_approval_for("dangerous_tool");
    let id = gate.request_approval("dangerous_tool", json!({"arg": "val"}));
    let req = gate.registry().get(&id).unwrap();
    assert_eq!(req.tool_name, "dangerous_tool");
    assert_eq!(req.state, ApprovalState::Pending);
}

#[test]
fn test_approval_gate_request_without_requirement() {
    let registry = ApprovalRegistry::new();
    let gate = ApprovalGate::new(registry);
    let id = gate.request_approval("unregistered_tool", json!({}));
    assert!(!id.is_empty());
}

#[test]
fn test_approval_gate_duplicate_require() {
    let registry = ApprovalRegistry::new();
    let mut gate = ApprovalGate::new(registry);
    gate.require_approval_for("tool_a");
    gate.require_approval_for("tool_a");
    gate.require_approval_for("tool_b");
    assert!(gate.needs_approval("tool_a"));
    assert!(gate.needs_approval("tool_b"));
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
fn test_approval_error_display() {
    let err = ApprovalError::NotFound("abc123".to_string());
    assert!(err
        .to_string()
        .contains("Approval request not found: abc123"));
    let err = ApprovalError::InvalidTransition {
        id: "abc123".to_string(),
        from: ApprovalState::Pending,
        to: ApprovalState::Executed,
    };
    assert!(err.to_string().contains("Pending"));
    assert!(err.to_string().contains("Executed"));
}

#[test]
fn test_confirmation_gate_always_deny_skips_future_prompts() {
    let prompter = RecordingPrompter::new(PromptResponse::AlwaysDeny);
    let call_count = prompter.call_count.clone();
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(prompter),
    );
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
    assert_eq!(*call_count.read().unwrap(), 1);
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
    assert_eq!(*call_count.read().unwrap(), 1);
}

#[test]
fn test_confirmation_gate_accessors() {
    let gate = ConfirmationGate::auto_approve("test-session");
    assert_eq!(gate.session().session_id(), "test-session");
    assert_eq!(gate.audit_log().len(), 0);
    assert!(gate.risk_engine().rules().len() >= 10);
}

#[test]
fn test_confirmation_gate_risk_engine_mut() {
    let mut gate = ConfirmationGate::auto_approve("test-session");
    gate.risk_engine_mut()
        .set_override("special_tool", RiskLevel::Safe);
    let assessment = gate.risk_engine().assess("special_tool");
    assert_eq!(assessment.level, RiskLevel::Safe);
}

#[test]
fn test_auto_approve_prompter() {
    let prompter = AutoApprovePrompter;
    let response = prompter.prompt("any_tool", &json!({}), RiskLevel::Dangerous);
    assert_eq!(response, PromptResponse::Yes);
}

#[test]
fn test_auto_deny_prompter() {
    let prompter = AutoDenyPrompter;
    let response = prompter.prompt("any_tool", &json!({}), RiskLevel::Dangerous);
    assert_eq!(response, PromptResponse::No);
}

#[test]
fn test_confirmation_outcome_equality() {
    assert_eq!(ConfirmationOutcome::Blocked, ConfirmationOutcome::Blocked);
    assert_ne!(ConfirmationOutcome::Allowed, ConfirmationOutcome::Blocked);
}

#[test]
fn test_one_time_yes_approval() {
    let prompter = RecordingPrompter::new(PromptResponse::Yes);
    let call_count = prompter.call_count.clone();
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(prompter),
    );
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
    assert_eq!(*call_count.read().unwrap(), 1);
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
    assert_eq!(*call_count.read().unwrap(), 2);
}

#[test]
fn test_one_time_no_denial() {
    let prompter = RecordingPrompter::new(PromptResponse::No);
    let call_count = prompter.call_count.clone();
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(prompter),
    );
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
    assert_eq!(*call_count.read().unwrap(), 1);
    let outcome = gate.confirm("delete_file", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);
    assert_eq!(*call_count.read().unwrap(), 2);
}

#[test]
fn test_approval_state_serde_roundtrip() {
    for state in [
        ApprovalState::Pending,
        ApprovalState::Approved,
        ApprovalState::Denied,
        ApprovalState::Executed,
        ApprovalState::Skipped,
    ] {
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ApprovalState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, state);
    }
}

#[test]
fn test_prompt_response_serde_roundtrip() {
    for resp in [
        PromptResponse::Yes,
        PromptResponse::No,
        PromptResponse::AlwaysAllow,
        PromptResponse::AlwaysDeny,
    ] {
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: PromptResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, resp);
    }
}

#[test]
fn test_confirmation_outcome_serde_roundtrip() {
    for outcome in [ConfirmationOutcome::Allowed, ConfirmationOutcome::Blocked] {
        let json = serde_json::to_string(&outcome).unwrap();
        let deserialized: ConfirmationOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, outcome);
    }
}

#[test]
fn test_approval_request_serde_roundtrip() {
    let now = SystemTime::now();
    let req = ApprovalRequest {
        id: "req-1".to_string(),
        tool_name: "delete_file".to_string(),
        arguments: json!({"path": "/tmp/test"}),
        state: ApprovalState::Approved,
        created_at: now,
        updated_at: now,
        reason: Some("looks safe".to_string()),
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: ApprovalRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "req-1");
    assert_eq!(deserialized.tool_name, "delete_file");
    assert_eq!(deserialized.state, ApprovalState::Approved);
    assert_eq!(deserialized.reason.as_deref(), Some("looks safe"));
}

#[test]
fn test_approval_error_debug() {
    let err = ApprovalError::NotFound("id-1".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("NotFound"));
    let err = ApprovalError::InvalidTransition {
        id: "id-1".to_string(),
        from: ApprovalState::Pending,
        to: ApprovalState::Executed,
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("InvalidTransition"));
}

#[test]
fn test_confirmation_outcome_debug() {
    let debug = format!("{:?}", ConfirmationOutcome::Allowed);
    assert!(debug.contains("Allowed"));
    let debug = format!("{:?}", ConfirmationOutcome::Blocked);
    assert!(debug.contains("Blocked"));
}

#[test]
fn test_confirmation_gate_warning_level_prompts() {
    let prompter = RecordingPrompter::new(PromptResponse::Yes);
    let call_count = prompter.call_count.clone();
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        SessionPermissions::new("test-session"),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(prompter),
    );
    let outcome = gate.confirm("write_config", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);
    assert_eq!(*call_count.read().unwrap(), 1);
}

#[test]
fn test_auto_approve_prompter_default() {
    let prompter = AutoApprovePrompter::default();
    let resp = prompter.prompt("tool", &json!({}), RiskLevel::Warning);
    assert_eq!(resp, PromptResponse::Yes);
}

#[test]
fn test_auto_deny_prompter_default() {
    let prompter = AutoDenyPrompter::default();
    let resp = prompter.prompt("tool", &json!({}), RiskLevel::Warning);
    assert_eq!(resp, PromptResponse::No);
}

#[test]
fn test_approval_registry_clone() {
    let registry = ApprovalRegistry::new();
    let id = registry.submit("tool", json!({}));
    let cloned = registry.clone();
    assert!(cloned.get(&id).is_some());
}

// =========================================================================
// Moved from inline tests — audit.rs
// =========================================================================

fn make_audit_entry(tool: &str, decision: AuditDecision, session: &str) -> AuditEntry {
    AuditEntry {
        id: uuid::Uuid::new_v4().to_string(),
        approval_id: "approval-1".to_string(),
        tool_name: tool.to_string(),
        arguments: json!({"path": "/tmp/test"}),
        risk_level: RiskLevel::Dangerous,
        decision,
        reason: None,
        session_id: session.to_string(),
        timestamp: SystemTime::now(),
    }
}

#[test]
fn test_record_and_retrieve() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry(
        "delete_file",
        AuditDecision::Approved,
        "s1",
    ));
    log.record(make_audit_entry("write_file", AuditDecision::Denied, "s1"));
    assert_eq!(log.len(), 2);
    assert!(!log.is_empty());
}

#[test]
fn test_entries_for_tool() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry(
        "delete_file",
        AuditDecision::Approved,
        "s1",
    ));
    log.record(make_audit_entry("write_file", AuditDecision::Denied, "s1"));
    log.record(make_audit_entry("delete_file", AuditDecision::Denied, "s2"));
    let entries = log.entries_for_tool("delete_file");
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_entries_for_session() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry(
        "delete_file",
        AuditDecision::Approved,
        "s1",
    ));
    log.record(make_audit_entry("write_file", AuditDecision::Denied, "s2"));
    let entries = log.entries_for_session("s1");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tool_name, "delete_file");
}

#[test]
fn test_ring_buffer() {
    let log = AuditLog::new(3);
    for _ in 0..5 {
        log.record(make_audit_entry("tool", AuditDecision::Approved, "s1"));
    }
    assert_eq!(log.len(), 3);
}

#[test]
fn test_clear() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry("tool", AuditDecision::Approved, "s1"));
    log.clear();
    assert!(log.is_empty());
}

#[test]
fn test_log_decision_convenience() {
    let log = AuditLog::new(100);
    let id = log.log_decision(
        "approval-42",
        "delete_file",
        json!({"path": "/etc/important"}),
        RiskLevel::Dangerous,
        AuditDecision::Denied,
        Some("too risky".to_string()),
        "session-1",
    );
    assert!(!id.is_empty());
    assert_eq!(log.len(), 1);
    let entry = &log.entries()[0];
    assert_eq!(entry.decision, AuditDecision::Denied);
    assert_eq!(entry.reason.as_deref(), Some("too risky"));
}

#[test]
fn test_json_round_trip() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry(
        "delete_file",
        AuditDecision::Approved,
        "s1",
    ));
    log.record(make_audit_entry("write_file", AuditDecision::Denied, "s1"));
    let json = log.to_json().unwrap();
    let log2 = AuditLog::new(100);
    let count = log2.load_from_json(&json).unwrap();
    assert_eq!(count, 2);
    assert_eq!(log2.len(), 2);
}

#[test]
fn test_audit_decision_display() {
    assert_eq!(AuditDecision::Approved.to_string(), "approved");
    assert_eq!(AuditDecision::Denied.to_string(), "denied");
    assert_eq!(AuditDecision::AutoApproved.to_string(), "auto-approved");
    assert_eq!(
        AuditDecision::SafeAutoApproved.to_string(),
        "safe-auto-approved"
    );
}

#[test]
fn test_audit_log_default() {
    let log = AuditLog::default();
    assert!(log.is_empty());
    assert_eq!(log.len(), 0);
}

#[test]
fn test_entries_snapshot_is_independent() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry("tool", AuditDecision::Approved, "s1"));
    let entries = log.entries();
    assert_eq!(entries.len(), 1);
    log.record(make_audit_entry("tool2", AuditDecision::Denied, "s1"));
    assert_eq!(entries.len(), 1);
    assert_eq!(log.len(), 2);
}

#[test]
fn test_load_from_json_invalid() {
    let log = AuditLog::new(100);
    let result = log.load_from_json("not json at all");
    assert!(result.is_err());
}

#[test]
fn test_load_from_json_respects_max_entries() {
    let log = AuditLog::new(2);
    log.record(make_audit_entry("existing", AuditDecision::Approved, "s1"));
    let other_log = AuditLog::new(100);
    other_log.record(make_audit_entry("a", AuditDecision::Approved, "s2"));
    other_log.record(make_audit_entry("b", AuditDecision::Denied, "s2"));
    other_log.record(make_audit_entry("c", AuditDecision::Denied, "s2"));
    let json = other_log.to_json().unwrap();
    log.load_from_json(&json).unwrap();
    assert_eq!(log.len(), 2);
}

#[test]
fn test_audit_entry_serde_roundtrip() {
    let entry = make_audit_entry("test_tool", AuditDecision::Denied, "session-42");
    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: AuditEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool_name, "test_tool");
    assert_eq!(deserialized.decision, AuditDecision::Denied);
    assert_eq!(deserialized.session_id, "session-42");
}

#[test]
fn test_audit_log_to_json_empty() {
    let log = AuditLog::new(100);
    let json = log.to_json().unwrap();
    assert_eq!(json, "[]");
}

#[test]
fn test_audit_decision_serde_roundtrip() {
    for decision in [
        AuditDecision::Approved,
        AuditDecision::Denied,
        AuditDecision::AutoApproved,
        AuditDecision::SafeAutoApproved,
    ] {
        let json = serde_json::to_string(&decision).unwrap();
        let deserialized: AuditDecision = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, decision);
    }
}

#[test]
fn test_audit_log_clone_shares_data() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry("tool", AuditDecision::Approved, "s1"));
    let cloned = log.clone();
    assert_eq!(cloned.len(), 1);
    cloned.record(make_audit_entry("tool2", AuditDecision::Denied, "s1"));
    assert_eq!(log.len(), 2);
}

#[test]
fn test_audit_entry_debug() {
    let entry = make_audit_entry("tool", AuditDecision::Approved, "s1");
    let debug = format!("{:?}", entry);
    assert!(debug.contains("tool"));
    assert!(debug.contains("Approved"));
}

#[test]
fn test_audit_log_load_empty_json_array() {
    let log = AuditLog::new(100);
    let count = log.load_from_json("[]").unwrap();
    assert_eq!(count, 0);
    assert!(log.is_empty());
}

#[test]
fn test_entries_for_tool_none_found() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry("tool_a", AuditDecision::Approved, "s1"));
    let entries = log.entries_for_tool("tool_b");
    assert!(entries.is_empty());
}

#[test]
fn test_entries_for_session_none_found() {
    let log = AuditLog::new(100);
    log.record(make_audit_entry("tool", AuditDecision::Approved, "s1"));
    let entries = log.entries_for_session("s2");
    assert!(entries.is_empty());
}

#[test]
fn test_log_decision_without_reason() {
    let log = AuditLog::new(100);
    let id = log.log_decision(
        "approval-1",
        "read_file",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::SafeAutoApproved,
        None,
        "session-1",
    );
    assert!(!id.is_empty());
    let entries = log.entries();
    assert!(entries[0].reason.is_none());
}

// =========================================================================
// Moved from inline tests — risk.rs
// =========================================================================

#[test]
fn test_risk_level_ordering() {
    assert!(RiskLevel::Safe < RiskLevel::Warning);
    assert!(RiskLevel::Warning < RiskLevel::Dangerous);
}

#[test]
fn test_risk_level_display() {
    assert_eq!(RiskLevel::Safe.to_string(), "safe");
    assert_eq!(RiskLevel::Warning.to_string(), "warning");
    assert_eq!(RiskLevel::Dangerous.to_string(), "dangerous");
}

#[test]
fn test_risk_engine_no_rules() {
    let engine = RiskEngine::new();
    let a = engine.assess("unknown_tool");
    assert_eq!(a.level, RiskLevel::Safe);
    assert!(!a.requires_approval);
    assert!(a.matched_rule.is_none());
}

#[test]
fn test_risk_engine_custom_rule() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new(
        "delete_*",
        RiskLevel::Dangerous,
        "irreversible",
    ));
    let a = engine.assess("delete_everything");
    assert_eq!(a.level, RiskLevel::Dangerous);
    assert!(a.requires_approval);
    assert!(a.matched_rule.is_some());
}

#[test]
fn test_override_takes_priority_over_rules() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new(
        "delete_*",
        RiskLevel::Dangerous,
        "irreversible",
    ));
    engine.set_override("delete_temp", RiskLevel::Safe);
    let a = engine.assess("delete_temp");
    assert_eq!(a.level, RiskLevel::Safe);
    assert!(a.matched_rule.is_none());
    let a = engine.assess("delete_database");
    assert_eq!(a.level, RiskLevel::Dangerous);
}

#[test]
fn test_engine_first_rule_wins() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new(
        "delete_*",
        RiskLevel::Dangerous,
        "first rule",
    ));
    engine.add_rule(RiskRule::new("delete_*", RiskLevel::Safe, "second rule"));
    let a = engine.assess("delete_file");
    assert_eq!(a.level, RiskLevel::Dangerous);
}

#[test]
fn test_engine_with_defaults() {
    let engine = RiskEngine::with_defaults();
    assert_eq!(engine.assess("delete_file").level, RiskLevel::Dangerous);
    assert_eq!(engine.assess("exec_command").level, RiskLevel::Dangerous);
    assert_eq!(engine.assess("write_config").level, RiskLevel::Warning);
    assert_eq!(engine.assess("read_file").level, RiskLevel::Safe);
    assert_eq!(engine.assess("list_users").level, RiskLevel::Safe);
}

#[test]
fn test_remove_override() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new("delete_*", RiskLevel::Dangerous, "test"));
    engine.set_override("delete_temp", RiskLevel::Safe);
    assert_eq!(engine.assess("delete_temp").level, RiskLevel::Safe);
    engine.remove_override("delete_temp");
    assert_eq!(engine.assess("delete_temp").level, RiskLevel::Dangerous);
}

#[test]
fn test_set_default_level() {
    let mut engine = RiskEngine::new();
    engine.set_default_level(RiskLevel::Warning);
    let a = engine.assess("unknown_tool");
    assert_eq!(a.level, RiskLevel::Warning);
}

#[test]
fn test_risk_assessment_requires_approval() {
    let engine = RiskEngine::with_defaults();
    let a = engine.assess("delete_file");
    assert!(a.requires_approval);
    let a = engine.assess("write_config");
    assert!(!a.requires_approval);
    let a = engine.assess("read_file");
    assert!(!a.requires_approval);
}

#[test]
fn test_risk_assessment_summary_contains_tool_name() {
    let engine = RiskEngine::with_defaults();
    let a = engine.assess("delete_file");
    assert!(a.summary.contains("delete_file"));
}

#[test]
fn test_risk_rule_new() {
    let rule = RiskRule::new("exec_*", RiskLevel::Dangerous, "arbitrary execution");
    assert_eq!(rule.pattern, "exec_*");
    assert_eq!(rule.level, RiskLevel::Dangerous);
    assert_eq!(rule.reason, "arbitrary execution");
}

#[test]
fn test_engine_rules_accessor() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new("r1", RiskLevel::Safe, "test"));
    engine.add_rule(RiskRule::new("r2", RiskLevel::Warning, "test"));
    assert_eq!(engine.rules().len(), 2);
}

#[test]
fn test_risk_level_serde_roundtrip() {
    let level = RiskLevel::Warning;
    let json = serde_json::to_string(&level).unwrap();
    let deserialized: RiskLevel = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, RiskLevel::Warning);
}

#[test]
fn test_default_level_dangerous_requires_approval() {
    let mut engine = RiskEngine::new();
    engine.set_default_level(RiskLevel::Dangerous);
    let a = engine.assess("any_tool");
    assert!(a.requires_approval);
}

#[test]
fn test_override_assessment_summary() {
    let mut engine = RiskEngine::new();
    engine.set_override("custom_tool", RiskLevel::Warning);
    let a = engine.assess("custom_tool");
    assert!(a.summary.contains("per-tool override"));
    assert!(a.matched_rule.is_none());
}

#[test]
fn test_with_defaults_covers_all_categories() {
    let engine = RiskEngine::with_defaults();
    assert_eq!(engine.assess("drop_table").level, RiskLevel::Dangerous);
    assert_eq!(engine.assess("shell_exec").level, RiskLevel::Dangerous);
    assert_eq!(engine.assess("rm_file").level, RiskLevel::Dangerous);
    assert_eq!(engine.assess("format_disk").level, RiskLevel::Dangerous);
    assert_eq!(engine.assess("update_config").level, RiskLevel::Warning);
    assert_eq!(engine.assess("create_user").level, RiskLevel::Warning);
    assert_eq!(engine.assess("send_email").level, RiskLevel::Warning);
    assert_eq!(engine.assess("publish_package").level, RiskLevel::Warning);
    assert_eq!(engine.assess("get_status").level, RiskLevel::Safe);
    assert_eq!(engine.assess("search_index").level, RiskLevel::Safe);
}

#[test]
fn test_risk_rule_serde_roundtrip() {
    let rule = RiskRule::new("delete_*", RiskLevel::Dangerous, "irreversible");
    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: RiskRule = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.pattern, "delete_*");
    assert_eq!(deserialized.level, RiskLevel::Dangerous);
    assert_eq!(deserialized.reason, "irreversible");
}

#[test]
fn test_risk_assessment_serde_roundtrip() {
    let engine = RiskEngine::with_defaults();
    let assessment = engine.assess("delete_file");
    let json = serde_json::to_string(&assessment).unwrap();
    let deserialized: RiskAssessment = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool_name, "delete_file");
    assert_eq!(deserialized.level, RiskLevel::Dangerous);
    assert!(deserialized.requires_approval);
    assert!(deserialized.matched_rule.is_some());
}

#[test]
fn test_risk_assessment_no_rule_serde() {
    let engine = RiskEngine::new();
    let assessment = engine.assess("unknown_tool");
    let json = serde_json::to_string(&assessment).unwrap();
    let deserialized: RiskAssessment = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.level, RiskLevel::Safe);
    assert!(deserialized.matched_rule.is_none());
}

#[test]
fn test_risk_engine_clone() {
    let mut engine = RiskEngine::with_defaults();
    engine.set_override("custom_tool", RiskLevel::Warning);
    let cloned = engine.clone();
    assert_eq!(cloned.assess("custom_tool").level, RiskLevel::Warning);
    assert_eq!(cloned.assess("delete_file").level, RiskLevel::Dangerous);
    assert_eq!(cloned.rules().len(), engine.rules().len());
}

#[test]
fn test_risk_rule_debug() {
    let rule = RiskRule::new("delete_*", RiskLevel::Dangerous, "test");
    let debug = format!("{:?}", rule);
    assert!(debug.contains("delete_*"));
    assert!(debug.contains("Dangerous"));
}

#[test]
fn test_risk_assessment_debug() {
    let engine = RiskEngine::new();
    let assessment = engine.assess("tool");
    let debug = format!("{:?}", assessment);
    assert!(debug.contains("tool"));
}

#[test]
fn test_remove_override_nonexistent() {
    let mut engine = RiskEngine::new();
    let result = engine.remove_override("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_remove_override_returns_old_level() {
    let mut engine = RiskEngine::new();
    engine.set_override("tool", RiskLevel::Warning);
    let removed = engine.remove_override("tool");
    assert_eq!(removed, Some(RiskLevel::Warning));
}

#[test]
fn test_risk_level_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(RiskLevel::Safe);
    set.insert(RiskLevel::Warning);
    set.insert(RiskLevel::Dangerous);
    set.insert(RiskLevel::Safe);
    assert_eq!(set.len(), 3);
}

#[test]
fn test_risk_level_copy() {
    let level = RiskLevel::Dangerous;
    let copy = level;
    assert_eq!(level, copy);
}

#[test]
fn test_risk_engine_default() {
    let engine = RiskEngine::default();
    assert!(engine.rules().is_empty());
    assert_eq!(engine.assess("anything").level, RiskLevel::Safe);
}

#[test]
fn test_assessment_summary_for_default() {
    let engine = RiskEngine::new();
    let a = engine.assess("custom_tool");
    assert!(a.summary.contains("no matching rule"));
    assert!(a.summary.contains("custom_tool"));
}

#[test]
fn test_assessment_summary_for_rule_match() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new(
        "write_*",
        RiskLevel::Warning,
        "modifies state",
    ));
    let a = engine.assess("write_file");
    assert!(a.summary.contains("write_*"));
    assert!(a.summary.contains("modifies state"));
}

// =========================================================================
// Moved from inline tests — session.rs
// =========================================================================

#[test]
fn test_always_allow() {
    let session = SessionPermissions::new("test-session");
    session.set_always_allow("delete_file");
    let status = session.check("delete_file");
    assert_eq!(status, Some(PermissionStatus::AlwaysAllow));
}

#[test]
fn test_always_deny() {
    let session = SessionPermissions::new("test-session");
    session.set_always_deny("delete_file");
    let status = session.check("delete_file");
    assert_eq!(status, Some(PermissionStatus::AlwaysDeny));
}

#[test]
fn test_no_permission_returns_none() {
    let session = SessionPermissions::new("test-session");
    assert_eq!(session.check("tool"), None);
}

#[test]
fn test_revoke_nonexistent() {
    let session = SessionPermissions::new("test-session");
    assert!(!session.revoke("nonexistent"));
}

#[test]
fn test_one_time_approval_history() {
    let session = SessionPermissions::new("test-session");
    assert!(!session.was_approved_before("tool"));
    session.record_one_time_approval("tool");
    assert!(session.was_approved_before("tool"));
}

#[test]
fn test_always_allow_counts_as_approved() {
    let session = SessionPermissions::new("test-session");
    session.set_always_allow("tool");
    assert!(session.was_approved_before("tool"));
}

#[test]
fn test_clear_session() {
    let session = SessionPermissions::new("test-session");
    session.set_always_allow("tool");
    session.record_one_time_approval("other_tool");
    session.clear();
    assert_eq!(session.check("tool"), None);
    assert!(!session.was_approved_before("other_tool"));
    assert!(session.all_permissions().is_empty());
}

#[test]
fn test_session_id() {
    let session = SessionPermissions::new("my-session-42");
    assert_eq!(session.session_id(), "my-session-42");
}

#[test]
fn test_permission_status_display() {
    assert_eq!(PermissionStatus::AlwaysAllow.to_string(), "always-allow");
    assert_eq!(PermissionStatus::AlwaysDeny.to_string(), "always-deny");
}

#[test]
fn test_approval_history() {
    let session = SessionPermissions::new("test");
    session.record_one_time_approval("tool_a");
    session.record_one_time_approval("tool_b");
    let history = session.approval_history();
    assert_eq!(history.len(), 2);
    assert!(history.contains(&"tool_a".to_string()));
    assert!(history.contains(&"tool_b".to_string()));
}

#[test]
fn test_overwrite_always_allow_with_always_deny() {
    let session = SessionPermissions::new("test");
    session.set_always_allow("tool");
    assert_eq!(session.check("tool"), Some(PermissionStatus::AlwaysAllow));
    session.set_always_deny("tool");
    assert_eq!(session.check("tool"), Some(PermissionStatus::AlwaysDeny));
}

#[test]
fn test_all_permissions_list() {
    let session = SessionPermissions::new("test");
    session.set_always_allow("tool_a");
    session.set_always_deny("tool_b");
    let perms = session.all_permissions();
    assert_eq!(perms.len(), 2);
}

#[test]
fn test_was_approved_before_with_deny() {
    let session = SessionPermissions::new("test");
    session.set_always_deny("tool");
    assert!(!session.was_approved_before("tool"));
}

#[test]
fn test_permission_record_fields() {
    let session = SessionPermissions::new("test");
    session.set_always_allow("tool");
    let perms = session.all_permissions();
    let record = &perms[0];
    assert_eq!(record.tool_name, "tool");
    assert_eq!(record.status, PermissionStatus::AlwaysAllow);
    assert_eq!(record.usage_count, 0);
}

#[test]
fn test_permission_status_serde() {
    let status = PermissionStatus::AlwaysAllow;
    let json = serde_json::to_string(&status).unwrap();
    let deserialized: PermissionStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, PermissionStatus::AlwaysAllow);
}

#[test]
fn test_permission_status_always_deny_serde() {
    let status = PermissionStatus::AlwaysDeny;
    let json = serde_json::to_string(&status).unwrap();
    let deserialized: PermissionStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, PermissionStatus::AlwaysDeny);
}

#[test]
fn test_permission_record_serde_roundtrip() {
    let record = PermissionRecord {
        tool_name: "tool".to_string(),
        status: PermissionStatus::AlwaysAllow,
        granted_at: SystemTime::now(),
        usage_count: 5,
    };
    let json = serde_json::to_string(&record).unwrap();
    let deserialized: PermissionRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool_name, "tool");
    assert_eq!(deserialized.status, PermissionStatus::AlwaysAllow);
    assert_eq!(deserialized.usage_count, 5);
}

#[test]
fn test_session_permissions_clone_shares_data() {
    let session = SessionPermissions::new("test");
    session.set_always_allow("tool");
    let cloned = session.clone();
    assert!(cloned.check("tool").is_some());
    assert_eq!(cloned.session_id(), "test");
}

#[test]
fn test_approval_history_empty() {
    let session = SessionPermissions::new("test");
    assert!(session.approval_history().is_empty());
}

#[test]
fn test_record_one_time_approval_idempotent() {
    let session = SessionPermissions::new("test");
    session.record_one_time_approval("tool");
    session.record_one_time_approval("tool");
    assert_eq!(session.approval_history().len(), 1);
}

#[test]
fn test_permission_status_debug() {
    let debug = format!("{:?}", PermissionStatus::AlwaysAllow);
    assert!(debug.contains("AlwaysAllow"));
    let debug = format!("{:?}", PermissionStatus::AlwaysDeny);
    assert!(debug.contains("AlwaysDeny"));
}

#[test]
fn test_permission_record_debug() {
    let record = PermissionRecord {
        tool_name: "tool".to_string(),
        status: PermissionStatus::AlwaysAllow,
        granted_at: SystemTime::now(),
        usage_count: 0,
    };
    let debug = format!("{:?}", record);
    assert!(debug.contains("tool"));
    assert!(debug.contains("AlwaysAllow"));
}

// =========================================================================
// Moved from inline tests — retry.rs
// =========================================================================

#[test]
fn test_retry_policy_defaults() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.backoff_ms, 100);
    assert!(policy.fallback_tool.is_none());
    assert!(policy.exponential);
}

#[test]
fn test_retry_policy_no_retry() {
    let policy = RetryPolicy::no_retry();
    assert_eq!(policy.max_retries, 0);
}

#[test]
fn test_retry_policy_fixed() {
    let policy = RetryPolicy::fixed(2, 50);
    assert_eq!(policy.max_retries, 2);
    assert_eq!(policy.backoff_ms, 50);
    assert!(!policy.exponential);
}

#[test]
fn test_retry_policy_with_fallback() {
    let policy = RetryPolicy::default().with_fallback("backup_tool");
    assert_eq!(policy.fallback_tool.as_deref(), Some("backup_tool"));
}

#[test]
fn test_delay_exponential() {
    let policy = RetryPolicy {
        max_retries: 3,
        backoff_ms: 100,
        fallback_tool: None,
        exponential: true,
    };
    assert_eq!(policy.delay_for(0), Duration::from_millis(100));
    assert_eq!(policy.delay_for(1), Duration::from_millis(200));
    assert_eq!(policy.delay_for(2), Duration::from_millis(400));
}

#[test]
fn test_delay_fixed() {
    let policy = RetryPolicy::fixed(3, 50);
    assert_eq!(policy.delay_for(0), Duration::from_millis(50));
    assert_eq!(policy.delay_for(1), Duration::from_millis(50));
}

#[test]
fn test_delay_zero_backoff() {
    let policy = RetryPolicy::no_retry();
    assert_eq!(policy.delay_for(0), Duration::ZERO);
}

#[test]
fn test_delay_exponential_high_attempt() {
    let policy = RetryPolicy {
        max_retries: 20,
        backoff_ms: 100,
        fallback_tool: None,
        exponential: true,
    };
    let delay_10 = policy.delay_for(10);
    let delay_11 = policy.delay_for(11);
    assert_eq!(delay_10, Duration::from_millis(100 * 1024));
    assert_eq!(delay_11, Duration::from_millis(100 * 1024));
}

#[test]
fn test_retry_policy_default_values() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.backoff_ms, 100);
    assert!(policy.exponential);
    assert!(policy.fallback_tool.is_none());
}

#[test]
fn test_retry_policy_no_retry_all_fields() {
    let policy = RetryPolicy::no_retry();
    assert_eq!(policy.max_retries, 0);
    assert_eq!(policy.backoff_ms, 0);
    assert!(!policy.exponential);
    assert!(policy.fallback_tool.is_none());
}

#[test]
fn test_retry_policy_with_fallback_chaining() {
    let policy = RetryPolicy::fixed(2, 50).with_fallback("backup");
    assert_eq!(policy.max_retries, 2);
    assert_eq!(policy.backoff_ms, 50);
    assert!(!policy.exponential);
    assert_eq!(policy.fallback_tool.as_deref(), Some("backup"));
}

#[test]
fn test_delay_for_first_attempt_exponential() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.delay_for(0), Duration::from_millis(100));
}

#[test]
fn test_delay_for_third_attempt_exponential() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.delay_for(3), Duration::from_millis(800));
}

#[test]
fn test_retry_policy_debug() {
    let policy = RetryPolicy::default();
    let debug = format!("{:?}", policy);
    assert!(debug.contains("max_retries"));
    assert!(debug.contains("backoff_ms"));
    assert!(debug.contains("exponential"));
}

#[test]
fn test_retry_policy_clone() {
    let policy = RetryPolicy::default().with_fallback("backup");
    let cloned = policy.clone();
    assert_eq!(cloned.max_retries, policy.max_retries);
    assert_eq!(cloned.backoff_ms, policy.backoff_ms);
    assert_eq!(cloned.exponential, policy.exponential);
    assert_eq!(cloned.fallback_tool, policy.fallback_tool);
}

#[test]
fn test_tool_call_outcome_debug() {
    let outcome = ToolCallOutcome::Success(serde_json::json!({"result": "ok"}));
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Success"));
    let outcome = ToolCallOutcome::FallbackSuccess {
        fallback_tool: "backup".to_string(),
        result: serde_json::json!({"result": "fallback ok"}),
    };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("FallbackSuccess"));
    assert!(debug.contains("backup"));
    let outcome = ToolCallOutcome::Failed {
        attempts: 3,
        last_error: RegistryError::CallFailed("timeout".to_string()),
    };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Failed"));
    assert!(debug.contains("3"));
}

#[test]
fn test_delay_for_large_backoff_saturating() {
    let policy = RetryPolicy {
        max_retries: 5,
        backoff_ms: u64::MAX,
        fallback_tool: None,
        exponential: false,
    };
    let delay = policy.delay_for(0);
    assert_eq!(delay, Duration::from_millis(u64::MAX));
}

// =========================================================================
// Moved from inline tests — telemetry.rs
// =========================================================================

fn make_telemetry_record(tool: &str, ok: bool, dur_ms: u64) -> ToolTelemetryRecord {
    ToolTelemetryRecord {
        tool_name: tool.to_string(),
        duration: Duration::from_millis(dur_ms),
        status: if ok {
            ExecutionStatus::Success
        } else {
            ExecutionStatus::Failure
        },
        input_size_bytes: 100,
        output_size_bytes: if ok { 200 } else { 0 },
        error: if ok { None } else { Some("oops".into()) },
        output_tokens_estimated: if ok { 50 } else { 0 },
    }
}

#[test]
fn test_collector_record_and_retrieve() {
    let col = TelemetryCollector::new(100);
    col.record(make_telemetry_record("tool_a", true, 50));
    col.record(make_telemetry_record("tool_a", false, 200));
    assert_eq!(col.record_count(), 2);
    let stats = col.stats_for("tool_a").unwrap();
    assert_eq!(stats.call_count, 2);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.failure_count, 1);
}

#[test]
fn test_stats_avg_duration() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 100));
    col.record(make_telemetry_record("t", true, 200));
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.avg_duration(), Some(Duration::from_millis(150)));
}

#[test]
fn test_stats_success_rate() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 10));
    col.record(make_telemetry_record("t", false, 10));
    col.record(make_telemetry_record("t", false, 10));
    let stats = col.stats_for("t").unwrap();
    assert!((stats.success_rate() - 1.0 / 3.0).abs() < 1e-9);
}

#[test]
fn test_collector_ring_buffer_telemetry() {
    let col = TelemetryCollector::new(3);
    for _ in 0..5 {
        col.record(make_telemetry_record("t", true, 10));
    }
    assert_eq!(col.record_count(), 3);
}

#[test]
fn test_collector_clear_telemetry() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 10));
    col.clear();
    assert_eq!(col.record_count(), 0);
    assert!(col.stats_for("t").is_none());
}

#[test]
fn test_all_stats_multiple_tools() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("tool_a", true, 10));
    col.record(make_telemetry_record("tool_b", false, 20));
    let all = col.all_stats();
    assert!(all.contains_key("tool_a"));
    assert!(all.contains_key("tool_b"));
}

#[test]
fn test_record_tool_telemetry_convenience() {
    let col = TelemetryCollector::default();
    record_tool_telemetry(
        &col,
        "my_tool",
        Duration::from_millis(75),
        ExecutionStatus::Success,
        50,
        120,
        None,
    );
    let stats = col.stats_for("my_tool").unwrap();
    assert_eq!(stats.call_count, 1);
    assert_eq!(stats.total_input_bytes, 50);
    assert_eq!(stats.total_output_bytes, 120);
}

#[test]
fn test_min_max_duration() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 10));
    col.record(make_telemetry_record("t", true, 50));
    col.record(make_telemetry_record("t", true, 30));
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.min_duration, Some(Duration::from_millis(10)));
    assert_eq!(stats.max_duration, Some(Duration::from_millis(50)));
}

#[test]
fn test_execution_status_display() {
    assert_eq!(ExecutionStatus::Success.to_string(), "success");
    assert_eq!(ExecutionStatus::Failure.to_string(), "failure");
}

#[test]
fn test_stats_empty_avg_duration() {
    let stats = ToolStats::default();
    assert!(stats.avg_duration().is_none());
}

#[test]
fn test_stats_empty_success_rate() {
    let stats = ToolStats::default();
    assert!((stats.success_rate() - 0.0).abs() < 1e-9);
}

#[test]
fn test_stats_perfect_success_rate() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 10));
    col.record(make_telemetry_record("t", true, 20));
    let stats = col.stats_for("t").unwrap();
    assert!((stats.success_rate() - 1.0).abs() < 1e-9);
}

#[test]
fn test_stats_zero_success_rate() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", false, 10));
    col.record(make_telemetry_record("t", false, 20));
    let stats = col.stats_for("t").unwrap();
    assert!((stats.success_rate() - 0.0).abs() < 1e-9);
}

#[test]
fn test_collector_stats_for_nonexistent() {
    let col = TelemetryCollector::default();
    assert!(col.stats_for("nonexistent").is_none());
}

#[test]
fn test_all_records_snapshot() {
    let col = TelemetryCollector::new(100);
    col.record(make_telemetry_record("t1", true, 10));
    col.record(make_telemetry_record("t2", false, 20));
    let records = col.all_records();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].tool_name, "t1");
    assert_eq!(records[0].status, ExecutionStatus::Success);
    assert_eq!(records[1].tool_name, "t2");
    assert_eq!(records[1].status, ExecutionStatus::Failure);
}

#[test]
fn test_record_tool_telemetry_failure() {
    let col = TelemetryCollector::default();
    record_tool_telemetry(
        &col,
        "failing_tool",
        Duration::from_millis(100),
        ExecutionStatus::Failure,
        50,
        0,
        Some("connection refused".to_string()),
    );
    let stats = col.stats_for("failing_tool").unwrap();
    assert_eq!(stats.call_count, 1);
    assert_eq!(stats.failure_count, 1);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.total_input_bytes, 50);
    assert_eq!(stats.total_output_bytes, 0);
}

#[test]
fn test_stats_total_bytes_accumulation() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 10));
    col.record(make_telemetry_record("t", true, 20));
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.total_input_bytes, 200);
    assert_eq!(stats.total_output_bytes, 400);
}

#[test]
fn test_stats_total_duration() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 100));
    col.record(make_telemetry_record("t", true, 200));
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.total_duration, Duration::from_millis(300));
}

#[test]
fn test_record_output_tokens_estimated() {
    let col = TelemetryCollector::default();
    record_tool_telemetry(
        &col,
        "tool",
        Duration::from_millis(10),
        ExecutionStatus::Success,
        100,
        400,
        None,
    );
    let records = col.all_records();
    assert_eq!(records[0].output_tokens_estimated, 100);
}

#[test]
fn test_min_max_with_single_record() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 42));
    let stats = col.stats_for("t").unwrap();
    assert_eq!(stats.min_duration, Some(Duration::from_millis(42)));
    assert_eq!(stats.max_duration, Some(Duration::from_millis(42)));
}

#[test]
fn test_telemetry_record_serde_roundtrip() {
    let record = make_telemetry_record("tool", true, 100);
    let json = serde_json::to_string(&record).unwrap();
    let deserialized: ToolTelemetryRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tool_name, "tool");
    assert_eq!(deserialized.status, ExecutionStatus::Success);
    assert_eq!(deserialized.duration, Duration::from_millis(100));
    assert_eq!(deserialized.input_size_bytes, 100);
    assert_eq!(deserialized.output_size_bytes, 200);
    assert!(deserialized.error.is_none());
}

#[test]
fn test_telemetry_record_failure_serde() {
    let record = make_telemetry_record("tool", false, 50);
    let json = serde_json::to_string(&record).unwrap();
    let deserialized: ToolTelemetryRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.status, ExecutionStatus::Failure);
    assert_eq!(deserialized.error.as_deref(), Some("oops"));
    assert_eq!(deserialized.output_size_bytes, 0);
}

#[test]
fn test_execution_status_serde_roundtrip() {
    let s1 = ExecutionStatus::Success;
    let json = serde_json::to_string(&s1).unwrap();
    let deserialized: ExecutionStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, ExecutionStatus::Success);
    let s2 = ExecutionStatus::Failure;
    let json = serde_json::to_string(&s2).unwrap();
    let deserialized: ExecutionStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, ExecutionStatus::Failure);
}

#[test]
fn test_tool_stats_serde_roundtrip() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 10));
    col.record(make_telemetry_record("t", false, 20));
    let stats = col.stats_for("t").unwrap();
    let json = serde_json::to_string(&stats).unwrap();
    let deserialized: ToolStats = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.call_count, 2);
    assert_eq!(deserialized.success_count, 1);
    assert_eq!(deserialized.failure_count, 1);
}

#[test]
fn test_tool_stats_default() {
    let stats = ToolStats::default();
    assert_eq!(stats.call_count, 0);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.failure_count, 0);
    assert_eq!(stats.total_duration, Duration::ZERO);
    assert!(stats.min_duration.is_none());
    assert!(stats.max_duration.is_none());
    assert_eq!(stats.total_input_bytes, 0);
    assert_eq!(stats.total_output_bytes, 0);
}

#[test]
fn test_collector_clone_shares_data() {
    let col = TelemetryCollector::default();
    col.record(make_telemetry_record("t", true, 10));
    let cloned = col.clone();
    assert_eq!(cloned.record_count(), 1);
    cloned.record(make_telemetry_record("t", true, 20));
    assert_eq!(col.record_count(), 2);
}

#[test]
fn test_record_tool_telemetry_zero_output() {
    let col = TelemetryCollector::default();
    record_tool_telemetry(
        &col,
        "tool",
        Duration::from_millis(10),
        ExecutionStatus::Success,
        0,
        0,
        None,
    );
    let records = col.all_records();
    assert_eq!(records[0].output_tokens_estimated, 0);
}
