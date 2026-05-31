use crate::catalog::{ErrorCategory, ErrorCode, ErrorEntry};

pub const ASYNC_RUNTIME_PROMISE_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(8),
        long_code: "HHS_E_ASYNC_RUNTIME_PROMISE_NOT_FOUND",
        short_code: "E0008",
        title: "Promise handle not found in runtime",
        short_description: "The async runtime could not locate a promise referenced by its handle, usually because it was already consumed or never registered.",
        long_description: "The async runtime keeps a registry of live promises keyed by an internal handle. When the interpreter tried to resolve, reject, or await a promise, the runtime found no entry for the given handle. This usually means the promise was already settled and garbage-collected, was created on a different runtime instance, or the handle was fabricated or corrupted.

Fix this by ensuring each promise is awaited at most once, that promise handles are not shared across distinct runtime contexts, and that foreign code using the FFI does not hold stale handles after a runtime restart.

If you are embedding HudHudScript, verify that the runtime whose `Executor` created the promise is the same one now being queried — mixing runtimes is not supported.",
        hints: &["Do not await the same promise twice — the second await sees no handle", "Ensure the async runtime has not been torn down before settling the promise", "Check that you are not mixing promises from different runtime instances", "Avoid storing raw handles across suspend points if you cannot guarantee liveness"],
        example_bad: Some("let p = async_task();
let a = await p;
let b = await p; // handle already consumed"),
        example_good: Some("let p = async_task();
let a = await p;
// reuse the resolved value, not the promise
let b = a;"),
        see_also: &["HHS_E_ASYNC_RUNTIME_RUNTIME_ERROR", "HHS_E_PROMISE_RECEIVER_DROPPED", "HHS_E_RUNTIME_PROMISE_REJECTED"],
        since_version: "0.4.0",
        category: ErrorCategory::Runtime,
    };

pub const ASYNC_RUNTIME_RUNTIME_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(9),
        long_code: "HHS_E_ASYNC_RUNTIME_RUNTIME_ERROR",
        short_code: "E0009",
        title: "Async runtime internal failure",
        short_description: "The async executor reported an internal error while scheduling, polling, or driving a task to completion.",
        long_description: "This is a catch-all error surfaced when the async runtime itself — not user code — encounters a failure. Causes include a dropped waker, a panicking future, reactor I/O errors, or an invariant violation inside the executor. The wrapped message from the runtime is attached for diagnosis.

First, inspect the inner message to classify the fault: I/O errors usually indicate environmental issues (sockets, files); panics suggest a bug in a native task; shutdown errors mean the runtime was stopped while tasks were still live.

If the error is reproducible from safe HudHudScript code, it is likely a runtime bug and should be reported. In embedding contexts, verify that the host is not tearing down the runtime while scripts are still executing.",
        hints: &["Read the wrapped message — it identifies the sub-failure precisely", "Avoid shutting the async runtime down while tasks are still in flight", "If triggered by plain script code, file a bug against hudhudscript-async", "Check host logs for panics from native futures registered via FFI"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_ASYNC_RUNTIME_TASK_SPAWN_FAILED", "HHS_E_ASYNC_RUNTIME_PROMISE_NOT_FOUND", "HHS_E_RUNTIME_EXECUTION_FAILED"],
        since_version: "0.4.0",
        category: ErrorCategory::Runtime,
    };

pub const ASYNC_RUNTIME_TASK_SPAWN_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(10),
        long_code: "HHS_E_ASYNC_RUNTIME_TASK_SPAWN_FAILED",
        short_code: "E0010",
        title: "Failed to spawn async task",
        short_description: "The runtime refused to spawn a new async task, typically because it is shutting down or has exhausted its worker capacity.",
        long_description: "When `async fn` invocations or `spawn(...)` calls create a new task, the executor must accept it onto one of its worker threads. This error is returned when the executor rejects the submission — most commonly because shutdown has already been initiated, a bounded task queue is full, or the host embedding has imposed a spawn quota.

To fix: ensure you are not spawning tasks after the runtime has begun shutdown, throttle producers so the task queue cannot grow unbounded, and verify that any embedder-imposed limits match expected workload.

Inside a sandbox, this can also trip if the sandbox forbids further spawning after a gas-like budget has been consumed.",
        hints: &["Do not call `spawn` from shutdown hooks or after `runtime.stop()`", "Throttle producers to avoid overflowing bounded task queues", "Check sandbox/embedder policies that cap concurrent task count", "Prefer joining existing tasks to unbounded fan-out"],
        example_bad: Some("for i in 0..1_000_000 {
    spawn(async { work(i) });
}"),
        example_good: Some("let sem = Semaphore::new(64);
for i in 0..1_000_000 {
    let permit = await sem.acquire();
    spawn(async { work(i); drop(permit); });
}"),
        see_also: &["HHS_E_ASYNC_RUNTIME_RUNTIME_ERROR", "HHS_E_RUNTIME_RESOURCE_ERROR", "HHS_E_RUNTIME_OUT_OF_GAS"],
        since_version: "0.4.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_AGENT_ALREADY_EXISTS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(224),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_AGENT_NOT_FOUND: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(225),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_BREAK: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(226),
        long_code: "HHS_E_RUNTIME_BREAK",
        short_code: "E0226",
        title: "`break` used outside a loop",
        short_description: "A `break` statement escaped the innermost loop without being caught, meaning it was executed where no loop contained it.",
        long_description: "Internally, `break` is modeled as a runtime unwind signal: when executed, it propagates upward until a loop frame catches it. If execution of `break` reaches a place where no loop is active — for example, a `break` at file scope, inside a plain function body, or inside a closure that was called outside its originating loop — the signal escapes and surfaces as this error.

Most users will never see this at runtime because the type checker and control-flow analysis reject `break` outside loops statically. If you do see it, check for closures that capture `break`-like semantics, macros that expand to stray `break`, or `break` in an `if` arm that sits in a function but not inside a loop.

To fix, wrap the offending code in a loop, or replace `break` with `return` if you meant to exit a function.",
        hints: &["Only use `break` inside `loop`, `while`, or `for`", "If you meant to exit a function, use `return` instead", "Check for closures that use `break` but are invoked outside their loop", "Static analysis should catch this — report as a bug if it slipped through"],
        example_bad: Some("fn main() {
    break; // no enclosing loop
}"),
        example_good: Some("fn main() {
    loop {
        break;
    }
}"),
        see_also: &["HHS_E_RUNTIME_CONTINUE", "HHS_E_RUNTIME_RETURN", "HHS_E_RUNTIME_YIELD"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_CALL_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(227),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_CONTINUE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(228),
        long_code: "HHS_E_RUNTIME_CONTINUE",
        short_code: "E0228",
        title: "`continue` used outside a loop",
        short_description: "A `continue` statement escaped the innermost loop without being caught, meaning it was executed where no loop contained it.",
        long_description: "`continue` is implemented as a runtime unwind signal that propagates to the nearest enclosing loop, which catches it and restarts its next iteration. If no loop is active at the point `continue` executes, the signal escapes and this error surfaces.

This almost always indicates either a `continue` outside any loop (caught statically in most cases) or a closure that uses `continue` and is invoked outside the loop where it was constructed. The fix is to place `continue` inside a loop, or rethink the control flow.

Static analysis rejects `continue` outside a loop at parse or type-check time, so seeing this at runtime typically indicates a corner case such as dynamic code generation or a bug in the analyzer.",
        hints: &["Only use `continue` inside `loop`, `while`, or `for`", "Closures that use `continue` must be called inside their source loop", "If you meant to skip a branch of code, use an `if/else` instead", "Report to the compiler team if it slipped past static checks"],
        example_bad: Some("fn main() {
    continue; // no enclosing loop
}"),
        example_good: Some("for i in 0..10 {
    if i % 2 == 0 { continue; }
    print(i);
}"),
        see_also: &["HHS_E_RUNTIME_BREAK", "HHS_E_RUNTIME_RETURN", "HHS_E_RUNTIME_YIELD"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_CUSTOM: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(229),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_DIVISION_BY_ZERO: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(230),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_EXECUTION_FAILED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(231),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_GOVERNANCE_VIOLATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(232),
        long_code: "HHS_E_RUNTIME_GOVERNANCE_VIOLATION",
        short_code: "E0232",
        title: "Action violates active constitution",
        short_description: "An action was blocked by the governance layer because it violates a rule in the currently enforced constitution.",
        long_description: "HudHudScript's governance system lets hosts attach a constitution — a declarative set of rules — to a running script. When a script attempts something the constitution forbids (for example, calling a denied tool, accessing a restricted resource, or exceeding a declared capability), the runtime aborts with this error, naming the constitution and the specific rule that was violated.

Fix by either changing the script to stay within the allowed actions, or by updating the constitution to grant the needed capability — the latter requires authorization and is not something scripts can do themselves.

This error is distinct from `SecurityViolation`: governance is user- or deployment-defined policy, whereas security violations are structural sandbox breaks.",
        hints: &["Read the error: it names the constitution and the violated rule", "Adjust the script to stay within allowed capabilities", "Request a constitution update through the appropriate governance channel", "Do not try to work around governance — it is enforced at every step"],
        example_bad: None,
        example_good: None,
        see_also: &["HHS_E_RUNTIME_SECURITY_VIOLATION", "HHS_E_RUNTIME_EXECUTION_FAILED", "HHS_E_RUNTIME_RESOURCE_ERROR"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_IMMUTABLE_VARIABLE: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(233),
        long_code: "HHS_E_RUNTIME_IMMUTABLE_VARIABLE",
        short_code: "E0233",
        title: "Assignment to immutable variable",
        short_description: "An assignment targeted a variable declared as immutable (via `let` or `const`) rather than mutable (`var` or `let mut`).",
        long_description: "HudHudScript distinguishes mutable and immutable bindings at declaration. `const` and plain `let` create immutable bindings; only `var` (or `let mut`, depending on dialect) allows reassignment. Attempting to assign to an immutable binding is a runtime error when it escapes static analysis, or reported here when discovered dynamically.

To fix, either declare the variable as mutable from the start, or restructure the code to avoid reassignment (e.g., introduce a new binding with a different name). Preferring immutability is idiomatic and helps the optimizer.

Note that immutability of the binding is separate from mutability of the referent: an immutable binding to a list still allows mutating the list's contents via methods, because only the binding is frozen.",
        hints: &["Declare the variable with `var` (or `let mut`) if you need reassignment", "Prefer creating a new binding with `let` over mutating an old one", "Immutable binding still allows mutating fields of the referent", "Check for accidental shadowing when you intended to reassign"],
        example_bad: Some("let x = 1;
x = 2;"),
        example_good: Some("var x = 1;
x = 2;"),
        see_also: &["HHS_E_RUNTIME_VARIABLE_ALREADY_DEFINED", "HHS_E_RUNTIME_UNDEFINED_VARIABLE", "HHS_E_RUNTIME_UNINITIALIZED_VARIABLE"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_INDEX_OUT_OF_BOUNDS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(234),
        long_code: "HHS_E_RUNTIME_INDEX_OUT_OF_BOUNDS",
        short_code: "E0234",
        title: "Index out of bounds",
        short_description: "An index expression addressed a position outside the valid range `[0, length)` of the collection.",
        long_description: "HudHudScript collections are bounds-checked at every access. Reading or writing `coll[i]` with `i < 0` or `i >= coll.len()` raises this error. The message carries both the offending index and the collection's length for debugging.

Fix by validating the index before use, by using a safe accessor like `coll.get(i)` that returns `null` for out-of-range indices, or by iterating with a constructs that cannot go out of range (e.g., `for x in coll`).

Off-by-one bugs at the edges (`coll[coll.len()]`) and reliance on signed arithmetic that can produce negative indices are the most common sources.",
        hints: &["Use `coll.get(i)` for safe access that returns `null` on OOB", "Prefer iteration over manual indexing when possible", "Check for off-by-one: the last valid index is `len - 1`", "Negative indices are not wrapped — they raise this error"],
        example_bad: Some("let xs = [1, 2, 3];
print(xs[3]);"),
        example_good: Some("let xs = [1, 2, 3];
if let Some(v) = xs.get(3) { print(v); }"),
        see_also: &["HHS_E_RUNTIME_TYPE_ERROR", "HHS_E_RUNTIME_PROPERTY_NOT_FOUND", "HHS_E_RUNTIME_INVALID_OPERATION"],
        since_version: "0.1.0",
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_INVALID_OPERATION: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(235),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_MODULE_ERROR: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(236),
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
        category: ErrorCategory::Runtime,
    };

pub const RUNTIME_OUT_OF_GAS: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(237),
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
        category: ErrorCategory::Runtime,
    };

pub use core::*;
