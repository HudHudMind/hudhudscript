use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};
pub const NETWORK_NETWORK_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(153),
        long_code: "HHS_E_NETWORK_NETWORK_NOT_FOUND",
        short_code: "E0153",
        title: "Referenced network does not exist",
        short_description: "A runtime lookup referenced a network name that is not present in the orchestration registry.",
        long_description: "Networks must be registered before they can be started, inspected, or torn down. This error signals that the caller used a network name that is not currently in the registry — either because it was never registered, was already removed, or was registered in a different namespace.

Check initialization order: the orchestrator must bring networks online before handler code runs. Also verify that any multi-tenant prefix is being applied consistently between the place that registers the network and the place that looks it up.

When debugging, dump the current registry contents with `orchestrator.list_networks()`.",
        hints: &["Confirm the network has finished initializing before first use", "Dump `orchestrator.list_networks()` to see what is registered", "Apply tenant/namespace prefixes consistently"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_NETWORK_ALREADY_EXISTS", "HHS_E_ORCHESTRATION_NETWORK_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const NETWORK_TIMEOUT_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(154),
        long_code: "HHS_E_NETWORK_TIMEOUT_EXCEEDED",
        short_code: "E0154",
        title: "Network execution exceeded its overall timeout",
        short_description: "A network run did not finish within its configured wall-clock budget and was cancelled.",
        long_description: "Networks have an overall timeout that bounds how long the entire pipeline — across all layers — may run. When the deadline passes, the orchestrator cancels in-flight layers and returns this error. Per-layer timeouts are independent and do not extend the network budget.

If the timeout is firing legitimately, raise the network budget after profiling end-to-end latency. If a single slow layer dominates, address it directly with `LayerTimeoutExceeded` mitigations: per-agent timeouts, retries with caps, or model fallbacks. Always set the network budget strictly larger than the sum of expected layer durations plus inter-layer overhead.

For user-facing requests, fail fast with a shorter network timeout and surface a clear progress message.",
        hints: &["Set the network timeout strictly larger than sum of layer budgets", "Profile end-to-end latency before raising the limit", "Use shorter network timeouts for interactive requests"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_TIMEOUT_EXCEEDED", "HHS_E_ORCHESTRATION_WORKFLOW_TIMED_OUT"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_INVALID_WORKFLOW: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(159),
        long_code: "HHS_E_ORCHESTRATION_INVALID_WORKFLOW",
        short_code: "E0159",
        title: "Workflow definition failed validation",
        short_description: "The workflow definition is structurally invalid — missing required fields, conflicting steps, or unreachable nodes.",
        long_description: "Workflows are validated at registration time against a schema covering required fields, step graph well-formedness, and reachability of terminal states. This error fires when one or more rules are violated, with the offending rule named in the message.

Fix the workflow by re-reading the validation message and editing the definition. Common causes: missing `start` step, duplicate step ids, transitions to undefined steps, terminal nodes that no path can reach, or recursive cycles where a DAG is required.

Use the workflow builder API rather than raw maps so the type system catches the most common errors at compile time.",
        hints: &["Use the workflow builder API to catch errors at compile time", "Verify every transition target points to a defined step", "Run `workflow.lint()` before registering"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_INVALID_TOPOLOGY", "HHS_E_ORCHESTRATION_WORKFLOW_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_LAYER_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(160),
        long_code: "HHS_E_ORCHESTRATION_LAYER_ERROR",
        short_code: "E0160",
        title: "Orchestration encountered a layer-level error",
        short_description: "Top-level orchestration wraps a failure that originated inside one of its layers.",
        long_description: "This error is the top-of-stack wrapper that the orchestrator uses to surface inner `LayerError` variants — not found, already exists, execution failed, timed out, etc. The wrapped layer error is the source of truth.

Unwrap the cause using your error reporting facility (`source()` or `Display` chain) and treat the underlying variant as the actionable failure. The wrapper exists so workflow code can pattern-match on a single `OrchestrationError` family without enumerating every layer-level variant.

When logging, include both the orchestration code and the inner layer code so dashboards can group by either dimension.",
        hints: &["Unwrap the inner LayerError for actionable detail", "Log both the orchestration and layer error codes", "Pattern-match on `OrchestrationError::LayerError(_)` in handlers"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_EXECUTION_FAILED", "HHS_E_ORCHESTRATION_NETWORK_ERROR"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_NETWORK_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(161),
        long_code: "HHS_E_ORCHESTRATION_NETWORK_ERROR",
        short_code: "E0161",
        title: "Orchestration encountered a network-level error",
        short_description: "Top-level orchestration wraps a failure that originated inside one of its networks.",
        long_description: "This wrapper carries an inner `NetworkError` to the orchestration boundary so workflow code can react with a single match arm. Common inner causes include cyclic dependencies, missing layers, invalid topologies, and network-wide timeouts.

Unwrap the source error and address it at the network layer; do not retry blindly at the orchestration level since most network errors are deterministic configuration faults rather than transient issues. Transient cases (timeouts) are the main exception and may benefit from a bounded retry with backoff.

Report both the orchestration code and the wrapped network code to make incidents diagnosable from logs alone.",
        hints: &["Unwrap the inner NetworkError for actionable detail", "Treat configuration errors as deterministic — do not blindly retry", "Retry only transient network errors such as timeouts"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_INVALID_TOPOLOGY", "HHS_E_ORCHESTRATION_LAYER_ERROR", "HHS_E_ORCHESTRATION_NETWORK_EXECUTION_FAILED"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_NETWORK_EXECUTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(162),
        long_code: "HHS_E_ORCHESTRATION_NETWORK_EXECUTION_FAILED",
        short_code: "E0162",
        title: "Network execution failed during orchestration",
        short_description: "An orchestration step ran a network that returned a failure, aborting the workflow under fail-fast policy.",
        long_description: "When a workflow step delegates to a network and that network ends in failure, the orchestrator surfaces this error with both the network name and the underlying cause attached. The workflow's error policy then decides whether to retry the step, branch into a fallback, or abort.

Fix the failing network first by inspecting the inner error. Once the root cause is understood, configure the workflow with the appropriate fault-tolerance strategy: retries with backoff for transient failures, fallback steps for permanent ones, or compensating actions for partial successes.

Make sure the workflow's overall budget allows for the chosen retry strategy or you risk hitting `WorkflowTimedOut` instead.",
        hints: &["Inspect the wrapped network error for the real cause", "Configure workflow-level retries only for transient failures", "Verify the workflow timeout accommodates the retry strategy"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_LAYER_EXECUTION_FAILED", "HHS_E_ORCHESTRATION_NETWORK_ERROR", "HHS_E_ORCHESTRATION_WORKFLOW_TIMED_OUT"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_NETWORK_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(163),
        long_code: "HHS_E_ORCHESTRATION_NETWORK_NOT_FOUND",
        short_code: "E0163",
        title: "Workflow references an unknown network",
        short_description: "A workflow step delegated to a network whose name is not registered with the orchestrator.",
        long_description: "Workflows can call into networks by name. When the step executes, the orchestrator resolves the name against its network registry; an unknown name yields this error and aborts the step.

This is almost always a wiring problem: the network was never registered, or was registered under a different name (case sensitivity, namespace prefix). Validate workflow definitions against the registered network set at workflow load time so the failure surfaces before runtime.

In multi-tenant deployments, double-check that the tenant prefix used by the workflow matches the prefix used during registration.",
        hints: &["Validate workflows against the registered network set at load time", "Match case and namespace prefixes exactly", "Use shared constants for network names referenced from workflows"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_NETWORK_NOT_FOUND", "HHS_E_ORCHESTRATION_WORKFLOW_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_WORKFLOW_ALREADY_EXISTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(164),
        long_code: "HHS_E_ORCHESTRATION_WORKFLOW_ALREADY_EXISTS",
        short_code: "E0164",
        title: "Workflow with this name is already registered",
        short_description: "An attempt was made to register a workflow whose name collides with an existing one in the orchestrator.",
        long_description: "The orchestrator keeps a registry of workflow definitions keyed by name. Re-registering a name without first removing the old definition is rejected to avoid silent overwrite of a possibly in-use workflow.

During development hot-reload, call `orchestrator.remove_workflow(name)` (or use `upsert_workflow`) before re-registering. In production, treat workflow registration as part of immutable deployment: bump the workflow name on each version (e.g. `analyze_v3`) so old in-flight runs can finish under the previous definition.

Namespacing per tenant or per service prevents collisions in shared environments.",
        hints: &["Use versioned workflow names in production deployments", "Call `upsert_workflow` for intentional in-place updates", "Namespace workflows per tenant or service"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_ORCHESTRATION_WORKFLOW_NOT_FOUND", "HHS_E_NETWORK_NETWORK_ALREADY_EXISTS"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_WORKFLOW_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(165),
        long_code: "HHS_E_ORCHESTRATION_WORKFLOW_NOT_FOUND",
        short_code: "E0165",
        title: "Referenced workflow is not registered",
        short_description: "A run, inspect, or cancel call referenced a workflow name that is not present in the orchestrator registry.",
        long_description: "Workflows must be registered before they can be started or queried. This error indicates the caller used a name that is not (or no longer) in the registry — typical causes are stale clients after a redeploy, namespace mismatches, or typos in the workflow id.

List currently registered workflows with `orchestrator.list_workflows()` to confirm what is available, and validate workflow names at the edge of your public API. For versioned workflows, prefer explicit version suffixes so old clients fail fast and loudly rather than silently picking up the wrong definition.

Wrap lookups in a typed handle so the rest of your code cannot drift from the registry.",
        hints: &["Use `orchestrator.list_workflows()` to confirm registration", "Validate workflow ids at the API boundary", "Prefer versioned workflow names to detect client drift"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_ORCHESTRATION_WORKFLOW_ALREADY_EXISTS", "HHS_E_ORCHESTRATION_INVALID_WORKFLOW"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const ORCHESTRATION_WORKFLOW_TIMED_OUT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(166),
        long_code: "HHS_E_ORCHESTRATION_WORKFLOW_TIMED_OUT",
        short_code: "E0166",
        title: "Workflow execution exceeded its overall timeout",
        short_description: "A workflow run did not complete within its configured deadline and was cancelled by the orchestrator.",
        long_description: "Workflows carry a wall-clock budget that bounds the total time from start to terminal step. When the deadline passes, the orchestrator cancels any in-flight steps (including their underlying networks and layers) and returns this error.

Fix this by either raising the workflow budget after profiling, or by reducing per-step latency: tune model selection, parallelize independent steps, cache intermediate results, or move long-running work into background jobs that the workflow only awaits when needed.

Remember that workflow timeout supersedes network and layer timeouts; the inner budgets must collectively fit inside the workflow budget plus expected overhead.",
        hints: &["Profile end-to-end workflow latency before raising the deadline", "Parallelize independent steps to reduce wall-clock time", "Ensure inner network/layer budgets fit inside the workflow budget"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_TIMEOUT_EXCEEDED", "HHS_E_LAYER_TIMEOUT_EXCEEDED", "HHS_E_ORCHESTRATION_NETWORK_EXECUTION_FAILED"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const PERMISSION_AGENT_NOT_REGISTERED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(186),
        long_code: "HHS_E_PERMISSION_AGENT_NOT_REGISTERED",
        short_code: "E0186",
        title: "Agent is not registered with the permission system",
        short_description: "A permission check was performed for an agent that has no entry in the orchestration ACL registry.",
        long_description: "Every agent participating in a permissioned orchestration must be registered with the permission system so it has an identity to authorize against. This error fires when a check is requested for an agent id that has no corresponding ACL record — either because registration was skipped, the agent was deregistered, or the id is misspelled.

Register each agent during spawn with `permissions.register_agent(id, role)` and deregister on shutdown. Treat the registration as part of the agent lifecycle so it cannot be forgotten. For dynamically spawned agents, use a factory that performs registration as part of construction.

If you intentionally want unauthenticated agents, place them in a sandbox network with a permissive default policy rather than bypassing registration.",
        hints: &["Register agents as part of their spawn lifecycle", "Use a factory that registers permissions on construction", "Place unauthenticated agents in a sandbox network instead of skipping registration"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_PERMISSION_DENIED"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const PERMISSION_DENIED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(187),
        long_code: "HHS_E_PERMISSION_DENIED",
        short_code: "E0187",
        title: "Agent is not permitted to perform this action",
        short_description: "The permission system rejected an action because the agent's role does not grant access to the requested resource.",
        long_description: "HudHudScript orchestration enforces ACLs on cross-agent calls, resource reads, and tool invocations. When an agent attempts an action that its role does not permit, the call is rejected with this error and the violation is logged for audit.

Fix this by either granting the agent the necessary permission (via role assignment or policy update), or by routing the action through an agent that already has the right. Do not catch and ignore this error blindly — permission failures are signals about either a policy gap or an attempted privilege escalation, both of which deserve attention.

Audit logs include `agent`, `resource`, and `action` fields so you can triage whether the policy needs to change or the calling code is misbehaving.",
        hints: &["Review the role assigned to the agent and the policy on the resource", "Route the action through an agent that already holds the permission", "Never silently swallow permission errors — log and alert on them"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_PERMISSION_AGENT_NOT_REGISTERED"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };
