//! C FFI bindings for HudHudScript.
//!
//! Allows embedding HudHudScript in C/C++ game engines (Unity, Godot, Unreal).
//!
//! Usage from C:
//!   HudVM* vm = hud_vm_new();
//!   hud_vm_execute_source(vm, "let x = 42");
//!   double x = hud_vm_get_number(vm, "x");
//!   hud_vm_free(vm);
//!
//! C header (hudhud.h):
//!
//! ```c
//! // hudhud.h — Generated C header
//! typedef struct HudVM HudVM;
//! HudVM* hud_vm_new();
//! void hud_vm_free(HudVM* vm);
//! int hud_vm_execute_source(HudVM* vm, const char* source);
//! double hud_vm_get_number(HudVM* vm, const char* name);
//! char* hud_vm_get_string(HudVM* vm, const char* name);
//! int hud_vm_get_bool(HudVM* vm, const char* name);
//! void hud_string_free(char* s);
//! const char* hud_vm_last_error(HudVM* vm);
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

/// Opaque VM handle for C consumers.
pub struct HudVM {
    vm: VM,
    last_error: Option<CString>,
}

/// Create a new VM instance.
#[no_mangle]
pub extern "C" fn hud_vm_new() -> *mut HudVM {
    let vm = HudVM {
        vm: VM::new(),
        last_error: None,
    };
    Box::into_raw(Box::new(vm))
}

/// Free a VM instance.
///
/// # Safety
/// `vm` must be a pointer returned by `hud_vm_new` and must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_free(vm: *mut HudVM) {
    if !vm.is_null() {
        drop(Box::from_raw(vm));
    }
}

/// Execute HudHudScript source code. Returns 0 on success, -1 on error.
///
/// # Safety
/// `vm` must be a valid pointer from `hud_vm_new`. `source` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_execute_source(vm: *mut HudVM, source: *const c_char) -> i32 {
    if vm.is_null() || source.is_null() {
        return -1;
    }
    let vm = &mut (*vm);
    vm.last_error = None;

    let source = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(e) => {
            vm.last_error = CString::new(format!("Invalid UTF-8: {e}")).ok();
            return -1;
        }
    };

    let ast = match parse(source) {
        Ok(ast) => ast,
        Err(e) => {
            vm.last_error = CString::new(format!("Parse error: {e}")).ok();
            return -1;
        }
    };

    let mut compiler = Compiler::new();
    let bytecode = match compiler.compile(&ast) {
        Ok(bc) => bc,
        Err(e) => {
            vm.last_error = CString::new(format!("Compile error: {e}")).ok();
            return -1;
        }
    };

    match vm.vm.execute(&bytecode) {
        Ok(()) => 0,
        Err(e) => {
            vm.last_error = CString::new(format!("Runtime error: {e}")).ok();
            -1
        }
    }
}

/// Get a number variable from the VM. Returns 0.0 if not found.
///
/// # Safety
/// `vm` must be a valid pointer from `hud_vm_new`. `name` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_get_number(vm: *mut HudVM, name: *const c_char) -> f64 {
    if vm.is_null() || name.is_null() {
        return 0.0;
    }
    let vm = &(*vm);
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return 0.0,
    };
    match vm.vm.get_variable(name) {
        Some(v) => v.as_number().unwrap_or(0.0),
        None => 0.0,
    }
}

/// Get a string variable. Caller must free the returned pointer with `hud_string_free()`.
/// Returns null if the variable is not found or is not a string.
///
/// # Safety
/// `vm` must be a valid pointer from `hud_vm_new`. `name` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_get_string(vm: *mut HudVM, name: *const c_char) -> *mut c_char {
    if vm.is_null() || name.is_null() {
        return std::ptr::null_mut();
    }
    let vm = &(*vm);
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match vm.vm.get_variable(name) {
        Some(v) => {
            if let Some(s) = v.as_str() {
                CString::new(s.to_string())
                    .map(|c| c.into_raw())
                    .unwrap_or(std::ptr::null_mut())
            } else {
                std::ptr::null_mut()
            }
        }
        None => std::ptr::null_mut(),
    }
}

/// Get a boolean variable. Returns 1 for true, 0 for false or not found.
///
/// # Safety
/// `vm` must be a valid pointer from `hud_vm_new`. `name` must be a valid null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_get_bool(vm: *mut HudVM, name: *const c_char) -> i32 {
    if vm.is_null() || name.is_null() {
        return 0;
    }
    let vm = &(*vm);
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    match vm.vm.get_variable(name) {
        Some(v) => v.as_bool().map(|b| i32::from(b)).unwrap_or(0),
        None => 0,
    }
}

/// Free a string returned by `hud_vm_get_string()`.
///
/// # Safety
/// `s` must be a pointer returned by `hud_vm_get_string` and must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn hud_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Get last error message. Returns null if no error.
/// The returned pointer is valid until the next call to `hud_vm_execute_source()` or `hud_vm_free()`.
///
/// # Safety
/// `vm` must be a valid pointer from `hud_vm_new`.
#[no_mangle]
pub unsafe extern "C" fn hud_vm_last_error(vm: *mut HudVM) -> *const c_char {
    if vm.is_null() {
        return std::ptr::null();
    }
    let vm = &(*vm);
    match &vm.last_error {
        Some(err) => err.as_ptr(),
        None => std::ptr::null(),
    }
}
