use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const ASYNC_RUNTIME_PROMISE_NOT_FOUND: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(8),
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
        category: ExceptionCategory::Runtime,
    };

pub const ASYNC_RUNTIME_RUNTIME_ERROR: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(9),
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
        category: ExceptionCategory::Runtime,
    };
