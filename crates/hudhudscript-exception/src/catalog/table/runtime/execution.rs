use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RUNTIME_AGENT_ALREADY_EXISTS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(224),
        long_code: "HHS_E_RUNTIME_AGENT_ALREADY_EXISTS",
        short_code: "E0224",
        title: "Agent with that name is already registered",
        short_description: "An attempt to spawn or register an agent failed because another agent with the same identifier is already live in the runtime.",
        long_description: "Agent identifiers must be unique within a runtime. This error fires when `spawn_agent` or an equivalent registration call is invoked with a name that is already taken by a running agent. It usually points to initialization code running twice, a hot-reload loop that did not tear down the previous instance, or a name-collision between two modules.

To fix, either choose a unique name, explicitly stop the existing agent before spawning a new one, or use a registry helper that generates unique suffixes. Agent teardown is not automatic on module reload unless you wire it up in your lifecycle hooks.

In distributed deployments, also confirm that the name is not being reused across nodes that share an agent namespace.",
        hints: &["Use `Agent.stop(name)` before re-registering with the same name", "Add a unique suffix (UUID, counter) when spawning dynamically", "Check for double-initialization in module setup code", "In hot-reload flows, clean up old agents in a teardown hook"],
        example_bad: Some("spawn_agent(\"logger\", LoggerAgent);
spawn_agent(\"logger\", LoggerAgent); // collision"),
        example_good: Some("if Agent.exists(\"logger\") { Agent.stop(\"logger\"); }
spawn_agent(\"logger\", LoggerAgent);"),
        see_also: &["HHS_E_RUNTIME_AGENT_NOT_FOUND", "HHS_E_RUNTIME_STATE_ERROR", "HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED"],
        since_version: "0.4.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_AGENT_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(225),
        long_code: "HHS_E_RUNTIME_AGENT_NOT_FOUND",
        short_code: "E0225",
        title: "Referenced agent does not exist",
        short_description: "A message was sent to or a query was made for an agent that is not currently registered in the runtime.",
        long_description: "Agent lookup by name failed: the runtime has no live agent under the given identifier. Possible causes include typos in the name, the agent not having been spawned yet (ordering bug), the agent having already exited, or the message originating from a different runtime scope.

Fix by spawning the agent before sending to it, checking `Agent.exists(name)` first, or using supervision so that dead agents are restarted automatically. If you expect the agent to be long-lived, log its lifecycle transitions to pinpoint when it disappeared.

In actor-style code, prefer keeping an `ActorRef` handle rather than looking up by string name — this avoids both races and typos.",
        hints: &["Check that the agent was spawned before sending to it", "Verify the name spelling exactly — lookup is case-sensitive", "Use `Agent.exists(name)` to probe before sending", "Hold an `ActorRef` instead of re-looking-up by name"],
        example_bad: Some("Agent.send(\"wroker\", msg); // typo: 'wroker' vs 'worker'"),
        example_good: Some("let worker = spawn_agent(\"worker\", Worker);
worker.send(msg);"),
        see_also: &["HHS_E_RUNTIME_AGENT_ALREADY_EXISTS", "HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_TASK_NOT_FOUND"],
        since_version: "0.4.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_CALL_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(227),
        long_code: "HHS_E_RUNTIME_CALL_ERROR",
        short_code: "E0227",
        title: "Error invoking a callable value",
        short_description: "A function, method, or callable object could not be invoked — arguments were invalid, the callee failed internally, or the value was not callable in context.",
        long_description: "This error is raised when the interpreter attempts to call a value and the call itself fails in a way not covered by a more specific error (such as arity mismatch or type error). Typical causes are: the callee is a native/builtin that returned a structured error, the receiver object is in a bad state, or an internal invariant of the dispatch layer was violated.

The attached message identifies the callee (by name or display form) and the underlying cause. Start by reading the inner message to categorize the failure, then verify the callee's contract — argument types, preconditions, and any required initialization.

For method calls, confirm that the receiver has not been mutated into an incompatible state between the lookup and the invocation.",
        hints: &["Read the inner message — it names the callee and the sub-error", "Verify argument types and counts match the callee's signature", "Check receiver state for method calls — was it modified since lookup?", "For native callables, consult host logs for structured error details"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_TYPE_ERROR", "HHS_E_RUNTIME_INVALID_OPERATION", "HHS_E_RUNTIME_PROPERTY_NOT_FOUND"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_CUSTOM: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(229),
        long_code: "HHS_E_RUNTIME_CUSTOM",
        short_code: "E0229",
        title: "Custom runtime error",
        short_description: "A generic runtime error carrying a caller-supplied message, used by builtins and embeddings that do not warrant a dedicated variant.",
        long_description: "This is a catch-all raised by library functions, builtins, or host extensions when they need to report a runtime failure that does not fit any structured variant. The attached string is the full message and is meant to be read by the end user or programmer directly.

Treat this as unstructured: the contents depend on whoever raised it. If you are a library author, prefer raising a more specific error variant when one exists so tooling can present it better. If you are a user seeing one, the attached message is your primary source of information — the stack trace will show where in script code it was raised.

Over time, frequent `Custom` usages should be promoted to dedicated error variants.",
        hints: &["Read the attached message — it is free-form and meant for humans", "Library authors: prefer a specific variant over `Custom` when possible", "Inspect the stack trace to locate the raising site", "File a ticket to promote recurring Custom messages to real variants"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_EXECUTION_FAILED", "HHS_E_RUNTIME_INVALID_OPERATION", "HHS_E_RUNTIME_THROW"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_DIVISION_BY_ZERO: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(230),
        long_code: "HHS_E_RUNTIME_DIVISION_BY_ZERO",
        short_code: "E0230",
        title: "Division by zero",
        short_description: "An arithmetic expression attempted to divide or take the modulus of a value by zero.",
        long_description: "HudHudScript follows strict integer division semantics: dividing or taking `%` with a zero divisor is a runtime error rather than producing `Infinity` or `NaN`. This applies to both `/` and `%` on integers; floating-point division follows IEEE-754 and produces `inf`/`nan` instead of raising.

Fix by guarding the division with an explicit zero check, or by using a saturating helper such as `checked_div(a, b)` that returns `null` on zero. If the divisor comes from user input, validate it at the boundary.

Keep in mind that a divisor you believe to be non-zero may still be zero when the arithmetic underflows, when parsing returns 0 as a default, or when a default-constructed struct field is read before assignment.",
        hints: &["Add an explicit `if divisor != 0` guard before dividing", "Use `checked_div` / `checked_mod` to get `null` instead of an error", "Validate user-supplied divisors at the input boundary", "Remember that `%` by zero raises the same error as `/` by zero"],
        example_bad: Some("let n = 10;
let d = 0;
let q = n / d;"),
        example_good: Some("let n = 10;
let d = 0;
let q = if d != 0 { n / d } else { 0 };"),
        see_also: &["HHS_E_RUNTIME_INVALID_OPERATION", "HHS_E_RUNTIME_TYPE_ERROR", "HHS_E_RUNTIME_CALL_ERROR"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_EXECUTION_FAILED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(231),
        long_code: "HHS_E_RUNTIME_EXECUTION_FAILED",
        short_code: "E0231",
        title: "Runtime execution failed",
        short_description: "The runtime aborted execution due to an internal failure not classified under a more specific variant.",
        long_description: "This error is surfaced when the runtime — generally the higher-level host runtime rather than the interpreter core — cannot complete a requested operation. The attached message explains why. Typical causes include a failed sub-service, a corrupted execution context, or an unhandled host-side failure during a script call.

Start with the attached message to identify the subsystem that failed. If the failure is reproducible from script code alone, it likely merits a more specific error variant and should be reported. If it only happens in embedding, check host-side logs for the originating error.

When seen alongside governance or security violations, resolve those first — this error may be a downstream consequence.",
        hints: &["Read the wrapped message to identify the failing subsystem", "Check host logs when running embedded for the root cause", "Resolve any governance or security errors first — they may cascade here", "Report reproducible cases so a dedicated variant can be added"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_STATE_ERROR", "HHS_E_RUNTIME_RESOURCE_ERROR", "HHS_E_RUNTIME_CUSTOM"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_INVALID_OPERATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(235),
        long_code: "HHS_E_RUNTIME_INVALID_OPERATION",
        short_code: "E0235",
        title: "Invalid operation for operand types",
        short_description: "An operator or builtin was applied to operands in a combination the runtime does not support.",
        long_description: "This error is raised when an operation is syntactically valid but semantically nonsense for the runtime types involved — for example, subtracting a string from an object, indexing into an integer, or calling a method that exists on some types but not the one provided. It differs from `TypeError` in that `TypeError` indicates a failed structural match, whereas `InvalidOperation` indicates that the operation itself does not apply.

The attached message names the operation and the offending operands. Fix by coercing or validating operand types, or by using a different operation that is defined for the actual types at hand.

In well-typed code this should be caught statically; encountering it dynamically usually indicates code paths the type checker could not prove.",
        hints: &["Read the message — it names the operation and the operand types", "Coerce operands to compatible types before the operation", "Consider whether the operation is even defined for these values", "Refactor toward types the checker can verify statically"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_TYPE_ERROR", "HHS_E_RUNTIME_DIVISION_BY_ZERO", "HHS_E_RUNTIME_CALL_ERROR"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_MODULE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(236),
        long_code: "HHS_E_RUNTIME_MODULE_ERROR",
        short_code: "E0236",
        title: "Module-level error",
        short_description: "A module failed to load, initialize, or respond to a lookup — its name and the sub-error are attached.",
        long_description: "Module errors cover a wide range of failures during module resolution, loading, evaluation of the top-level code, and symbol lookup. Typical causes include: the module file cannot be found; its source fails to parse or type-check; its initializer panics; or a lookup asks for a symbol the module does not export.

Fix by reading the attached message, which names the module and the sub-error. For file-not-found cases, verify the import path and module search roots. For initialization failures, inspect the top-level code of the module for errors. For missing exports, check for typos and for `pub`/export visibility.

Modules are cached after first load, so a failed initialization may leave a poisoned entry — restart the runtime or clear the cache if retries persist.",
        hints: &["Check the import path and module search roots", "Verify the symbol is actually exported (`pub`/export)", "Inspect the module's top-level code for init errors", "Clear the module cache if a poisoned entry is suspected"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_EXECUTION_FAILED", "HHS_E_RUNTIME_CALL_ERROR"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_OUT_OF_GAS: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(237),
        long_code: "HHS_E_RUNTIME_OUT_OF_GAS",
        short_code: "E0237",
        title: "Execution gas limit exceeded",
        short_description: "The script executed more steps than the configured gas budget allows, and was aborted by the sandbox.",
        long_description: "HudHudScript runs each script under a gas budget: every interpreted step (expression evaluation, call, loop iteration) charges some gas against a cap. When the cap is reached, execution is aborted with this error rather than allowed to run indefinitely. The message reports the limit that was exceeded.

Fix by making the script terminate sooner: add loop bounds, avoid quadratic work on user input, memoize repeated computation, or offload heavy work to native functions that charge gas differently. If the script is legitimately heavy, the embedder can raise the gas limit — scripts cannot change it themselves.

Gas exists to make execution bounded and predictable; hitting it repeatedly is a strong signal either that the script has a bug (infinite loop) or that the budget is wrong for the workload.",
        hints: &["Look for infinite or unbounded loops first", "Avoid quadratic algorithms on large inputs", "Memoize repeated work to reduce step count", "If legitimate, ask the embedder to raise the gas limit"],
        example_bad: Some("loop {} // never terminates"),
        example_good: Some("for _ in 0..1000 { /* bounded */ }"),
        see_also: &["HHS_E_RUNTIME_STACK_OVERFLOW", "HHS_E_RUNTIME_STM_TIMEOUT", "HHS_E_RUNTIME_RESOURCE_ERROR"],
        since_version: "0.4.10",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_PROPERTY_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(239),
        long_code: "HHS_E_RUNTIME_PROPERTY_NOT_FOUND",
        short_code: "E0239",
        title: "Property not found on value",
        short_description: "A property access (`obj.name`) failed because the receiver has no such field, method, or getter.",
        long_description: "Property access walks the receiver's layout (fields, methods, inherited slots) to find a match. When no match exists, this error is raised, naming both the property and the receiver type. Common causes: typos; accessing a property that only exists on a subclass; using a field that was removed in a refactor; or reaching through a `null`/`None` value.

Fix by checking the spelling and capitalization, confirming the property exists on the declared type, or using optional chaining (`obj?.name`) when the receiver may be absent. If the property is dynamic (e.g. from a map), use `obj.get(\"name\")` instead of dot access.

For objects that are the result of a downcast or dynamic dispatch, confirm the runtime type actually provides the property.",
        hints: &["Check spelling and capitalization of the property name", "Use optional chaining `obj?.name` when the receiver may be null", "For map-like values, prefer `get(\"key\")` over dot access", "Verify the runtime type of the receiver matches your expectation"],
        example_bad: Some("let p = { name: \"a\" };
print(p.nmae);"),
        example_good: Some("let p = { name: \"a\" };
print(p.name);"),
        see_also: &["HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_TYPE_ERROR", "HHS_E_RUNTIME_CALL_ERROR"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_RESOURCE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(240),
        long_code: "HHS_E_RUNTIME_RESOURCE_ERROR",
        short_code: "E0240",
        title: "Resource access failed",
        short_description: "A resource-backed operation (file, socket, handle, capability) failed at runtime.",
        long_description: "Resource errors cover failures in the host-provided resource layer: opening a file, reading/writing a stream, acquiring a handle, or interacting with a capability. The attached message identifies the resource and the underlying cause (permission denied, not found, quota exceeded, etc.).

Fix by addressing the underlying cause: ensure the resource exists and is accessible, handle quota/limit conditions, and release handles when done. Many resources are capability-gated, so check your constitution/governance settings if access is unexpectedly denied.

In long-running scripts, resource leaks accumulate silently and eventually manifest here — use `defer` or RAII-style wrappers to guarantee cleanup.",
        hints: &["Read the wrapped message to classify the sub-error", "Check file existence, permissions, and quotas at the host level", "Release handles promptly — use `defer` for cleanup", "Governance may be denying access — check the constitution"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_GOVERNANCE_VIOLATION", "HHS_E_RUNTIME_SECURITY_VIOLATION", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_SECURITY_VIOLATION: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(242),
        long_code: "HHS_E_RUNTIME_SECURITY_VIOLATION",
        short_code: "E0242",
        title: "Security sandbox violation",
        short_description: "An operation was blocked by the sandbox layer because it would break one of the runtime's structural security invariants.",
        long_description: "Security violations indicate that a script attempted something the sandbox is structurally forbidden to allow — for example, crossing into a foreign memory area, escaping the capability graph, or invoking a builtin in a context where it is disallowed. The message names the subsystem and the specific invariant.

These are distinct from governance violations: governance is declarative policy from the host, while security is the sandbox's own non-negotiable guarantee. If you see one, either the script is doing something genuinely unsafe or there is a bug in the sandbox — the latter should be reported.

Fix by removing the offending operation, or by requesting the needed capability through a safe, host-approved channel.",
        hints: &["Read the message — it identifies the invariant and subsystem", "Use only sanctioned APIs to access restricted functionality", "Request host capabilities rather than bypassing the sandbox", "If the script seems legitimate, report as a possible sandbox bug"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_GOVERNANCE_VIOLATION", "HHS_E_RUNTIME_RESOURCE_ERROR", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_STATE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(244),
        long_code: "HHS_E_RUNTIME_STATE_ERROR",
        short_code: "E0244",
        title: "Invalid runtime state",
        short_description: "An operation was invoked on a runtime subsystem in a state that does not permit it.",
        long_description: "This error fires when a subsystem (scheduler, actor registry, module cache, etc.) receives a request in a state where the request is not valid — for example, using a runtime after shutdown, re-initializing an already-initialized component, or advancing a task that is in a terminal state.

Fix by sequencing lifecycle calls correctly: initialize before use, do not reuse a stopped runtime, and do not drive tasks past their terminal states. The message names the subsystem and the state violation.

When embedding, ensure your host's lifecycle management matches the runtime's expectations. Double-shutdowns and re-initializations are the most common sources.",
        hints: &["Do not use a runtime or subsystem after shutdown", "Check that initialization has completed before making calls", "Avoid double-initialization of lifecycle-managed components", "Read the message — it names the subsystem and invalid transition"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_EXECUTION_FAILED", "HHS_E_RUNTIME_AGENT_ALREADY_EXISTS", "HHS_E_RUNTIME_RESOURCE_ERROR"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_TASK_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(247),
        long_code: "HHS_E_RUNTIME_TASK_NOT_FOUND",
        short_code: "E0247",
        title: "Task handle does not exist",
        short_description: "A lookup by task handle failed because no such task is registered with the runtime.",
        long_description: "The scheduler keeps a registry of live tasks keyed by handle. This error is raised when an operation (cancel, join, query status) is issued for a handle that does not appear in the registry — typically because the task has already completed and been reaped, or because the handle was created on a different runtime.

Fix by checking task liveness before operating on a handle, avoiding use of handles after completion, and keeping handles scoped to the runtime that created them.

If you are joining a task, prefer using a dedicated join-handle type that cannot be used after completion, or a `JoinSet` that tracks the set of live children for you.",
        hints: &["Do not use a task handle after the task has completed", "Keep task handles scoped to one runtime instance", "Prefer `JoinSet` or structured join handles over raw IDs", "Probe status before issuing an operation if the handle may be stale"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_AGENT_NOT_FOUND", "HHS_E_ASYNC_RUNTIME_PROMISE_NOT_FOUND", "HHS_E_RUNTIME_STATE_ERROR"],
        since_version: "0.4.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_TOOL_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(249),
        long_code: "HHS_E_RUNTIME_TOOL_ERROR",
        short_code: "E0249",
        title: "Tool invocation failed",
        short_description: "An invocation of an external tool (agent tool, plugin, or host-provided action) returned an error.",
        long_description: "HudHudScript agents can call registered tools — functions provided by the host for side-effects like HTTP, database, or file access. When a tool returns an error, the runtime surfaces it here, with the tool's own message attached.

Fix by addressing the underlying cause reported by the tool, adjusting input arguments, or adding a fallback path when the tool is allowed to fail. Tool errors should usually be caught at the call site so the agent can reason about the failure and retry or choose a different action.

If the same tool consistently errors, verify that it is properly registered, that its constitution/governance permissions are granted, and that its configuration (credentials, endpoints) is correct.",
        hints: &["Read the tool's own message for the precise failure", "Catch tool errors at the call site so the agent can adapt", "Validate tool input arguments against its schema", "Verify tool registration, credentials, and governance permissions"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_CALL_ERROR", "HHS_E_RUNTIME_GOVERNANCE_VIOLATION", "HHS_E_RUNTIME_RESOURCE_ERROR"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_TYPE_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(250),
        long_code: "HHS_E_RUNTIME_TYPE_ERROR",
        short_code: "E0250",
        title: "Runtime type mismatch",
        short_description: "A value's runtime type did not match what the operation expected — expected and actual types are both reported.",
        long_description: "This error is raised when the interpreter performs an operation that requires a specific type (or type class) and the actual value has a different type. The message names the context, the expected type, and the type actually found. Typical causes are passing the wrong argument to a function, receiving a value from a dynamic source (JSON, user input) without validating its shape, or accessing a field through a base trait where the concrete implementation does not match.

Fix by validating or coercing values at the boundary, adding explicit type guards, or fixing the caller to pass the correct type. Where possible, lean on the static type checker to catch these at compile time.

For values crossing the dynamic/static boundary (e.g., deserialized JSON), prefer schema-driven parsing so mismatches are detected at parse time rather than deep inside business logic.",
        hints: &["Validate or coerce values at the boundary (parse, unmarshal, etc.)", "Add explicit type guards before operations that require a specific type", "Lean on the static type checker when possible", "Prefer schema-driven deserialization over ad-hoc field access"],
        example_bad: Some("fn add(a, b) { return a + b; }
add(1, \"two\");"),
        example_good: Some("fn add(a: int, b: int) -> int { return a + b; }
add(1, 2);"),
        see_also: &["HHS_E_RUNTIME_INVALID_OPERATION", "HHS_E_RUNTIME_CALL_ERROR", "HHS_E_RUNTIME_PROPERTY_NOT_FOUND"],
        since_version: "0.1.0",
        category: ExceptionCategory::Runtime,
    };
