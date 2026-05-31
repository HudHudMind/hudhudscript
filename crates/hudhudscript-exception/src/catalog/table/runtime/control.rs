use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

pub const RUNTIME_RETURN: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(241),
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
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_THROW: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(248),
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
        category: ExceptionCategory::Runtime,
    };

pub const RUNTIME_YIELD: ExceptionEntry =
    ExceptionEntry {
        code: ExceptionCode(254),
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
        category: ExceptionCategory::Runtime,
    };
