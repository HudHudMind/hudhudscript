//! FFI: promise await / mint / resolve.
//!
//! `hud_vm_await_promise` and `hud_vm_create_promise` run on the VM owner
//! thread. `hud_vm_resolve_promise` settles a pre-minted promise from ANY
//! thread (it only fires a stored sender — never touches the VM) which is
//! what makes the Dart→script async data push race-free.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::time::Duration;

use hudhudscript_bytecode::{PromiseState16, Value16};

use crate::bridge_state::{settle_promise, with_bridge};
use crate::value_dto::{dto_to_value16, value16_to_dto, HudValue};
use crate::{check_owner, ensure_pool, set_error, HudVM};

fn cstr_arg<'a>(ptr: *const c_char, what: &str) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err(format!("{what} is null"));
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| format!("Invalid UTF-8 in {what}"))
}

/// Block until the promise `promise_id` settles. `timeout_ms == 0` waits
/// forever; on timeout the receiver is preserved so the same id can be
/// awaited again (chunked waits). Returns a boxed `HudValue` or null.
///
/// # Safety
/// `vm` from `hud_vm_new`; owner thread only.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_await_promise(
    vm: *mut HudVM,
    promise_id: *const c_char,
    timeout_ms: u32,
) -> *mut HudValue {
    if vm.is_null() {
        return std::ptr::null_mut();
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    ensure_pool(hud);
    let id = match cstr_arg(promise_id, "promise id") {
        Ok(s) => s.to_string(),
        Err(e) => {
            set_error(hud, e);
            return std::ptr::null_mut();
        }
    };
    let timeout = if timeout_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(timeout_ms as u64))
    };
    match hud.vm.await_promise_owned(&id, timeout) {
        Ok(settled) => Box::into_raw(Box::new(value16_to_dto(&settled, 0))),
        Err(e) => {
            set_error(hud, format!("Await error: {e}"));
            std::ptr::null_mut()
        }
    }
}

/// Mint a promise id the embedder can hand to script code (as a promise
/// DTO argument) and settle later from any thread via
/// `hud_vm_resolve_promise`. Returns a C string (free with
/// `hud_string_free`) or null on error.
///
/// # Safety
/// `vm` from `hud_vm_new`; owner thread only.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_create_promise(vm: *mut HudVM) -> *mut c_char {
    if vm.is_null() {
        return std::ptr::null_mut();
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    ensure_pool(hud);
    let bridge_id = hud.bridge_id;
    let (tx, rx) = std::sync::mpsc::channel::<Result<Value16, String>>();
    let id = hud.vm.register_promise_owned(rx);
    let ok = with_bridge(bridge_id, |state| {
        state.promise_txs.insert(id.clone(), tx);
    })
    .is_some();
    if !ok {
        set_error(hud, "bridge state missing".to_string());
        return std::ptr::null_mut();
    }
    match CString::new(id) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Settle a promise created by `hud_vm_create_promise` or an async host
/// callback's promise. Thread-safe: safe to call from the Dart/UI thread
/// while the VM is executing. Exactly one of `result` / `error` should be
/// non-null (`result` wins when both are set). Returns 0 on success, -1
/// when the id is unknown or already settled.
///
/// # Safety
/// `result` (if set) is borrowed for the call duration; `error` is a
/// valid C string.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_resolve_promise(
    bridge_id: i64,
    promise_id: *const c_char,
    result: *const HudValue,
    error: *const c_char,
) -> i32 {
    let id = match cstr_arg(promise_id, "promise id") {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };
    let outcome = if !result.is_null() {
        match dto_to_value16(result, 0) {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    } else if !error.is_null() {
        Err(CStr::from_ptr(error).to_string_lossy().into_owned())
    } else {
        Ok(Value16::promise(PromiseState16::Resolved(Box::new(
            Value16::null(),
        ))))
    };
    if settle_promise(bridge_id, &id, outcome) {
        0
    } else {
        -1
    }
}

/// Non-consuming check: 1 = still pending, 0 = settled or unknown-but-was
/// seen (await will resolve instantly or error), -1 = id unknown to the
/// bridge. Promise ids minted by the VM itself (async functions) are
/// reported as pending (1) — their liveness is tracked inside the VM, not
/// the bridge.
///
/// # Safety
/// `promise_id` a valid C string.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_promise_pending(
    bridge_id: i64,
    promise_id: *const c_char,
) -> i32 {
    let id = match cstr_arg(promise_id, "promise id") {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match with_bridge(bridge_id, |state| {
        state.promise_txs.contains_key(id) || state.free_ids.iter().any(|f| f.as_str() == id)
    }) {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    }
}
