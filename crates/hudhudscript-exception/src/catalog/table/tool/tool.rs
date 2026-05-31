use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const TOOL_EXECUTION_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(295),
        long_code: "HHS_E_TOOL_EXECUTION_FAILED",
        short_code: "E0295",
        title: "Tool dispatch ran but failed",
        short_description: "A registered tool was located and invoked, but its execution returned a runtime failure.",
        long_description: "This is the generic outer wrapper for any tool that fails during its own work. The wrapped message comes from the tool implementation and is the authoritative description. Use this error as a router: read the wrapped detail, then jump to the more specific error code if one exists (HTTP, DB, Git, etc.).

Fix it by inspecting the wrapped cause and following its remediation. If the tool is custom, add structured error context inside the tool implementation so callers do not have to grep free-text.

For unreliable tools, wrap calls in `try`/`catch` and decide per-tool whether to retry, fall back, or surface to the operator.",
        hints: &["Read the wrapped cause — it identifies the real subsystem", "Add structured context in custom tool implementations", "Decide per-tool whether to retry or surface failures", "Jump to the specific error (HTTP/DB/Git) when one exists"],
        example_bad: Some("tool::call(\"flaky\", args); // no error handling"),
        example_good: Some("try { tool::call(\"flaky\", args); } catch (e) { log::warn(e); }"),
        see_also: &["ToolInvalidArguments", "ToolValidation", "ToolSecurityViolation"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const TOOL_INVALID_ARGUMENTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(296),
        long_code: "HHS_E_TOOL_INVALID_ARGUMENTS",
        short_code: "E0296",
        title: "Tool received invalid arguments",
        short_description: "Arguments to a tool call failed the tool's input schema check before any work was done.",
        long_description: "Each tool declares an input schema. This error fires when the supplied arguments fail that schema — missing required fields, wrong types, values outside enums, or extra unknown fields when the schema is closed.

Fix it by inspecting the tool's declared schema (`tool::describe(name)`) and aligning the call site. When generating tool calls from LLM output, validate the JSON locally before dispatch so that the model can be re-prompted on its own malformed output rather than failing the script.

This is the right error to raise from custom tools that want to reject bad input cleanly — keep the message specific to the offending field.",
        hints: &["Inspect the schema with tool::describe(name)", "Validate LLM-generated tool calls before dispatch", "Name the offending field in custom tool error messages", "Watch for closed schemas that reject extra fields"],
        example_bad: Some("tool::call(\"send_email\", { to: 42 }); // wrong type"),
        example_good: Some("tool::call(\"send_email\", { to: \"a@b.c\", subject: \"hi\" });"),
        see_also: &["ToolValidation", "ToolExecutionFailed", "ToolSecurityViolation"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const TOOL_SECURITY_VIOLATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(297),
        long_code: "HHS_E_TOOL_SECURITY_VIOLATION",
        short_code: "E0297",
        title: "Tool call blocked by security policy",
        short_description: "The runtime's security policy denied the tool call — for example a sandboxed script tried to access the network or filesystem.",
        long_description: "HudHudScript can run with a configurable security policy that restricts which tools a script can invoke and which resources those tools may touch. This error fires when a call is blocked by that policy: a sandboxed script reaches out to the network, a read-only context tries to mutate files, or a deny-listed tool is invoked.

Fix it by either rewriting the script to stay within the allowed surface, or by adjusting the policy in the host configuration if the access is legitimately required. Never silently widen a policy in production without auditing what it now allows.

For multi-tenant hosts, log every security violation with the script id and the requested tool — these logs are the audit trail for sandbox escapes.",
        hints: &["Read the policy to see which tools/resources are allowed", "Refactor scripts to stay inside the sandbox where possible", "Audit policy widenings — they expand the attack surface", "Log every violation with script id + requested tool"],
        example_bad: Some("// script in network-deny sandbox
http::get(\"https://example.com\");"),
        example_good: Some("// move external calls into a privileged orchestrator script"),
        see_also: &["ToolExecutionFailed", "ToolValidation", "ApprovalInvalidTransition"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };

pub const TOOL_VALIDATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(298),
        long_code: "HHS_E_TOOL_VALIDATION",
        short_code: "E0298",
        title: "Tool input failed semantic validation",
        short_description: "Arguments parsed against the schema but failed a stronger semantic check, such as cross-field constraints or referential integrity.",
        long_description: "Where `ToolInvalidArguments` covers shape/type problems, `ToolValidation` covers richer rules: a date range whose end precedes its start, a foreign key that does not resolve, mutually exclusive options that were both supplied, or a value that fails a regex constraint.

Fix it by reading the validation message — it should name the failing rule. For custom tools, prefer raising this error (rather than `ToolInvalidArguments`) for cross-field and lookup-based checks so callers can distinguish syntactic from semantic issues.

When building tool inputs from user data, run the same validation client-side first to give faster feedback before dispatch.",
        hints: &["Distinguish: ToolInvalidArguments = shape, ToolValidation = rules", "Name the failing rule in custom validation messages", "Mirror server-side validation client-side for faster feedback", "Reject mutually exclusive options early"],
        example_bad: Some("tool::call(\"book\", { start: \"2026-05-10\", end: \"2026-05-01\" });"),
        example_good: Some("tool::call(\"book\", { start: \"2026-05-01\", end: \"2026-05-10\" });"),
        see_also: &["ToolInvalidArguments", "ToolExecutionFailed", "ToolSecurityViolation"],
        since_version: "0.4.0",
        category: ExceptionCategory::Tool,
    };
