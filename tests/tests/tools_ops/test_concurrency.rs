//! Concurrency, edge-case, and integration tests for hudhudscript-tools-ops.
//!
//! Supplements `tools_ops_test_lib.rs` with thread-safety scenarios,
//! boundary conditions, and cross-module integration flows.

use hudhudscript_tools_ops::approval::{
    ApprovalPrompter, ApprovalRegistry, ApprovalState, ConfirmationGate, ConfirmationOutcome,
    PromptResponse,
};
use hudhudscript_tools_ops::audit::{AuditDecision, AuditLog};
use hudhudscript_tools_ops::retry::RetryPolicy;
use hudhudscript_tools_ops::risk::{RiskEngine, RiskLevel, RiskRule};
use hudhudscript_tools_ops::session::{PermissionStatus, SessionPermissions};
use hudhudscript_tools_ops::telemetry::{
    record_tool_telemetry, ExecutionStatus, TelemetryCollector,
};
use serde_json::json;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

// =========================================================================
// 1. ApprovalRegistry — concurrent submit + approve from multiple threads
// =========================================================================

#[test]
fn approval_registry_concurrent_submit_and_resolve() {
    let registry = ApprovalRegistry::new();
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = Vec::new();

    for i in 0..10 {
        let reg = registry.clone();
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            let id = reg.submit(format!("tool_{}", i), json!({"idx": i}));
            // Approve immediately
            reg.approve(&id, Some(format!("auto-{}", i))).unwrap();
            reg.mark_executed(&id).unwrap();
            id
        }));
    }

    let ids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(ids.len(), 10);

    // All should be Executed
    for id in &ids {
        let req = registry.get(id).unwrap();
        assert_eq!(req.state, ApprovalState::Executed);
        assert!(req.reason.is_some());
    }

    // No pending requests remain
    assert!(registry.pending().is_empty());
}

// =========================================================================
// 2. AuditLog — concurrent writes respect ring-buffer cap
// =========================================================================

#[test]
fn audit_log_concurrent_writes_respect_max_entries() {
    let log = AuditLog::new(50);
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = Vec::new();

    for t in 0..10 {
        let l = log.clone();
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            for i in 0..20 {
                l.log_decision(
                    format!("ap-{}-{}", t, i),
                    format!("tool_{}", t),
                    json!({}),
                    RiskLevel::Safe,
                    AuditDecision::Approved,
                    None,
                    "sess-concurrent",
                );
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // 10 threads x 20 entries = 200 total, but max is 50
    assert!(log.len() <= 50);
    assert!(log.len() > 0);
}

// =========================================================================
// 3. SessionPermissions — concurrent set/check does not panic
// =========================================================================

#[test]
fn session_permissions_concurrent_access() {
    let session = SessionPermissions::new("concurrent-session");
    let barrier = Arc::new(Barrier::new(6));
    let mut handles = Vec::new();

    // 3 writers setting always-allow
    for i in 0..3 {
        let s = session.clone();
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            s.set_always_allow(format!("tool_{}", i));
        }));
    }

    // 3 readers checking
    for i in 0..3 {
        let s = session.clone();
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            // May or may not find the permission yet — must not panic
            let _ = s.check(&format!("tool_{}", i));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // After all threads join, all three tools should have permissions
    for i in 0..3 {
        let status = session.check(&format!("tool_{}", i));
        assert_eq!(status, Some(PermissionStatus::AlwaysAllow));
    }
}

// =========================================================================
// 4. TelemetryCollector — concurrent recording preserves stats accuracy
// =========================================================================

#[test]
fn telemetry_collector_concurrent_records_stats_consistent() {
    let collector = TelemetryCollector::new(5000);
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = Vec::new();

    // 4 threads each recording 100 success entries for the same tool
    for _ in 0..4 {
        let c = collector.clone();
        let bar = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            bar.wait();
            for _ in 0..100 {
                record_tool_telemetry(
                    &c,
                    "shared_tool",
                    Duration::from_millis(5),
                    ExecutionStatus::Success,
                    32,
                    64,
                    None,
                );
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let stats = collector.stats_for("shared_tool").unwrap();
    assert_eq!(stats.call_count, 400);
    assert_eq!(stats.success_count, 400);
    assert_eq!(stats.failure_count, 0);
    assert_eq!(stats.total_input_bytes, 400 * 32);
    assert_eq!(stats.total_output_bytes, 400 * 64);
    assert!((stats.success_rate() - 1.0).abs() < f64::EPSILON);
}

// =========================================================================
// 5. RetryPolicy — delay_for with zero backoff always returns ZERO
// =========================================================================

#[test]
fn retry_policy_delay_for_zero_backoff_all_attempts() {
    let policy = RetryPolicy {
        max_retries: 5,
        backoff_ms: 0,
        fallback_tool: None,
        exponential: true, // exponential should not matter when backoff_ms = 0
    };

    for attempt in 0..10 {
        assert_eq!(policy.delay_for(attempt), Duration::ZERO);
    }
}

// =========================================================================
// 6. RetryPolicy — exponential delay caps at attempt 10 (1 << 10 = 1024)
// =========================================================================

#[test]
fn retry_policy_exponential_caps_shift_at_10() {
    let policy = RetryPolicy {
        max_retries: 20,
        backoff_ms: 1,
        fallback_tool: None,
        exponential: true,
    };

    // attempt 10 and 11 should both produce the same multiplier (1 << 10 = 1024)
    let d10 = policy.delay_for(10);
    let d11 = policy.delay_for(11);
    assert_eq!(d10, Duration::from_millis(1024));
    assert_eq!(d11, Duration::from_millis(1024));
}

// =========================================================================
// 7. RiskRule — glob-star-only pattern matches everything
// =========================================================================

#[test]
fn risk_rule_star_only_matches_any_tool() {
    let rule = RiskRule::new("*", RiskLevel::Warning, "catch-all");
    assert!(rule.matches("anything"));
    assert!(rule.matches(""));
    assert!(rule.matches("delete_file"));
}

// =========================================================================
// 8. RiskEngine — override removed falls back to rule then default
// =========================================================================

#[test]
fn risk_engine_override_removal_falls_through_to_rule() {
    let mut engine = RiskEngine::new();
    engine.add_rule(RiskRule::new(
        "deploy_*",
        RiskLevel::Warning,
        "deploy is risky",
    ));
    engine.set_override("deploy_prod", RiskLevel::Dangerous);

    // With override
    assert_eq!(engine.assess("deploy_prod").level, RiskLevel::Dangerous);
    assert!(engine.assess("deploy_prod").requires_approval);

    // Remove override — should fall through to the pattern rule
    let removed = engine.remove_override("deploy_prod");
    assert_eq!(removed, Some(RiskLevel::Dangerous));
    assert_eq!(engine.assess("deploy_prod").level, RiskLevel::Warning);
    assert!(!engine.assess("deploy_prod").requires_approval);
}

// =========================================================================
// 9. RiskAssessment — Warning level does NOT require approval
// =========================================================================

#[test]
fn risk_assessment_warning_does_not_require_approval() {
    let engine = RiskEngine::with_defaults();
    let assessment = engine.assess("write_file"); // matches write_* => Warning
    assert_eq!(assessment.level, RiskLevel::Warning);
    assert!(
        !assessment.requires_approval,
        "Warning level should not require approval, only Dangerous does"
    );
}

// =========================================================================
// 10. ConfirmationGate — full flow with custom AlwaysAllow prompter
// =========================================================================

/// A prompter that always returns AlwaysAllow.
struct AlwaysAllowPrompter;

impl ApprovalPrompter for AlwaysAllowPrompter {
    fn prompt(
        &self,
        _tool_name: &str,
        _arguments: &serde_json::Value,
        _risk_level: RiskLevel,
    ) -> PromptResponse {
        PromptResponse::AlwaysAllow
    }
}

#[test]
fn confirmation_gate_always_allow_prompter_remembers_session() {
    let session = SessionPermissions::new("test-aa");
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        session.clone(),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(AlwaysAllowPrompter),
    );

    // First call to a dangerous tool — prompter returns AlwaysAllow
    let outcome = gate.confirm("delete_something", &json!({}));
    assert_eq!(outcome, ConfirmationOutcome::Allowed);

    // Session should now remember always-allow for this tool
    let status = session.check("delete_something");
    assert_eq!(status, Some(PermissionStatus::AlwaysAllow));

    // Second call should be auto-approved via session cache (no prompt needed)
    let outcome2 = gate.confirm("delete_something", &json!({}));
    assert_eq!(outcome2, ConfirmationOutcome::Allowed);

    // Audit log should have entries for both calls
    assert!(gate.audit_log().len() >= 2);
}

// =========================================================================
// 11. ConfirmationGate — AlwaysDeny prompter blocks and remembers
// =========================================================================

/// A prompter that always returns AlwaysDeny.
struct AlwaysDenyPrompter;

impl ApprovalPrompter for AlwaysDenyPrompter {
    fn prompt(
        &self,
        _tool_name: &str,
        _arguments: &serde_json::Value,
        _risk_level: RiskLevel,
    ) -> PromptResponse {
        PromptResponse::AlwaysDeny
    }
}

#[test]
fn confirmation_gate_always_deny_prompter_blocks_and_remembers() {
    let session = SessionPermissions::new("test-ad");
    let gate = ConfirmationGate::new(
        RiskEngine::with_defaults(),
        session.clone(),
        AuditLog::default(),
        ApprovalRegistry::new(),
        Box::new(AlwaysDenyPrompter),
    );

    let outcome = gate.confirm("exec_command", &json!({"cmd": "rm -rf /"}));
    assert_eq!(outcome, ConfirmationOutcome::Blocked);

    // Session should remember always-deny
    let status = session.check("exec_command");
    assert_eq!(status, Some(PermissionStatus::AlwaysDeny));

    // Subsequent calls are auto-blocked without prompting
    let outcome2 = gate.confirm("exec_command", &json!({"cmd": "ls"}));
    assert_eq!(outcome2, ConfirmationOutcome::Blocked);
}

// =========================================================================
// 12. ToolStats — mixed success/failure rate calculation
// =========================================================================

#[test]
fn tool_stats_mixed_success_failure_rate() {
    let collector = TelemetryCollector::new(100);

    // 3 successes, 1 failure
    for i in 0..4 {
        let status = if i < 3 {
            ExecutionStatus::Success
        } else {
            ExecutionStatus::Failure
        };
        record_tool_telemetry(
            &collector,
            "flaky_tool",
            Duration::from_millis(10),
            status,
            8,
            16,
            if i >= 3 {
                Some("error".to_string())
            } else {
                None
            },
        );
    }

    let stats = collector.stats_for("flaky_tool").unwrap();
    assert_eq!(stats.call_count, 4);
    assert_eq!(stats.success_count, 3);
    assert_eq!(stats.failure_count, 1);
    assert!((stats.success_rate() - 0.75).abs() < f64::EPSILON);
}

// =========================================================================
// 13. ApprovalRegistry — submit preserves arguments and tool_name
// =========================================================================

#[test]
fn approval_registry_submit_preserves_fields() {
    let registry = ApprovalRegistry::new();
    let args = json!({"path": "/etc/shadow", "mode": "read"});
    let id = registry.submit("read_sensitive_file", args.clone());

    let req = registry.get(&id).unwrap();
    assert_eq!(req.tool_name, "read_sensitive_file");
    assert_eq!(req.arguments, args);
    assert_eq!(req.state, ApprovalState::Pending);
    assert!(req.reason.is_none());
    // created_at and updated_at should be close
    assert!(req.created_at <= req.updated_at);
}

// =========================================================================
// 14. AuditLog — JSON round-trip preserves tool_name and decision
// =========================================================================

#[test]
fn audit_log_json_roundtrip_preserves_data() {
    let log = AuditLog::new(100);
    log.log_decision(
        "ap-1",
        "dangerous_op",
        json!({"x": 42}),
        RiskLevel::Dangerous,
        AuditDecision::Denied,
        Some("too risky".to_string()),
        "session-rt",
    );
    log.log_decision(
        "ap-2",
        "safe_read",
        json!({}),
        RiskLevel::Safe,
        AuditDecision::SafeAutoApproved,
        None,
        "session-rt",
    );

    let json_str = log.to_json().unwrap();

    let log2 = AuditLog::new(100);
    let loaded = log2.load_from_json(&json_str).unwrap();
    assert_eq!(loaded, 2);

    let entries = log2.entries();
    assert_eq!(entries.len(), 2);

    let denied_entry = entries
        .iter()
        .find(|e| e.tool_name == "dangerous_op")
        .unwrap();
    assert_eq!(denied_entry.decision, AuditDecision::Denied);
    assert_eq!(denied_entry.reason.as_deref(), Some("too risky"));
    assert_eq!(denied_entry.risk_level, RiskLevel::Dangerous);
    assert_eq!(denied_entry.session_id, "session-rt");

    let safe_entry = entries.iter().find(|e| e.tool_name == "safe_read").unwrap();
    assert_eq!(safe_entry.decision, AuditDecision::SafeAutoApproved);
    assert!(safe_entry.reason.is_none());
}

// =========================================================================
// 15. SessionPermissions — revoke resets to prompt state
// =========================================================================

#[test]
fn session_permissions_revoke_resets_to_prompt() {
    let session = SessionPermissions::new("revoke-test");
    session.set_always_allow("tool_a");

    assert_eq!(session.check("tool_a"), Some(PermissionStatus::AlwaysAllow));

    let revoked = session.revoke("tool_a");
    assert!(revoked);

    // After revocation, check returns None (user would be prompted again)
    assert_eq!(session.check("tool_a"), None);

    // But approval history via one-time is separate and not affected by revoke
    // (was_approved_before may still return false since always-allow was revoked
    //  and no one-time approval was recorded)
    assert!(!session.was_approved_before("tool_a"));
}
