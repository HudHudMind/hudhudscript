use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const EVENT_BUS_CHANNEL_CLOSED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(78),
        long_code: "HHS_E_EVENT_BUS_CHANNEL_CLOSED",
        short_code: "E0078",
        title: "Event bus channel is closed",
        short_description: "A publish or subscribe operation was attempted on an event bus channel that has already been closed.",
        long_description: "Event bus channels in HudHudScript orchestration are backed by async broadcast primitives that can be shut down either explicitly or when the owning network is torn down. Once a channel is closed, no further messages can be sent and no new subscribers can attach.

This error usually indicates that the orchestrator was shut down while background tasks still held references to the bus, or that a subscriber outlived the publisher. Ensure clean shutdown ordering: tear down producers after all consumers have drained, or use `bus.is_closed()` guards before publishing.

In long-running deployments, consider wrapping publishes in a retry-with-reconnect strategy if channels may be recreated.",
        hints: &["Check `bus.is_closed()` before publishing events", "Ensure the event bus outlives all its producers and subscribers", "Shut down subscribers before closing the channel to avoid late sends"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_EVENT_BUS_NO_SUBSCRIBERS", "HHS_E_ORCHESTRATION_NETWORK_ERROR"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const EVENT_BUS_NO_SUBSCRIBERS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(79),
        long_code: "HHS_E_EVENT_BUS_NO_SUBSCRIBERS",
        short_code: "E0079",
        title: "No subscribers attached to event bus topic",
        short_description: "A publish operation found no active subscribers on the target topic, and the bus is configured to treat this as an error.",
        long_description: "By default, event buses are fire-and-forget: publishing to an empty topic silently drops the message. When a bus is configured in strict mode (or when using `publish_required`), dispatching to a topic with zero subscribers raises this error so unnoticed drops cannot propagate.

To fix this, either register at least one subscriber before the first publish, switch the bus to best-effort mode, or guard the call with `bus.subscriber_count(topic) > 0`. Race conditions at startup are a common cause — producers can come online before consumers finish subscribing.

If the topic is genuinely optional, use the non-strict publish variant instead of treating empty delivery as an error.",
        hints: &["Use a startup barrier so subscribers attach before producers begin", "Call `bus.subscriber_count(topic)` to gate strict publishes", "Switch to best-effort publish if empty delivery is acceptable"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_EVENT_BUS_CHANNEL_CLOSED"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const LAYER_DEPENDENCY_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(113),
        long_code: "HHS_E_LAYER_DEPENDENCY_NOT_FOUND",
        short_code: "E0113",
        title: "Layer references an unknown dependency",
        short_description: "A layer declared a dependency on another layer that has not been registered in the orchestration graph.",
        long_description: "Layers form a directed graph: each layer can depend on zero or more upstream layers that must complete before it runs. When the graph is built, every declared dependency is resolved against the set of registered layers. A missing dependency aborts graph construction.

This error commonly occurs when layers are registered in the wrong order, when a dependency name is misspelled, or when a layer is conditionally omitted under some configurations but still referenced by downstream code.

Register all layers up front, then validate the graph with `network.validate()` before running it. Use constants or enums for layer names to avoid typos.",
        hints: &["Register all layers before building the execution graph", "Use constants for layer names to avoid typos", "Call `network.validate()` after construction to catch this early"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_LAYER_NOT_FOUND", "HHS_E_NETWORK_CYCLIC_DEPENDENCY"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const LAYER_EXECUTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(114),
        long_code: "HHS_E_LAYER_EXECUTION_FAILED",
        short_code: "E0114",
        title: "Layer execution returned a failure",
        short_description: "One or more agents within a layer returned an error, causing the whole layer to fail.",
        long_description: "A layer runs a set of agents either in parallel or as a pipeline. By default, if any agent inside the layer fails, the layer itself is marked failed and downstream layers are not executed. The error message wraps the underlying agent failure.

Fix the root cause in the failing agent (see the wrapped error), or configure the layer with a fault-tolerance policy: `continue_on_error`, `best_effort`, or retry. For non-critical agents, consider moving them to an optional side layer that does not block the main pipeline.

When debugging, enable layer-level tracing to see which specific agent raised the inner error and with what inputs.",
        hints: &["Inspect the wrapped inner error for the failing agent", "Configure `continue_on_error` for layers where partial failure is acceptable", "Add per-agent retries before failing the entire layer"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_TIMEOUT_EXCEEDED", "HHS_E_NETWORK_LAYER_EXECUTION_FAILED", "HHS_E_ORCHESTRATION_LAYER_ERROR"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const LAYER_LAYER_ALREADY_EXISTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(115),
        long_code: "HHS_E_LAYER_LAYER_ALREADY_EXISTS",
        short_code: "E0115",
        title: "Layer with this name is already registered",
        short_description: "An attempt was made to register a layer whose name collides with an existing layer in the same network.",
        long_description: "Layer names must be unique within a network. Registering a second layer with the same name would make dependency resolution ambiguous, so the orchestrator rejects the operation.

This usually happens during hot-reload or re-initialization flows where the previous layer was not properly unregistered, or when a setup routine runs twice. Check whether you need `network.upsert_layer(...)` instead of `add_layer(...)`, or explicitly remove the old layer first.

If you are composing multiple modules, namespace layer names with a module prefix to avoid collisions across independently developed components.",
        hints: &["Use `network.has_layer(name)` before adding, or call `upsert_layer`", "Namespace layer names per module to avoid cross-module collisions", "Ensure initialization routines are idempotent or guarded by a once-cell"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_LAYER_NOT_FOUND", "HHS_E_NETWORK_NETWORK_ALREADY_EXISTS"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const LAYER_LAYER_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(116),
        long_code: "HHS_E_LAYER_LAYER_NOT_FOUND",
        short_code: "E0116",
        title: "Referenced layer does not exist",
        short_description: "A lookup, update, or execution call referenced a layer name that is not registered in the network.",
        long_description: "Layer operations — running, inspecting, removing, or wiring dependencies — require the target layer to exist. This error signals that the name was not found in the registry at the moment of the call.

Common causes include races between registration and use, typos in layer names, and operations against a network that was rebuilt without re-adding all layers. Treat layer names as configuration keys and validate them at startup.

For dynamic scenarios, wrap lookups with `network.get_layer(name).ok_or(...)` and surface a clear user-facing error rather than letting the raw orchestration error propagate.",
        hints: &["Verify the layer is registered before calling into it", "Use `network.list_layers()` to confirm the expected set", "Guard layer lookups with explicit existence checks in user-facing code"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_DEPENDENCY_NOT_FOUND", "HHS_E_LAYER_LAYER_ALREADY_EXISTS", "HHS_E_NETWORK_LAYER_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const LAYER_TIMEOUT_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(117),
        long_code: "HHS_E_LAYER_TIMEOUT_EXCEEDED",
        short_code: "E0117",
        title: "Layer exceeded its execution timeout",
        short_description: "A layer did not complete within its configured timeout budget and was cancelled by the orchestrator.",
        long_description: "Each layer can declare a maximum execution time. When the timer fires, the orchestrator cancels any in-flight agents within the layer and reports this error. Downstream layers that depended on it are also marked as skipped or failed depending on the network policy.

Timeouts usually fire because an upstream LLM call stalled, a tool call blocked on I/O, or the layer fanned out more work than expected. Increase the budget only after confirming the underlying slowdown is legitimate; otherwise add per-agent timeouts, bounded retries, or circuit breakers inside the layer.

Enable tracing to see which specific agent consumed most of the budget before the timeout.",
        hints: &["Tune `layer.timeout` to the 95th-percentile observed duration plus headroom", "Add per-agent timeouts so one slow agent cannot starve the rest", "Use tracing spans to identify which agent is responsible"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_TIMEOUT_EXCEEDED", "HHS_E_ORCHESTRATION_WORKFLOW_TIMED_OUT", "HHS_E_LAYER_EXECUTION_FAILED"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const NETWORK_CYCLIC_DEPENDENCY: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(148),
        long_code: "HHS_E_NETWORK_CYCLIC_DEPENDENCY",
        short_code: "E0148",
        title: "Cycle detected in network dependency graph",
        short_description: "The network's layer dependency graph contains a cycle, which prevents a valid topological execution order.",
        long_description: "Networks execute layers in topological order based on declared dependencies. A cycle (A depends on B, B depends on A — possibly via intermediate layers) makes this ordering impossible, so graph construction is aborted.

To fix this, inspect the chain reported in the error message and break the cycle: split one of the participating layers in two, move shared state into an event bus instead of a dependency edge, or restructure the pipeline into feedforward stages. If you genuinely need feedback loops, model them with an iterative workflow rather than static layer deps.

Run `network.validate()` during construction to catch cycles at build time rather than at first execution.",
        hints: &["Break the cycle by splitting one participating layer", "Use an event bus for feedback instead of a dependency edge", "Call `network.validate()` early to surface cycles at build time"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_DEPENDENCY_NOT_FOUND", "HHS_E_NETWORK_INVALID_TOPOLOGY"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const NETWORK_INVALID_TOPOLOGY: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(149),
        long_code: "HHS_E_NETWORK_INVALID_TOPOLOGY",
        short_code: "E0149",
        title: "Network topology configuration is invalid",
        short_description: "The declared network topology (mesh, star, pipeline, etc.) failed validation against the supplied layers or routing rules.",
        long_description: "Each network topology imposes structural constraints: a star requires exactly one hub, a pipeline requires a linear chain with no forks, a council requires an odd number of voters, and so on. When the layers you registered do not satisfy those constraints, this error is raised during build.

Read the error detail to identify which rule was violated, then either restructure the layers to match or switch to a topology that fits your graph (for example, `mesh` accepts any DAG). Avoid hand-rolling topology strings — use the typed builders so the compiler can catch most mistakes at build time.

Mixed topologies are not supported within a single network; compose them by nesting networks instead.",
        hints: &["Use typed topology builders instead of raw strings", "Compose mixed shapes by nesting sub-networks", "Check the topology's documented constraints before building"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_CYCLIC_DEPENDENCY", "HHS_E_ORCHESTRATION_INVALID_WORKFLOW"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const NETWORK_LAYER_EXECUTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(150),
        long_code: "HHS_E_NETWORK_LAYER_EXECUTION_FAILED",
        short_code: "E0150",
        title: "Network-scoped layer execution failed",
        short_description: "A layer executed within a network returned an error, aborting the network run under fail-fast policy.",
        long_description: "When a network executes a layer as part of its pipeline, any layer-level failure is re-raised at the network level with both the layer name and the underlying cause. This wraps `LayerExecutionFailed` with network context so callers can react at the right granularity.

Fix the inner layer error first (see the wrapped message), then decide whether the network should tolerate this layer's failure. Networks support partial-failure policies: mark the layer as optional, provide a fallback branch, or retry with exponential backoff at the network level.

Use structured logging fields (`network`, `layer`, `agent`) so dashboards can aggregate failures per stage.",
        hints: &["Unwrap the inner layer error to locate the real cause", "Mark non-critical layers as optional in the network config", "Add network-level retry for transient layer failures"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_EXECUTION_FAILED", "HHS_E_ORCHESTRATION_NETWORK_EXECUTION_FAILED", "HHS_E_NETWORK_TIMEOUT_EXCEEDED"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const NETWORK_LAYER_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(151),
        long_code: "HHS_E_NETWORK_LAYER_NOT_FOUND",
        short_code: "E0151",
        title: "Layer not found in network",
        short_description: "The network was asked to operate on a layer name that is not part of its registered layer set.",
        long_description: "Network-level operations (run, skip, inspect) accept a layer name and dispatch into that layer's runtime. If the name is absent from the network's registry, this error fires.

This is typically a configuration drift issue: the caller used a stale name after the network was rebuilt, or the layer was removed without updating the caller. Always resolve layer names through a shared configuration source, and prefer strongly-typed handles over raw strings where possible.

For dynamic introspection, use `network.list_layers()` to enumerate the current set before attempting operations.",
        hints: &["Use `network.list_layers()` to confirm current layers", "Share layer names through typed config, not scattered string literals", "Validate layer handles at the edge of your public API"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_LAYER_LAYER_NOT_FOUND", "HHS_E_NETWORK_NETWORK_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

pub const NETWORK_NETWORK_ALREADY_EXISTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(152),
        long_code: "HHS_E_NETWORK_NETWORK_ALREADY_EXISTS",
        short_code: "E0152",
        title: "Network with this name is already registered",
        short_description: "An attempt was made to create a network whose name collides with an existing registered network.",
        long_description: "The orchestration runtime maintains a process-wide registry of networks keyed by name. Registering a second network with the same name would create ambiguity for later lookups, so the runtime refuses the second registration.

Ensure initialization code runs only once (guarded by `OnceCell` or a setup flag), or use `registry.upsert_network(...)` if you intentionally want to replace the previous definition. During hot-reload, explicitly deregister the old network before adding the new one.

Namespacing (e.g. `tenant_id/network_name`) helps avoid collisions in multi-tenant deployments.",
        hints: &["Guard network construction behind a `OnceCell` or init flag", "Use namespaced names in multi-tenant setups", "Deregister the old network before a hot-reload replaces it"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_NETWORK_NETWORK_NOT_FOUND", "HHS_E_ORCHESTRATION_WORKFLOW_ALREADY_EXISTS"],
        since_version: "0.4.0",
        category: ErrorCategory::Orchestration,
    };

mod core;
pub use core::*;

pub static ENTRIES: &[ErrorEntry] = &[
    EVENT_BUS_CHANNEL_CLOSED,
    EVENT_BUS_NO_SUBSCRIBERS,
    LAYER_DEPENDENCY_NOT_FOUND,
    LAYER_EXECUTION_FAILED,
    LAYER_LAYER_ALREADY_EXISTS,
    LAYER_LAYER_NOT_FOUND,
    LAYER_TIMEOUT_EXCEEDED,
    NETWORK_CYCLIC_DEPENDENCY,
    NETWORK_INVALID_TOPOLOGY,
    NETWORK_LAYER_EXECUTION_FAILED,
    NETWORK_LAYER_NOT_FOUND,
    NETWORK_NETWORK_ALREADY_EXISTS,
    NETWORK_NETWORK_NOT_FOUND,
    NETWORK_TIMEOUT_EXCEEDED,
    ORCHESTRATION_INVALID_WORKFLOW,
    ORCHESTRATION_LAYER_ERROR,
    ORCHESTRATION_NETWORK_ERROR,
    ORCHESTRATION_NETWORK_EXECUTION_FAILED,
    ORCHESTRATION_NETWORK_NOT_FOUND,
    ORCHESTRATION_WORKFLOW_ALREADY_EXISTS,
    ORCHESTRATION_WORKFLOW_NOT_FOUND,
    ORCHESTRATION_WORKFLOW_TIMED_OUT,
    PERMISSION_AGENT_NOT_REGISTERED,
    PERMISSION_DENIED,
];
