//! FFI: bytecode execution, script function invocation, globals.
//!
//! All functions here run on the VM owner thread (the Dart worker
//! isolate's thread); `check_owner` enforces this in debug builds.

use std::ffi::CStr;
use std::os::raw::c_char;

use hudhudscript_bytecode::Bytecode;

use crate::value_dto::{dto_to_value16, value16_to_dto, HudValue};
use crate::{check_owner, compile_source, ensure_pool, set_error, HudVM};

/// Opaque handle to a compiled bytecode unit.
pub struct HudBytecode {
    pub(crate) inner: Bytecode,
}

/// Compile source into a storable bytecode unit without running it.
/// Free with `hud_bytecode_free`. Returns null on compile error (see
/// `hud_vm_last_error`).
///
/// # Safety
/// `vm` and `source` must be valid; `vm` from `hud_vm_new`.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_compile(vm: *mut HudVM, source: *const c_char) -> *mut HudBytecode {
    if vm.is_null() || source.is_null() {
        return std::ptr::null_mut();
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    let source = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(hud, format!("Invalid UTF-8: {e}"));
            return std::ptr::null_mut();
        }
    };
    match compile_source(source) {
        Ok(bc) => Box::into_raw(Box::new(HudBytecode { inner: bc })),
        Err(e) => {
            set_error(hud, e);
            std::ptr::null_mut()
        }
    }
}

/// Free a compiled bytecode unit.
///
/// # Safety
/// `bc` must come from `hud_vm_compile` and not be freed twice.
#[no_mangle]
pub unsafe extern "C" fn hud_bytecode_free(bc: *mut HudBytecode) {
    if !bc.is_null() {
        drop(Box::from_raw(bc));
    }
}

/// Execute a previously compiled unit (`.hudb` bytes loaded via
/// `hud_vm_execute_bytecode` or a `hud_vm_compile` handle). Returns 0 on
/// success, -1 on error. The unit is retained for later `hud_vm_call`.
///
/// # Safety
/// `vm` from `hud_vm_new`, `bc` from `hud_vm_compile`.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_execute_compiled(vm: *mut HudVM, bc: *mut HudBytecode) -> i32 {
    if vm.is_null() || bc.is_null() {
        return -1;
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    ensure_pool(hud);
    let bytecode = (*bc).inner.clone();
    let code = match hud.vm.execute(&bytecode) {
        Ok(()) => 0,
        Err(e) => {
            set_error(hud, format!("Runtime error: {e}"));
            -1
        }
    };
    if code == 0 {
        hud.last_bytecode = Some(bytecode);
    }
    code
}

/// Load and execute `.hudb` bytecode bytes (`Bytecode::to_bytes` output).
/// Returns 0 on success, -1 on error (version mismatch included).
///
/// # Safety
/// `ptr` must be readable for `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_execute_bytecode(
    vm: *mut HudVM,
    ptr: *const u8,
    len: usize,
) -> i32 {
    if vm.is_null() || ptr.is_null() {
        return -1;
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    let bytes = std::slice::from_raw_parts(ptr, len);
    let bytecode = match Bytecode::from_bytes(bytes) {
        Ok(bc) => bc,
        Err(e) => {
            set_error(hud, format!("Bytecode load error: {e}"));
            return -1;
        }
    };
    ensure_pool(hud);
    let code = match hud.vm.execute(&bytecode) {
        Ok(()) => 0,
        Err(e) => {
            set_error(hud, format!("Runtime error: {e}"));
            -1
        }
    };
    if code == 0 {
        hud.last_bytecode = Some(bytecode);
    }
    code
}

fn args_from_dto(args: *const HudValue, len: i32) -> Result<Vec<hudhudscript_bytecode::Value16>, String> {
    if args.is_null() || len <= 0 {
        return Ok(Vec::new());
    }
    let slice = unsafe { std::slice::from_raw_parts(args, len as usize) };
    let mut values = Vec::with_capacity(slice.len());
    for item in slice {
        values.push(dto_to_value16(item, 0)?);
    }
    Ok(values)
}

fn call_function(
    hud: &mut HudVM,
    name: &str,
    args: Vec<hudhudscript_bytecode::Value16>,
) -> Result<hudhudscript_bytecode::Value16, String> {
    let bytecode = hud
        .last_bytecode
        .as_ref()
        .ok_or_else(|| "no bytecode executed yet".to_string())?
        .clone();
    hud.vm
        .call_public(name, &args, &bytecode)
        .map_err(|e| format!("Call error: {e}"))
}

/// Call a top-level script function by name. Returns a boxed `HudValue`
/// (free with `hud_value_free`), or null on error. Async functions return
/// a promise DTO — follow up with `hud_vm_await_promise`.
///
/// # Safety
/// `vm` from `hud_vm_new`; `args` readable for `len` elements.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_call(
    vm: *mut HudVM,
    name: *const c_char,
    args: *const HudValue,
    len: i32,
) -> *mut HudValue {
    if vm.is_null() || name.is_null() {
        return std::ptr::null_mut();
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    ensure_pool(hud);
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_error(hud, "Invalid UTF-8 in function name".to_string());
            return std::ptr::null_mut();
        }
    };
    let values = match args_from_dto(args, len) {
        Ok(v) => v,
        Err(e) => {
            set_error(hud, e);
            return std::ptr::null_mut();
        }
    };
    match call_function(hud, name, values) {
        Ok(result) => Box::into_raw(Box::new(value16_to_dto(&result, 0))),
        Err(e) => {
            set_error(hud, e);
            std::ptr::null_mut()
        }
    }
}

/// Call a function and, when it yields a promise, block until it settles
/// (worker thread only). `timeout_ms == 0` means wait forever; on timeout
/// the underlying receiver is preserved for a later retry. Returns a boxed
/// `HudValue` or null on error.
///
/// # Safety
/// Same contract as `hud_vm_call`.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_call_await(
    vm: *mut HudVM,
    name: *const c_char,
    args: *const HudValue,
    len: i32,
    timeout_ms: u32,
) -> *mut HudValue {
    use hudhudscript_bytecode::PromiseState16;
    use std::time::Duration;

    if vm.is_null() || name.is_null() {
        return std::ptr::null_mut();
    }
    check_owner(&*vm);
    let hud = &mut *vm;
    hud.last_error = None;
    ensure_pool(hud);
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_error(hud, "Invalid UTF-8 in function name".to_string());
            return std::ptr::null_mut();
        }
    };
    let values = match args_from_dto(args, len) {
        Ok(v) => v,
        Err(e) => {
            set_error(hud, e);
            return std::ptr::null_mut();
        }
    };
    let result = match call_function(hud, name, values) {
        Ok(r) => r,
        Err(e) => {
            set_error(hud, e);
            return std::ptr::null_mut();
        }
    };
    let id = match result.as_promise_state() {
        Some(PromiseState16::AsyncPending(id)) => id.clone(),
        // Already-resolved promises and plain values pass through as-is.
        _ => return Box::into_raw(Box::new(value16_to_dto(&result, 0))),
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

/// Read a global as a `HudValue`. Null return means "not found" (no error
/// is recorded — use `hud_vm_last_error` to distinguish when needed).
///
/// # Safety
/// `vm` from `hud_vm_new`, `name` a valid C string.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_get_global(vm: *mut HudVM, name: *const c_char) -> *mut HudValue {
    if vm.is_null() || name.is_null() {
        return std::ptr::null_mut();
    }
    let hud = &*vm;
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match hud.vm.get_variable_owned(name) {
        Some(v) => Box::into_raw(Box::new(value16_to_dto(&v, 0))),
        None => std::ptr::null_mut(),
    }
}

/// Define (or overwrite) a global before a run or between calls.
/// Returns 0 on success, -1 on bad input.
///
/// # Safety
/// `vm` from `hud_vm_new`; `value` borrowed for the call duration.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_set_global(
    vm: *mut HudVM,
    name: *const c_char,
    value: *const HudValue,
) -> i32 {
    if vm.is_null() || name.is_null() || value.is_null() {
        return -1;
    }
    let hud = &mut *vm;
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_error(hud, "Invalid UTF-8 in global name".to_string());
            return -1;
        }
    };
    match dto_to_value16(value, 0) {
        Ok(v) => {
            hud.vm.define_global(name.to_string(), v);
            0
        }
        Err(e) => {
            set_error(hud, e);
            -1
        }
    }
}

/// Bridge id for cross-thread calls (`hud_callback_complete`,
/// `hud_vm_resolve_promise`) that must not touch the VM handle.
///
/// # Safety
/// `vm` from `hud_vm_new`.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_bridge_id(vm: *mut HudVM) -> i64 {
    if vm.is_null() {
        return 0;
    }
    (*vm).bridge_id
}
