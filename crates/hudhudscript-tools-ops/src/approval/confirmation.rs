use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::audit::{AuditDecision, AuditLog};
use crate::risk::{RiskEngine, RiskLevel};
use crate::session::{PermissionStatus, SessionPermissions};

use super::{ApprovalPrompter, ApprovalRegistry, AutoApprovePrompter, PromptResponse};

/// The outcome of the full confirmation flow for a single tool call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfirmationOutcome {
    /// The operation may proceed.
    Allowed,
    /// The operation was blocked.
    Blocked,
}

/// An enhanced approval gate that integrates risk assessment, session
/// permissions, interactive prompts, and audit logging.
pub struct ConfirmationGate {
    pub(crate) risk_engine: RiskEngine,
    pub(crate) session: SessionPermissions,
    pub(crate) audit_log: AuditLog,
    pub(crate) registry: ApprovalRegistry,
    pub(crate) prompter: Box<dyn ApprovalPrompter>,
}

impl ConfirmationGate {
    /// Create a new confirmation gate.
    pub fn new(
        risk_engine: RiskEngine,
        session: SessionPermissions,
        audit_log: AuditLog,
        registry: ApprovalRegistry,
        prompter: Box<dyn ApprovalPrompter>,
    ) -> Self {
        Self {
            risk_engine,
            session,
            audit_log,
            registry,
            prompter,
        }
    }

    /// Create a gate with sensible defaults for testing or auto-approve mode.
    pub fn auto_approve(session_id: impl Into<String>) -> Self {
        Self {
            risk_engine: RiskEngine::with_defaults(),
            session: SessionPermissions::new(session_id),
            audit_log: AuditLog::default(),
            registry: ApprovalRegistry::new(),
            prompter: Box::new(AutoApprovePrompter),
        }
    }

    /// Run the full confirmation flow for a tool invocation.
    ///
    /// 1. Assess risk level.
    /// 2. If safe, auto-approve.
    /// 3. Check session permission memory.
    /// 4. If no cached decision, prompt the user.
    /// 5. Log the decision in the audit log.
    /// 6. Return whether the call is allowed.
    pub fn confirm(&self, tool_name: &str, arguments: &serde_json::Value) -> ConfirmationOutcome {
        let assessment = self.risk_engine.assess(tool_name);

        // Safe operations are auto-approved without prompting.
        if assessment.level == RiskLevel::Safe {
            debug!(tool = tool_name, "Safe operation — auto-approved");
            self.audit_log.log_decision(
                "",
                tool_name,
                arguments.clone(),
                assessment.level,
                AuditDecision::SafeAutoApproved,
                None,
                self.session.session_id(),
            );
            return ConfirmationOutcome::Allowed;
        }

        // Check session permission cache.
        if let Some(status) = self.session.check(tool_name) {
            let (outcome, decision) = match status {
                PermissionStatus::AlwaysAllow => {
                    info!(tool = tool_name, "Session permission: always-allow");
                    (ConfirmationOutcome::Allowed, AuditDecision::AutoApproved)
                }
                PermissionStatus::AlwaysDeny => {
                    info!(tool = tool_name, "Session permission: always-deny");
                    (ConfirmationOutcome::Blocked, AuditDecision::Denied)
                }
            };
            self.audit_log.log_decision(
                "",
                tool_name,
                arguments.clone(),
                assessment.level,
                decision,
                Some(format!("Session permission: {}", status)),
                self.session.session_id(),
            );
            return outcome;
        }

        // No cached decision — submit a registry request and prompt.
        let approval_id = self.registry.submit(tool_name, arguments.clone());
        let response = self.prompter.prompt(tool_name, arguments, assessment.level);

        let (outcome, decision, reason) = match response {
            PromptResponse::Yes => {
                self.registry.approve(&approval_id, None).ok();
                self.session.record_one_time_approval(tool_name);
                (ConfirmationOutcome::Allowed, AuditDecision::Approved, None)
            }
            PromptResponse::No => {
                self.registry.deny(&approval_id, None).ok();
                (ConfirmationOutcome::Blocked, AuditDecision::Denied, None)
            }
            PromptResponse::AlwaysAllow => {
                self.registry
                    .approve(&approval_id, Some("always-allow".into()))
                    .ok();
                self.session.set_always_allow(tool_name);
                (
                    ConfirmationOutcome::Allowed,
                    AuditDecision::Approved,
                    Some("User chose always-allow".to_string()),
                )
            }
            PromptResponse::AlwaysDeny => {
                self.registry
                    .deny(&approval_id, Some("always-deny".into()))
                    .ok();
                self.session.set_always_deny(tool_name);
                (
                    ConfirmationOutcome::Blocked,
                    AuditDecision::Denied,
                    Some("User chose always-deny".to_string()),
                )
            }
        };

        self.audit_log.log_decision(
            &approval_id,
            tool_name,
            arguments.clone(),
            assessment.level,
            decision,
            reason,
            self.session.session_id(),
        );

        outcome
    }

    /// Access the risk engine.
    pub fn risk_engine(&self) -> &RiskEngine {
        &self.risk_engine
    }

    /// Access the risk engine mutably (e.g. to add rules).
    pub fn risk_engine_mut(&mut self) -> &mut RiskEngine {
        &mut self.risk_engine
    }

    /// Access the session permissions.
    pub fn session(&self) -> &SessionPermissions {
        &self.session
    }

    /// Access the audit log.
    pub fn audit_log(&self) -> &AuditLog {
        &self.audit_log
    }

    /// Access the underlying approval registry.
    pub fn registry(&self) -> &ApprovalRegistry {
        &self.registry
    }
}
