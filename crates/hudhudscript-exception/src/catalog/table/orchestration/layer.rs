use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const LAYER_DEPENDENCY_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(113),
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
        category: ExceptionCategory::Orchestration,
    };

pub const LAYER_EXECUTION_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(114),
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
        category: ExceptionCategory::Orchestration,
    };

pub const LAYER_LAYER_ALREADY_EXISTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(115),
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
        category: ExceptionCategory::Orchestration,
    };

pub const LAYER_LAYER_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(116),
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
        category: ExceptionCategory::Orchestration,
    };

pub const LAYER_TIMEOUT_EXCEEDED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(117),
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
        category: ExceptionCategory::Orchestration,
    };
