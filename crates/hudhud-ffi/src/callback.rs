//! FFI: script-callable host functions.
//!
//! Two registration flavors:
//!
//! * `hud_vm_register_host_fn` — a plain C trampoline (`HudHostFn`). Called
//!   synchronously on the VM thread; must return a boxed `HudValue`. This
//!   is the C/C++ game-engine path.
//! * `hud_vm_register_dart_fn` — the Dart path. The trampoline cannot
//!   exist in Dart, so instead the embedder installs a
//!   `NativeCallable.listener` address via `hud_dart_set_callback`; when
//!   the script calls the function the bridge mints a pre-registered
//!   promise, queues the request, and fires the callable with a request
//!   id (non-blocking post to the Dart isolate's event loop). The Dart
//!   side fetches args with `hud_callback_take`, runs its handler, and
//!   settles the promise with `hud_callback_complete` — from any thread,
//!   without touching the VM handle.
//!
//! Re-entrancy rule: a host callback must not call back into the same VM
//! (the interpreter loop already holds `&mut VM`). The Dart path is
//! naturally safe: settlement goes through the bridge tables, not the VM.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

use hudhudscript_bytecode::{ObjMap, PromiseState16, Value16};

use crate::bridge_state::{next_request_id, take_free_promise, with_bridge, PendingHostCall};
use crate::value_dto::{dto_to_value16, free_dto_contents, value16_to_dto, HudValue};
use crate::{check_owner, ensure_pool, HudVM};

/// Synchronous C trampoline: receives borrowed args, returns an owned
/// `HudValue` (freed by the bridge) or null on failure.
pub type HudHostFn =
    unsafe extern "C" fn(user_data: *mut c_void, args: *const HudValue, len: i32) -> *mut HudValue;

/// `*mut c_void` carries no thread-safety; the embedder's safety contract
/// (pointee stays valid and VM-thread-usable for the VM's lifetime) is
/// expressed by laundering the address through `usize`. A newtype with
/// `unsafe impl Send` would NOT suffice — Rust 2021 precise captures grab
/// the raw-pointer FIELD when the closure only touches `wrapper.0`.
fn user_data_bits(user_data: *mut c_void) -> usize {
    user_data as usize
}

/// Bare `module.method(...)` calls compile to a global load of `module`
/// followed by MethodCall, which dispatches through the receiver's
/// `__module` marker (see `vm/globals.rs` — the stdlib pre-installs the
/// same marker for `http`, `os`, ...). Registered host modules therefore
/// need a marker global too; stdlib names are left untouched.
fn ensure_module_global(hud: &mut HudVM, module: &str) {
    if hud.vm.get_variable(module).is_some() {
        return;
    }
    let mut obj = ObjMap::default();
    obj.insert("__module", Value16::string(module));
    obj.insert("__loaded", Value16::bool_(true));
    hud.vm.define_global(module.to_string(), Value16::object(obj));
}

/// Engine quirk guard: pre-registering the module/method names keeps the
/// process-first compile of `await <module>.<method>(...)` on the correct
/// path (see tests/engine_quirk_test.rs for the full history — the root
/// cause was a RegAlloc/temp-register zone overlap, fixed in the
/// compiler; this interning remains as cheap, harmless warm-up).
fn intern_module_names(module: &str, name: &str) {
    hudhudscript_bytecode::interner::intern(module);
    hudhudscript_bytecode::interner::intern(name);
}

#[repr(C)]
pub struct HudCallbackInfo {
    /// "module.method" key the Dart side resolves to a handler.
    pub key: *mut c_char,
    /// Argument array of `len` elements (owned by this struct).
    pub args: *mut HudValue,
    pub len: u32,
}

/// Register a C-trampoline host function callable from script as
/// `module.method(...)`. Returns 0 on success.
///
/// # Safety
/// `vm` from `hud_vm_new`; `module`/`name` valid C strings; `trampoline`
/// must stay valid for the VM's lifetime.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_register_host_fn(
    vm: *mut HudVM,
    module: *const c_char,
    name: *const c_char,
    trampoline: HudHostFn,
    user_data: *mut c_void,
) -> i32 {
    if vm.is_null() || module.is_null() || name.is_null() {
        return -1;
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    let module = match CStr::from_ptr(module).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    intern_module_names(module, name);

    let user_data = user_data_bits(user_data);
    let handler: hudhudscript_vm::vm::registry::BuiltinFn =
        Box::new(move |args: &[Value16]| {
            let mut dtos: Vec<HudValue> = args.iter().map(|a| value16_to_dto(a, 0)).collect();
            let (ptr, len) = if dtos.is_empty() {
                (std::ptr::null(), 0i32)
            } else {
                (dtos.as_ptr(), dtos.len() as i32)
            };
            let out = trampoline(user_data as *mut c_void, ptr, len);
            for d in dtos.iter_mut() {
                free_dto_contents(d);
            }
            if out.is_null() {
                return Err(hudhudscript_errors::runtime_error("host function failed"));
            }
            let value = dto_to_value16(out, 0);
            crate::value_dto::hud_value_free(out);
            value.map_err(hudhudscript_errors::runtime_error)
        });
    hud.vm.register_method(module, name, handler);
    ensure_module_global(hud, module);
    0
}

/// Register a Dart-backed host function. The bridge calls the address
/// installed by `hud_dart_set_callback` with a request id; see module docs.
/// Returns 0 on success.
///
/// # Safety
/// `vm` from `hud_vm_new`; `module`/`name` valid C strings.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_register_dart_fn(
    vm: *mut HudVM,
    module: *const c_char,
    name: *const c_char,
) -> i32 {
    if vm.is_null() || module.is_null() || name.is_null() {
        return -1;
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    ensure_pool(hud);
    let bridge_id = hud.bridge_id;
    let module = match CStr::from_ptr(module).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };
    intern_module_names(&module, &name);
    let key = format!("{module}.{name}");

    let handler: hudhudscript_vm::vm::registry::BuiltinFn =
        Box::new(move |args: &[Value16]| {
            let request_id = next_request_id();
            let setup = with_bridge(bridge_id, |state| {
                if state.dart_cb.is_none() {
                    return Err("no dart callback installed".to_string());
                }
                let Some(promise_id) = take_free_promise(state) else {
                    return Err("dart callback promise pool exhausted".to_string());
                };
                state.pending_calls.insert(
                    request_id,
                    PendingHostCall {
                        key: key.clone(),
                        args: Some(args.to_vec()),
                        promise_id,
                    },
                );
                Ok(())
            });
            let promise_id = match setup {
                Some(Ok(())) => with_bridge(bridge_id, |state| {
                    state
                        .pending_calls
                        .get(&request_id)
                        .map(|p| p.promise_id.clone())
                }),
                Some(Err(e)) => return Err(hudhudscript_errors::runtime_error(e)),
                None => return Err(hudhudscript_errors::runtime_error("bridge state missing")),
            };
            let promise_id = match promise_id {
                Some(Some(pid)) => pid,
                _ => return Err(hudhudscript_errors::runtime_error("bridge state missing")),
            };
            let cb_addr = with_bridge(bridge_id, |state| state.dart_cb).flatten();
        let Some(cb_addr) = cb_addr else {
            return Err(hudhudscript_errors::runtime_error("no dart callback installed"));
        };
        let notify: extern "C" fn(i64) = std::mem::transmute(cb_addr);
        notify(request_id);
        Ok(Value16::promise(PromiseState16::AsyncPending(promise_id)))
    });
    hud.vm.register_method(&module, &name, handler);
    ensure_module_global(hud, &module);
    0
}

/// Install the Dart `NativeCallable.listener` address for a bridge.
/// Signature on the Dart side: `void Function(int64 request_id)`.
/// Returns 0 on success, -1 for an unknown bridge.
///
/// # Safety
/// `cb` must be a valid `extern "C" fn(i64)` address kept alive by the
/// embedder (do not let the `NativeCallable` be GC'd).
#[no_mangle]
pub unsafe extern "C" fn hud_dart_set_callback(bridge_id: i64, cb: usize) -> i32 {
    match with_bridge(bridge_id, |state| state.dart_cb = Some(cb)) {
        Some(_) => 0,
        None => -1,
    }
}

/// Fetch the key + arguments of a callback request (any thread; called by
/// the Dart isolate that owns the handler). Returns null when the request
/// is unknown or its args were already taken. Free with
/// `hud_callback_info_free`.
///
/// # Safety
/// Returned pointer freed exactly once via `hud_callback_info_free`.
#[no_mangle]
pub unsafe extern "C" fn hud_callback_take(
    bridge_id: i64,
    request_id: i64,
) -> *mut HudCallbackInfo {
    let taken = with_bridge(bridge_id, |state| {
        if let Some(call) = state.pending_calls.get_mut(&request_id) {
            let key = CString::new(call.key.clone()).ok()?;
            let args = call.args.take()?;
            let dtos: Vec<HudValue> = args.iter().map(|a| value16_to_dto(a, 0)).collect();
            let len = dtos.len() as u32;
            let ptr = if dtos.is_empty() {
                std::ptr::null_mut()
            } else {
                Box::into_raw(dtos.into_boxed_slice()) as *mut HudValue
            };
            Some(HudCallbackInfo {
                key: key.into_raw(),
                args: ptr,
                len,
            })
        } else {
            None
        }
    });
    match taken {
        Some(Some(info)) => Box::into_raw(Box::new(info)),
        _ => std::ptr::null_mut(),
    }
}

/// Free a `HudCallbackInfo` from `hud_callback_take`.
///
/// # Safety
/// Pointer freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn hud_callback_info_free(info: *mut HudCallbackInfo) {
    if info.is_null() {
        return;
    }
    let boxed = Box::from_raw(info);
    if !boxed.key.is_null() {
        drop(CString::from_raw(boxed.key));
    }
    if !boxed.args.is_null() && boxed.len > 0 {
        let slice = std::slice::from_raw_parts_mut(boxed.args, boxed.len as usize);
        for item in slice {
            free_dto_contents(item);
        }
        drop(Vec::from_raw_parts(
            boxed.args,
            boxed.len as usize,
            boxed.len as usize,
        ));
    }
}

/// Complete a callback request: settles its promise with `result` (or
/// rejects with `error`). Thread-safe. Returns 0 on success, -1 when the
/// request is unknown or already completed.
///
/// # Safety
/// `result` (if set) borrowed for the call duration.
#[no_mangle]
pub unsafe extern "C" fn hud_callback_complete(
    bridge_id: i64,
    request_id: i64,
    result: *const HudValue,
    error: *const c_char,
) -> i32 {
    let outcome: Result<Value16, String> = if !result.is_null() {
        dto_to_value16(result, 0)
    } else if !error.is_null() {
        Err(CStr::from_ptr(error).to_string_lossy().into_owned())
    } else {
        Ok(Value16::null())
    };
    let settled = with_bridge(bridge_id, |state| {
        if let Some(call) = state.pending_calls.remove(&request_id) {
            let promise_id = call.promise_id;
            if let Some(tx) = state.promise_txs.remove(&promise_id) {
                let _ = tx.send(outcome);
                return true;
            }
        }
        false
    });
    if settled == Some(true) {
        0
    } else {
        -1
    }
}
