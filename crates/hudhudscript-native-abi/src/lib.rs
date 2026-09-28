//! Hudhud native ABI — the C layer every native backend links against
//! (JIT_AOT_ARCHITECTURE.md §10-11).

/// Runtime ABI sürüm damgası (0.9.12 = 912). AOT linker bu sembolü arar:
/// bayat .a (eski BigInt repr/eksik helperlar) net hata verir — sessiz
/// yavaş AOT binary'ler (VM'den bile yavaş fib/power/fact) bu yüzden oluyordu.
#[no_mangle]
pub extern "C" fn hudhud_runtime_version() -> u32 {
    916
}

pub mod array;
pub mod assert;
pub mod dyn_call;
pub mod input;
pub mod string_arena;
pub mod string_core;
pub use assert::{hudhud_assert_approx, hudhud_assert_eq, hudhud_assert_false, hudhud_assert_true};
pub use dyn_call::{hudhud_dyn_call_method, hudhud_dyn_register};
pub use input::{hudhud_confirm, hudhud_input};
pub mod bigint;
pub mod bigint_arith;
pub mod bigint_num;
pub mod date;
pub mod exception;
pub mod jit_exit;
pub mod object;
pub mod string_ops;
pub mod type_ops;
pub mod value;

pub use array::*;
pub use bigint::*;
pub use bigint_num::*;
pub use date::*;
pub use exception::*;
pub use object::*;
pub use string_core::*;
pub use string_ops::*;
pub use type_ops::*;

pub use jit_exit::{
    JitExit, JIT_EXIT_DIV_ZERO, JIT_EXIT_OVERFLOW, JIT_EXIT_RETURNED, JIT_EXIT_UNCAUGHT_EXCEPTION,
};
pub use value::{HudTag, HudValue, HUDHUD_RUNTIME_ABI_VERSION};

use std::ffi::{c_char, CStr, CString};

/// Print status codes (uniform, extendable).
pub const HUDHUD_PRINT_OK: i32 = 0;
pub const HUDHUD_PRINT_UNSUPPORTED_TAG: i32 = 1;

/// Display a value as a NUL-terminated C string (caller frees with
/// `hudhud_string_free`). Returns NULL for tags whose exact engine
/// formatting is not wired yet (Float/String/Object/...).
#[no_mangle]
pub extern "C" fn hudhud_value_display(v: HudValue) -> *mut c_char {
    let text = match v.tag() {
        HudTag::Null => Some("null".to_string()),
        HudTag::Bool => Some(if v.payload != 0 { "true" } else { "false" }.to_string()),
        HudTag::Int => Some(format!("{}", v.payload as i64)),
        HudTag::BigInt => {
            let p = v.payload as usize;
            if p != 0 {
                unsafe {
                    let real = crate::bigint::untag_bigint(p as *mut crate::bigint::HudBigInt);
                    Some(if !real.is_null() { crate::bigint::to_str(&*real) } else { "0".to_string() })
                }
            } else {
                Some("0".to_string())
            }
        }
        _ => None,
    };
    match text {
        Some(s) => CString::new(s).map(|c| c.into_raw()).unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

/// Print a value to stdout. 0 = printed, 1 = tag not supported yet.
#[no_mangle]
pub extern "C" fn hudhud_print(v: HudValue) -> i32 {
    let ptr = hudhud_value_display(v);
    if ptr.is_null() {
        return HUDHUD_PRINT_UNSUPPORTED_TAG;
    }
    unsafe {
        println!("{}", CStr::from_ptr(ptr).to_string_lossy());
    }
    unsafe { hudhud_string_free(ptr) };
    HUDHUD_PRINT_OK
}

/// Free a C string produced by this crate.
///
/// # Safety
/// `s` must originate from `hudhud_value_display` (or another
/// allocator-matched constructor of this crate) and be freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_free(s: *mut c_char) {
    if !s.is_null() {
        type_ops::unregister_string(s as usize);
        // v0.9.41: arena/intern pointer'ları tekil serbest bırakılamaz —
        // slab'ın içindeler (bump alloc). Sadece arena-dışı (büyük, tekil
        // Vec::leak) alloc'lar serbest bırakılır.
        if string_arena::is_arena_pointer(s as usize) {
            return; // arena slab'ı — leak-on-exit sözleşmesi
        }
        // F03: arena-dışı string'ler Rust ayırıcısından (Vec::leak/CString::
        // into_raw) geliyor; CString::from_raw ile serbest bırak.
        drop(CString::from_raw(s));
    }
}

/// Error value for the integer-overflow lane (§18): the JIT exit status
/// carries the same code in `payload`.
#[no_mangle]
pub extern "C" fn hudhud_err_overflow() -> HudValue {
    HudValue::error(HudTag::ErrOverflow)
}

/// Error value for division-by-zero (§18).
#[no_mangle]
pub extern "C" fn hudhud_err_div_zero() -> HudValue {
    HudValue::error(HudTag::ErrDivZero)
}

/// Print an i64 value (JIT-friendly: plain i64 param, no struct passing).
/// If `v` is a registered BigInt handle, prints the arbitrary-precision integer.
#[no_mangle]
pub extern "C" fn hudhud_print_int(v: i64) -> i32 {
    if crate::type_ops::is_bigint(v as u64) {
        unsafe {
            let real = crate::bigint::untag_bigint(v as *mut crate::bigint::HudBigInt);
            if !real.is_null() {
                println!("{}", crate::bigint::to_str(&*real));
            }
        }
    } else {
        println!("{v}");
    }
    HUDHUD_PRINT_OK
}

/// Print a null-terminated string (JIT-friendly: plain ptr param).
#[no_mangle]
pub extern "C" fn hudhud_print_str(s: *const c_char) -> i32 {
    if s.is_null() {
        return HUDHUD_PRINT_OK;
    }
    if crate::type_ops::is_bigint(s as u64) {
        return hudhud_print_int(s as i64);
    }
    unsafe {
        println!("{}", std::ffi::CStr::from_ptr(s).to_string_lossy());
    }
    HUDHUD_PRINT_OK
}

/// Print a float with the VM's formatting (integral values get `.0`).
#[no_mangle]
pub extern "C" fn hudhud_print_float(v: f64) -> i32 {
    if v.is_nan() {
        println!("NaN");
    } else if v.is_infinite() {
        println!("{}", if v > 0.0 { "Infinity" } else { "-Infinity" });
    } else {
        println!("{v}");
    }
    HUDHUD_PRINT_OK
}
