use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const APPROVAL_INVALID_TRANSITION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(1),
        long_code: "HHS_E_APPROVAL_INVALID_TRANSITION",
        short_code: "E0001",
        title: "Approval state transition is invalid",
        short_description: "An approval request cannot move from its current state to the requested state under the approval workflow rules.",
        long_description: "Approval requests follow a finite state machine: typically Pending -> Approved or Pending -> Denied, with terminal states that cannot be re-entered. This error means a caller tried an illegal transition, for example approving an already-denied request or re-opening one that was previously resolved.

Fix it by inspecting the current state of the request before issuing the transition, and by ensuring no two callers race to resolve the same request. If the workflow legitimately needs to revisit a decision, create a new approval request rather than mutating the old one.

This commonly appears in human-in-the-loop scripts where the operator clicks a button twice, or where retry logic re-issues a decision after a transient network failure.",
        hints: &["Fetch the current approval state before calling approve/deny", "Treat Approved and Denied as terminal — never transition out", "Make decision endpoints idempotent so retries are safe", "Issue a new approval request if a fresh decision is needed"],
        example_bad: Some("let req = approval::get(id);
approval::approve(id); // already Denied — invalid transition"),
        example_good: Some("let req = approval::get(id);
if req.state == \"Pending\" {
    approval::approve(id);
}"),
        see_also: &["ApprovalNotFound", "ToolExecutionFailed", "ToolValidation"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const APPROVAL_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(2),
        long_code: "HHS_E_APPROVAL_NOT_FOUND",
        short_code: "E0002",
        title: "Approval request id does not exist",
        short_description: "The approval store has no entry matching the supplied request id, so the operation cannot be applied.",
        long_description: "The approval subsystem keys requests by an opaque id. This error means that id is unknown — it was never created, has expired and been garbage collected, or belongs to a different approval store than the one being queried.

Fix it by ensuring the id is captured from the original `approval::request(...)` call and passed unchanged. Beware of stringification, JSON round-trips that lose precision, or stores that are scoped per-process and not shared with the consumer.

A common cause is calling the approver from a fresh script run without persisting the request id from the previous run, or pointing the approver and the requester at different backends.",
        hints: &["Verify the id came from a successful approval::request() call", "Check that requester and approver share the same backend store", "Watch for id truncation across JSON / database / URL boundaries", "Confirm the request has not exceeded its TTL and been purged"],
        example_bad: Some("approval::approve(\"req-123\"); // never created"),
        example_good: Some("let id = approval::request({ title: \"Deploy prod\" });
approval::approve(id);"),
        see_also: &["ApprovalInvalidTransition", "ToolExecutionFailed", "ToolInvalidArguments"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };
