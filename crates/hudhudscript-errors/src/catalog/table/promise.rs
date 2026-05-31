use super::{ErrorCategory, ErrorCode, ErrorEntry};

pub const PROMISE_ALREADY_REJECTED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(193),
        long_code: "HHS_E_PROMISE_ALREADY_REJECTED",
        short_code: "E0193",
        title: "Promise has already been rejected",
        short_description: "Attempt to settle a promise that is already in the rejected state — a promise can only transition once.",
        long_description: "HudHudScript promises are one-shot: once a promise is rejected, any further `resolve` or `reject` call is an error. This guards against logic bugs where multiple producers race to settle the same promise and the later one silently overwrites the earlier value.

Fix this by introducing a single owner of each promise's settlement, or by using a pattern like `Promise.any` / `Promise.race` when you intentionally want first-writer-wins semantics.

This error typically surfaces inside Deferred-style code (manual `new_promise()` + `resolve`/`reject`). Promises produced implicitly by `async fn` cannot hit this case — the compiler guarantees a single settlement.",
        hints: &["Make sure exactly one code path settles each Deferred promise", "Use `if !promise.is_settled()` as a guard before re-settling if truly needed", "For first-writer-wins semantics, use `Promise.race` instead of manual retries", "Log the second rejection's stack to find the duplicate-settle site"],
        example_bad: Some("let p = Promise.new();
p.reject(\"timeout\");
p.reject(\"cancel\"); // error"),
        example_good: Some("let p = Promise.new();
if !p.is_settled() {
    p.reject(\"timeout\");
}"),
        see_also: &["HHS_E_PROMISE_ALREADY_RESOLVED", "HHS_E_PROMISE_REJECTED", "HHS_E_PROMISE_RECEIVER_DROPPED"],
        since_version: "0.4.0",
        category: ErrorCategory::Promise,
    };

pub const PROMISE_ALREADY_RESOLVED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(194),
        long_code: "HHS_E_PROMISE_ALREADY_RESOLVED",
        short_code: "E0194",
        title: "Promise has already been resolved",
        short_description: "Attempt to settle a promise that has already been resolved — a promise can only transition once.",
        long_description: "HudHudScript promises are one-shot. Once `resolve(value)` has been called, any subsequent `resolve` or `reject` is an error because it would either discard the original value or leave observers with inconsistent state.

The most common cause is a race between a success path and a cleanup/cancel path both trying to settle the same promise. Use an explicit `is_settled()` guard, or funnel all settlement through a single closure.

For promises returned by `async fn`, this error cannot occur — it only arises with explicit Deferred-style promise creation.",
        hints: &["Ensure exactly one code path calls `resolve` on each Deferred promise", "Guard with `if !promise.is_settled()` when multiple producers are possible", "Prefer `async fn` returns over manual promise creation when possible", "Use `Promise.race` for first-writer-wins rather than manual re-resolve"],
        example_bad: Some("let p = Promise.new();
p.resolve(1);
p.resolve(2); // error"),
        example_good: Some("let p = Promise.new();
if !p.is_settled() {
    p.resolve(1);
}"),
        see_also: &["HHS_E_PROMISE_ALREADY_REJECTED", "HHS_E_PROMISE_REJECTED", "HHS_E_PROMISE_RECEIVER_DROPPED"],
        since_version: "0.4.0",
        category: ErrorCategory::Promise,
    };

pub const PROMISE_RECEIVER_DROPPED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(195),
        long_code: "HHS_E_PROMISE_RECEIVER_DROPPED",
        short_code: "E0195",
        title: "Promise receiver was dropped before settlement",
        short_description: "The awaiter of a promise was dropped before the promise was resolved or rejected, leaving no one to receive the result.",
        long_description: "Every HudHudScript promise has a sender side (the producer) and a receiver side (the awaiter). This error is raised when the producer goes to settle a promise but the receiver has already been dropped — typically because the awaiting task was cancelled, timed out, or the enclosing scope exited.

In most programs this is harmless and indicates a clean cancellation: the result simply has nowhere to go. However, if you see this unexpectedly, it usually means a task was cancelled while its parent still believed the work was in flight.

To debug, look at task cancellation sites and confirm that you are not dropping a future whose result you needed.",
        hints: &["Usually indicates clean cancellation — often safe to ignore", "Check for `spawn` results that are dropped without being awaited", "If unexpected, trace back to the cancellation site of the awaiting task", "Consider using `detach()` if you intentionally do not want the result"],
        example_bad: Some("let p = async_task();
// forgot to await p — receiver drops
return;"),
        example_good: Some("let p = async_task();
let result = await p;
return result;"),
        see_also: &["HHS_E_PROMISE_ALREADY_RESOLVED", "HHS_E_ASYNC_RUNTIME_PROMISE_NOT_FOUND", "HHS_E_PROMISE_REJECTED"],
        since_version: "0.4.0",
        category: ErrorCategory::Promise,
    };

pub const PROMISE_REJECTED: ErrorEntry =
    ErrorEntry {
        code: ErrorCode(196),
        long_code: "HHS_E_PROMISE_REJECTED",
        short_code: "E0196",
        title: "Awaited promise was rejected",
        short_description: "A promise reached an `await` point in the rejected state and its error value was propagated to the caller.",
        long_description: "When `await` observes a rejected promise, the rejection value is surfaced to the awaiting code. If nothing catches it, the error bubbles up as this variant. The attached value is whatever the producer passed to `reject(...)` — often a string or an error object.

Handle rejected promises with `try`/`catch` around `await`, or by chaining `.catch(handler)` on the promise. Top-level rejections should be caught in your main task to prevent silent failures.

This is distinct from `RuntimePromiseRejected`: this variant originates in the promise layer, the other is its surfacing in the interpreter's error chain.",
        hints: &["Wrap `await` in `try { ... } catch (e) { ... }` to handle failures locally", "Chain `.catch(...)` to transform rejections into fallback values", "Ensure the top-level task handles rejections to avoid silent exits", "Log the rejection value to identify which producer failed"],
        example_bad: Some("let data = await fetch(\"https://bad.url\");
process(data);"),
        example_good: Some("try {
    let data = await fetch(\"https://bad.url\");
    process(data);
} catch (e) {
    log(\"fetch failed: \" + e);
}"),
        see_also: &["HHS_E_RUNTIME_PROMISE_REJECTED", "HHS_E_PROMISE_ALREADY_REJECTED", "HHS_E_RUNTIME_THROW"],
        since_version: "0.4.0",
        category: ErrorCategory::Promise,
    };

pub static ENTRIES: &[ErrorEntry] = &[
    PROMISE_ALREADY_REJECTED,
    PROMISE_ALREADY_RESOLVED,
    PROMISE_RECEIVER_DROPPED,
    PROMISE_REJECTED,
];
