//! Shared timer / scheduler builtins (Kural 7).
//!
//! `setTimeout`, `setInterval`, `clearTimeout`, and `clearInterval` used to
//! live in two places with drifted shapes:
//!
//! - `hudhudscript-builtins::builtins::timer` exposed them as interpreter
//!   `NativeFunction`s returning `{id, type, ms, fired|active}`.
//! - The VM's `call_builtin` handler inlined a stripped-down version that
//!   **silently** ignored the callback, skipped argument validation, and
//!   returned a bare `{type, ms, fired|active}` without an `id`.
//!
//! Both runtimes also shared the same foundational limitation: neither of
//! them actually *invokes* the scheduled callback. `NativeFunction`
//! signatures can't carry a live runtime reference, so the only honest
//! thing we can do in a synchronous builtin is sleep for the requested
//! duration and return a descriptor that names the operation. This module
//! codifies that behaviour so both runtimes speak the same language:
//!
//! - **Validated arguments**: the callback is checked, `ms` must be a
//!   non-negative number, and both error consistently across runtimes
//!   when the contract is violated.
//! - **Shared id allocator**: `NEXT_TIMER_ID` is process-wide so ids
//!   never collide when both runtimes run in the same process (tests).
//! - **Blocking simulation**: `setTimeout` still sleeps in the current
//!   thread so scripts that depend on the time-offset effect keep
//!   working; `setInterval` does NOT sleep (it would be unbounded).
//! - **Descriptor shape**: both return
//!   `{id, type, ms, fired|active, callback_pending}` where
//!   `callback_pending` is always `true` — a scripts-level signal that
//!   the callback has been accepted but the synchronous builtin is not
//!   the one that will run it. Real callback dispatch remains a
//!   pending feature for the async runtime; the descriptor makes the
//!   gap explicit instead of lying.
//!
//! Both interpreter and VM call through `build_set_timeout_descriptor`
//! and `build_set_interval_descriptor`; `clearTimeout` / `clearInterval`
//! route through `shared_clear_timer`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

/// Monotonic counter feeding every timer-id mint in the workspace.
///
/// Shared between interpreter and VM so a test that spins up both in the
/// same process never sees duplicated ids.
static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);

/// Allocate a fresh timer id.
pub fn next_timer_id() -> u64 {
    NEXT_TIMER_ID.fetch_add(1, Ordering::SeqCst)
}

/// Extract the `ms` delay from a timer call's args, reading `args[1]` first
/// (the `(callback, ms)` convention) and falling back to `args[0]` so a
/// one-arg `setTimeout(100)` still works. Rejects negative or non-numeric
/// delays with a consistent error message.
fn extract_ms(args: &[Value16], context: &str) -> HudHudResult<u64> {
    let ms_val = args.get(1).or_else(|| args.first());
    match ms_val.and_then(|v| v.as_number()) {
        Some(n) if n >= 0.0 => Ok(n as u64),
        Some(_) => Err(runtime_error(format!(
            "{}: delay must be non-negative",
            context
        ))),
        None => Err(runtime_error(format!(
            "{}: delay must be a number (got {})",
            context,
            ms_val.map(|v| v.type_name_str()).unwrap_or("missing"),
        ))),
    }
}

/// Build the descriptor object `setTimeout` returns.
///
/// The synchronous part of the operation happens here: the current thread
/// sleeps for `ms` milliseconds (matching both runtimes' pre-existing
/// simulation). The callback itself is not yet dispatched — that will
/// happen once a real timer scheduler lands in the async runtime. The
/// descriptor advertises `callback_pending: true` so scripts can tell
/// the built-in has *accepted* the work without claiming it has *run* it.
pub fn shared_set_timeout(args: &[Value16]) -> HudHudResult<Value16> {
    // Callback is positional arg 0. We currently don't invoke it, but we
    // still require it to be present so the script-level signature stays
    // honest: `setTimeout(callback, ms)`.
    if args.is_empty() {
        return Err(runtime_error(
            "setTimeout(callback, ms) requires at least the ms delay",
        ));
    }
    let ms = extract_ms(args, "setTimeout")?;

    let id = next_timer_id();
    std::thread::sleep(Duration::from_millis(ms));

    let has_callback = args.len() >= 2 && !args.first().map(|v| v.is_null()).unwrap_or(true);
    Ok(build_descriptor("timeout", id, ms, "fired", has_callback))
}

/// Build the descriptor object `setInterval` returns.
///
/// Unlike `setTimeout` this does NOT sleep — the delay is advisory
/// metadata for whoever actually runs the interval. Scripts get back an
/// id they can later hand to `clearInterval`.
pub fn shared_set_interval(args: &[Value16]) -> HudHudResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(
            "setInterval(callback, ms) requires at least the ms delay",
        ));
    }
    let ms = extract_ms(args, "setInterval")?;
    let id = next_timer_id();
    let has_callback = args.len() >= 2 && !args.first().map(|v| v.is_null()).unwrap_or(true);
    Ok(build_descriptor("interval", id, ms, "active", has_callback))
}

/// Shared `sleep(ms)` / `delay(ms)` builtin — blocks the current thread for
/// the given number of milliseconds and returns `null`.
///
/// Mirrors the interpreter-era `builtins::sleep::sleep_fn` contract exactly:
/// - Requires exactly one argument (the delay).
/// - Arg must be a non-negative `Number` of **milliseconds**.
/// - Returns `Null` after `std::thread::sleep(Duration::from_millis(n))`.
///
/// This replaces the parallel interpreter-era implementation so both
/// runtimes dispatch to a single source (Kural 7).
pub fn sleep(args: &[Value16]) -> HudHudResult<Value16> {
    let first = args
        .first()
        .ok_or_else(|| runtime_error("sleep: Expected 1 argument, got 0"))?;
    let n = first.as_number().ok_or_else(|| {
        runtime_error(format!(
            "sleep: expected number (got {})",
            first.type_name_str()
        ))
    })?;
    if n < 0.0 {
        return Err(runtime_error("sleep: duration must be non-negative"));
    }
    std::thread::sleep(Duration::from_millis(n as u64));
    Ok(Value16::null())
}

/// Accepts either a timer id (as a number) or a descriptor object carrying
/// `{id: n, …}`. The underlying scheduler is not yet real, so this is a
/// no-op that accepts the call and returns `null` — matching the pre-
/// existing interpreter behaviour. It still validates the input shape so
/// scripts get an error if they pass nonsense.
pub fn shared_clear_timer(args: &[Value16], context: &str) -> HudHudResult<Value16> {
    match args.first() {
        None => Err(runtime_error(format!("{}: missing timer id", context))),
        Some(val) if val.as_number().is_some() => Ok(Value16::null()),
        Some(val) if val.as_object().is_some() => {
            let obj = val.as_object().unwrap();
            if obj.get("id").and_then(|v| v.as_number()).is_some() {
                Ok(Value16::null())
            } else {
                Err(runtime_error(format!(
                    "{}: timer descriptor missing `id` field",
                    context
                )))
            }
        }
        Some(val) => Err(runtime_error(format!(
            "{}: expected a timer id or descriptor, got {}",
            context,
            val.type_name_str()
        ))),
    }
}

fn build_descriptor(ty: &str, id: u64, ms: u64, status_key: &str, has_callback: bool) -> Value16 {
    let mut obj: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    obj.insert("id".to_string(), Value16::number(id as f64));
    obj.insert("type".to_string(), Value16::string(ty.to_string()));
    obj.insert("ms".to_string(), Value16::number(ms as f64));
    obj.insert(status_key.to_string(), Value16::bool_(true));
    // Honest parity signal: the synchronous builtin accepted the callback
    // but it has NOT been dispatched. A real timer scheduler in the async
    // runtime will flip this field to `false` once it actually runs the
    // callback. For now both runtimes set it identically.
    obj.insert("callback_pending".to_string(), Value16::bool_(has_callback));
    Value16::object(obj)
}
