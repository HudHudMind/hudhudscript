//! HudHudScript Tools — Operations
//!
//! Approval workflows, retry logic, telemetry, risk assessment, audit logging,
//! and per-session permission management.

pub mod approval;
pub mod audit;
pub mod retry;
pub mod risk;
pub mod session;
pub mod telemetry;

pub use approval::{
    ApprovalError, ApprovalGate, ApprovalPrompter, ApprovalRegistry, ApprovalRequest,
    ApprovalState, AutoApprovePrompter, AutoDenyPrompter, ConfirmationGate, ConfirmationOutcome,
    PromptResponse,
};
pub use audit::{AuditDecision, AuditEntry, AuditLog};
pub use retry::{RetryPolicy, ToolCallExecutor, ToolCallOutcome};
pub use risk::{RiskAssessment, RiskEngine, RiskLevel, RiskRule};
pub use session::{PermissionRecord, PermissionStatus, SessionPermissions};
pub use telemetry::{
    record_tool_telemetry, ExecutionStatus, InstrumentedToolExecutor, TelemetryCollector,
    ToolStats, ToolTelemetryRecord,
};
