use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RUNTIME_PROMISE_REJECTED: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(238),
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
        category: ExceptionCategory::Runtime,
    };
