use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const NETWORK_CYCLIC_DEPENDENCY: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(148),
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
        category: ExceptionCategory::Orchestration,
    };

pub const NETWORK_INVALID_TOPOLOGY: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(149),
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
        category: ExceptionCategory::Orchestration,
    };

pub const NETWORK_LAYER_EXECUTION_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(150),
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
        category: ExceptionCategory::Orchestration,
    };

pub const NETWORK_LAYER_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(151),
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
        category: ExceptionCategory::Orchestration,
    };

pub const NETWORK_NETWORK_ALREADY_EXISTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(152),
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
        category: ExceptionCategory::Orchestration,
    };

pub const NETWORK_NETWORK_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(153),
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
        category: ExceptionCategory::Orchestration,
    };

pub const NETWORK_TIMEOUT_EXCEEDED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(154),
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
        category: ExceptionCategory::Orchestration,
    };
