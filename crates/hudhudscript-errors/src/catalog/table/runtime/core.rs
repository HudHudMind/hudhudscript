use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};
pub const RUNTIME_PROMISE_REJECTED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(238),
        long_code: "HHS_E_RUNTIME_PROMISE_REJECTED",
        short_code: "E0238",
        title: "Promise rejection surfaced in interpreter",
        short_description: "A rejected promise was observed by the interpreter and propagated as a runtime error to the awaiting code.",
        long_description: "When `await` observes a rejected promise and no `try`/`catch` covers the site, the rejection value is wrapped in this runtime error and unwinds the stack. This variant is the interpreter's view of promise rejection; the promise layer has its own `PromiseRejected` that this wraps.

Handle it by wrapping the `await` in a `try`/`catch`, or by chaining a `.catch(...)` onto the promise before awaiting. Top-level awaits should always be guarded so rejections become structured diagnostics rather than crashing the task.

The attached value is exactly what the producer passed to `reject(...)` — typically a string or an error-like object.",
        hints: &["Wrap the `await` in `try { ... } catch (e) { ... }`", "Use `.catch(handler)` to convert rejection into a fallback value", "Always guard top-level awaits to avoid silent task failure", "Log the rejection value to identify the producer"],
        example_bad: Some("let d = await fetch(\"x\");"),
        example_good: Some("try { let d = await fetch(\"x\"); } catch (e) { log(e); }"),
        see_also: &["HHS_E_PROMISE_REJECTED", "HHS_E_RUNTIME_THROW", "HHS_E_ASYNC_RUNTIME_PROMISE_NOT_FOUND"],
        since_version: "0.4.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_PROPERTY_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(239),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_RESOURCE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(240),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_RETURN: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(241),
        long_code: "HHS_E_RUNTIME_RETURN",
        short_code: "E0241",
        title: "`return` used outside a function",
        short_description: "A `return` statement escaped all enclosing function frames, reaching the top level — no function was there to receive it.",
        long_description: "Internally, `return value` is modeled as a runtime unwind signal that propagates upward until a function-call frame catches it and treats its payload as the call's result. If execution reaches a `return` where no function frame is above it — for example, `return` at the top level of a script — the signal escapes and surfaces as this error.

Users almost never see this at runtime because parsers reject top-level `return`. When it does appear it is usually from code that was constructed dynamically (eval-like features) or a closure reaching outside its originating function.

Fix by wrapping the code in a function, or by replacing `return` with the appropriate top-level expression.",
        hints: &["Only use `return` inside `fn` / `async fn` bodies", "At the top level, use the last expression as the script's value", "Check dynamically generated code for stray `return`", "Closures that `return` must be invoked inside their host function"],
        example_bad: Some("return 1; // top level"),
        example_good: Some("fn main() { return 1; }"),
        see_also: &["HHS_E_RUNTIME_BREAK", "HHS_E_RUNTIME_CONTINUE", "HHS_E_RUNTIME_YIELD"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_SECURITY_VIOLATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(242),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_STACK_OVERFLOW: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(243),
        long_code: "HHS_E_RUNTIME_STACK_OVERFLOW",
        short_code: "E0243",
        title: "Call stack overflow",
        short_description: "The call stack grew beyond its depth limit, usually because of unbounded recursion.",
        long_description: "Each call pushes a frame onto the interpreter's stack; when the stack exceeds its configured depth, execution is aborted with this error. The overwhelming cause is recursion without a base case, or recursion whose base case is unreachable for the input given.

Fix by ensuring every recursive path has a terminating base case and that recursive arguments strictly converge toward it. For deep but principled recursion (e.g., tree traversals on deeply nested data), convert to an iterative loop with an explicit stack, or increase the limit via the embedder if genuinely needed.

Stack overflow can also indicate mutual recursion that never bottoms out — trace the call cycle to find where convergence fails.",
        hints: &["Verify recursive base cases are reachable for all inputs", "Convert deep recursion to iteration with an explicit stack", "Watch for mutual recursion that forms a non-terminating cycle", "Embedder can raise the stack limit for legitimately deep workloads"],
        example_bad: Some("fn f(n) { return f(n + 1); }
f(0);"),
        example_good: Some("fn f(n) { if n >= 100 { return n; } return f(n + 1); }
f(0);"),
        see_also: &["HHS_E_RUNTIME_OUT_OF_GAS", "HHS_E_RUNTIME_CALL_ERROR", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_STATE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(244),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_STM_MAX_RETRIES_EXCEEDED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(245),
        long_code: "HHS_E_RUNTIME_STM_MAX_RETRIES_EXCEEDED",
        short_code: "E0245",
        title: "STM transaction exceeded retry limit",
        short_description: "An `atomically` block conflicted with other transactions too many times in a row and was aborted to prevent livelock.",
        long_description: "HudHudScript's STM is optimistic: each transaction runs on a snapshot, then validates and commits. On a conflict, the transaction retries. To prevent livelock, the runtime caps consecutive retries; when the cap is hit, this error is raised rather than looping forever.

Livelock usually indicates high write contention on a small set of TVars. To fix, reduce contention by splitting hot TVars, ordering operations so writes do not overlap, reducing transaction size (shorter transactions conflict less often), or using coarser locking if the workload is truly write-heavy.

Keep in mind that `await` is forbidden inside `atomically` — if you suspect long transactions, audit for blocking operations that should live outside the atomic block.",
        hints: &["Split hot TVars so writers do not all collide on the same cell", "Shrink the atomic block — do expensive work outside it", "Never `await` inside `atomically` (it is forbidden)", "Consider an actor/message pattern for extreme write contention"],
        example_bad: Some("// tight loop of conflicting writers on a single TVar
atomically { counter.set(counter.get() + 1); }"),
        example_good: Some("// shard the counter to reduce contention
atomically { shards[tid % N].update(|v| v + 1); }"),
        see_also: &["HHS_E_RUNTIME_STM_TIMEOUT", "HHS_E_RUNTIME_STATE_ERROR", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.4.10",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_STM_TIMEOUT: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(246),
        long_code: "HHS_E_RUNTIME_STM_TIMEOUT",
        short_code: "E0246",
        title: "STM transaction timed out",
        short_description: "An `atomically` block ran longer than its configured time limit and was aborted.",
        long_description: "In addition to a retry cap, STM transactions have a wall-clock timeout to bound their worst-case latency. When a transaction (including its retries) exceeds the limit, the runtime aborts it with this error. The message reports the elapsed time and the configured limit.

The root cause is usually one of: the transaction body is genuinely slow (too much work inside the atomic block), extreme contention is causing many retries, or the timeout is mis-configured for the workload.

Fix by shrinking the transaction, reducing contention, or — if the workload is legitimate and bounded — raising the STM timeout via the embedder. Remember that `await` is forbidden inside `atomically`, so unexpected latency should not come from async operations.",
        hints: &["Shrink the atomic block — move expensive computation outside it", "Reduce contention (see: StmMaxRetriesExceeded)", "Never `await` inside `atomically`", "If legitimate, ask the embedder to raise the STM timeout"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_STM_MAX_RETRIES_EXCEEDED", "HHS_E_RUNTIME_OUT_OF_GAS", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.4.10",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_TASK_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(247),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_THROW: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(248),
        long_code: "HHS_E_RUNTIME_THROW",
        short_code: "E0248",
        title: "Uncaught user exception",
        short_description: "A `throw` expression's value propagated past every `try`/`catch` and reached the top of a task.",
        long_description: "`throw value` raises a user exception. The interpreter unwinds frames looking for a `catch` that can receive the value; if none is found before the stack is empty (or before the task's entry point), the exception becomes this error with the thrown value attached.

The fix is either to wrap the throwing code in `try { ... } catch (e) { ... }`, or to let the exception propagate deliberately and handle it at a higher level. Every task should have a top-level handler so unexpected throws become structured log entries instead of bringing down the task silently.

The attached value can be any HudHudScript value: strings, records, or error objects are all common. Inspect it in the catch handler using normal property access.",
        hints: &["Wrap throwing code in `try { ... } catch (e) { ... }`", "Install a top-level handler in every spawned task", "The thrown value can be any type — inspect it with field access", "Prefer structured error objects over bare strings for catch logic"],
        example_bad: Some("fn f() { throw \"bad\"; }
f();"),
        example_good: Some("fn f() { throw \"bad\"; }
try { f(); } catch (e) { log(e); }"),
        see_also: &["HHS_E_RUNTIME_PROMISE_REJECTED", "HHS_E_RUNTIME_CUSTOM", "HHS_E_PROMISE_REJECTED"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_TOOL_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(249),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_TYPE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(250),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_UNDEFINED_VARIABLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(251),
        long_code: "HHS_E_RUNTIME_UNDEFINED_VARIABLE",
        short_code: "E0251",
        title: "Reference to undefined variable",
        short_description: "The interpreter tried to read a variable that has not been declared in any visible scope.",
        long_description: "At runtime, an expression referenced a name for which no binding exists. The variable might be misspelled, declared in a different scope, used before its `let`/`const`/`var` declaration is reached, or removed by a refactor. The interpreter reports the offending name and source position.

To fix this, declare the variable before use, correct the spelling, or import it from the module that defines it. Unlike a parse error, this is detected only when execution reaches the offending expression — branches that are never taken will not raise it.

For module-qualified references, verify that the module is actually imported and that the symbol is exported from it.",
        hints: &["Check spelling and capitalization of the identifier", "Verify the variable is declared in an enclosing scope, not a sibling block", "If declared with `let`, the binding only exists after its declaration", "For module symbols, verify the `import` and that it is exported"],
        example_bad: Some("fn main() {
    println(usrname);
}"),
        example_good: Some("fn main() {
    let username = \"alice\";
    println(username);
}"),
        see_also: &["HHS_E_RUNTIME_UNINITIALIZED_VARIABLE", "HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED", "HHS_E_RUNTIME_PROPERTY_NOT_FOUND"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_UNINITIALIZED_VARIABLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(252),
        long_code: "HHS_E_RUNTIME_UNINITIALIZED_VARIABLE",
        short_code: "E0252",
        title: "Access before initialization (temporal dead zone)",
        short_description: "A variable was read before its declaration in the same scope executed — the binding exists but has no value yet.",
        long_description: "HudHudScript implements a temporal dead zone (TDZ) for `let`/`const` bindings: the binding is considered to exist from the start of its enclosing scope, but any read before its declaration statement runs is an error. This catches code that accidentally uses a name that will be declared later as if it were a global or a hoisted `var`.

Fix by moving the read below the declaration, or by declaring the variable earlier in the scope with an initial value. Do not rely on hoisting — `let`/`const` bindings are not initialized to a default.

This error is distinct from `UndefinedVariable`: the binding does exist here, it just has not been assigned yet.",
        hints: &["Move the use below the declaration", "Initialize the variable when declaring it, not later", "Do not rely on hoisting — `let`/`const` are not hoisted like `var`", "Check nested blocks: TDZ applies per scope"],
        example_bad: Some("fn main() {
    println(x);
    let x = 1;
}"),
        example_good: Some("fn main() {
    let x = 1;
    println(x);
}"),
        see_also: &["HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_IMMUTABLE_VARIABLE", "HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_VARIABLE_ALREADY_DEFINED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(253),
        long_code: "HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED",
        short_code: "E0253",
        title: "Variable already defined in scope",
        short_description: "A declaration collided with an existing binding of the same name in the same lexical scope.",
        long_description: "HudHudScript forbids re-declaring a name within the same scope. The second `let`/`const`/`var` hits this error because silent shadowing in the same block is a bug magnet. Shadowing in a nested inner scope is still allowed — only collisions in the same block are rejected.

Fix by choosing a different name, by moving the second declaration into an inner block where shadowing is permitted, or by converting the first declaration to `var` and reassigning rather than redeclaring.

If you are converting code from a language that allows free re-declaration, the fix is almost always to rename the second binding.",
        hints: &["Pick a different name for the second declaration", "Move the second declaration into a nested block if shadowing is intended", "Use reassignment with `var` instead of re-declaring", "Check for accidental copies/pastes that duplicated the declaration"],
        example_bad: Some("let x = 1;
let x = 2;"),
        example_good: Some("let x = 1;
{ let x = 2; /* inner shadow */ }"),
        see_also: &["HHS_E_RUNTIME_IMMUTABLE_VARIABLE", "HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_UNINITIALIZED_VARIABLE"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_YIELD: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(254),
        long_code: "HHS_E_RUNTIME_YIELD",
        short_code: "E0254",
        title: "`yield` used outside a generator",
        short_description: "A `yield` expression escaped all generator frames, meaning it was executed where no generator contained it.",
        long_description: "`yield` is implemented as a runtime unwind signal that is caught by an enclosing generator frame, which pauses the generator and hands the yielded value to its consumer. If `yield` runs where no generator is active — for example in a plain function — the signal escapes and surfaces here.

This is almost always caught statically: `yield` outside a generator is a parse or type error. Seeing it at runtime usually points to dynamically generated code, or a closure that yields but was called outside its host generator.

Fix by wrapping the code in a generator (`gen fn` or equivalent) or by replacing `yield` with `return` if a one-shot result is what you meant.",
        hints: &["Only use `yield` inside generator functions", "Closures that `yield` must run inside their host generator", "If you meant a single result, use `return` instead", "Report to the compiler team if it slipped past static checks"],
        example_bad: Some("fn f() { yield 1; } // not a generator"),
        example_good: Some("gen fn g() { yield 1; yield 2; }"),
        see_also: &["HHS_E_RUNTIME_BREAK", "HHS_E_RUNTIME_CONTINUE", "HHS_E_RUNTIME_RETURN"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };
