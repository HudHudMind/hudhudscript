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
        // F03: TÜM string'ler Rust ayırıcısından (Vec::leak/CString::into_raw)
        // geliyor; CString::from_raw ile serbest bırak. STR_LEN_CACHE ve
        // LAST_APPEND'in adres ömür varsayımları unregister ile temizlenir.
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

// ── String ABI ────────────────────────────────────────────────────────

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

/// Rust ayırıcısından (mimalloc) NUL-sonlu string tamponu ayır (F03).
/// Vec::leak → CString::from_raw ile serbest bırakılabilir.
unsafe fn rust_alloc_string(cap: usize) -> *mut c_char {
    let mut buf = Vec::with_capacity(cap);
    // Vec kapasitesi = ayırılan bayt; içerik caller tarafından yazılır
    buf.resize(cap, 0);
    let arr = buf.leak();
    arr.as_mut_ptr() as *mut c_char
}

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn malloc(size: usize) -> *mut std::ffi::c_void;
    fn memcpy(dest: *mut std::ffi::c_void, src: *const std::ffi::c_void, n: usize) -> *mut std::ffi::c_void;
}

thread_local! {
    static STR_LEN_CACHE: std::cell::Cell<[(usize, i64); 64]> = const { std::cell::Cell::new([(0, 0); 64]) };
    static LAST_APPEND: std::cell::Cell<(usize, usize, usize)> = const { std::cell::Cell::new((0, 0, 0)) };
}

#[inline]
pub(crate) fn set_string_len_cache(ptr: usize, len: i64) {
    if ptr != 0 {
        let idx = (ptr >> 4) & 63;
        STR_LEN_CACHE.with(|c| {
            let mut arr = c.get();
            arr[idx] = (ptr, len);
            c.set(arr);
        });
    }
}

/// Concatenate two null-terminated strings. Returns a new heap-allocated
/// C string (caller must `hudhud_string_free`). Returns null on OOM.
///
/// # Safety
/// `a` and `b` must be valid null-terminated C strings.
#[no_mangle]
#[link_section = ".hudhud_hot.string_concat"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_string_concat(a: *const c_char, b: *const c_char) -> *mut c_char {
    let mut a_tmp = std::ptr::null_mut();
    let mut b_tmp = std::ptr::null_mut();
    let a_str = if crate::type_ops::is_bigint(a as u64) {
        a_tmp = hudhud_int_to_string(a as i64);
        a_tmp as *const c_char
    } else {
        a
    };
    let b_str = if crate::type_ops::is_bigint(b as u64) {
        b_tmp = hudhud_int_to_string(b as i64);
        b_tmp as *const c_char
    } else {
        b
    };
    let la = if a_str.is_null() { 0 } else { strlen(a_str) };
    let lb = if b_str.is_null() { 0 } else { strlen(b_str) };
    // F03: Rust ayırıcısı (mimalloc) — CString::from_raw serbest bırakabilir.
    // C malloc KULLANILMAZ: CString::from_raw ile çapraz-free UB'dir.
    let mut buf = Vec::with_capacity(la + lb + 1);
    unsafe {
        if la > 0 {
            buf.extend_from_slice(std::slice::from_raw_parts(a_str as *const u8, la));
        }
        if lb > 0 {
            buf.extend_from_slice(std::slice::from_raw_parts(b_str as *const u8, lb));
        }
    }
    buf.push(0);
    if !a_tmp.is_null() { hudhud_string_free(a_tmp); }
    if !b_tmp.is_null() { hudhud_string_free(b_tmp); }
    let arr = buf.leak();
    let ptr = arr.as_mut_ptr() as *mut c_char;
    type_ops::register_string(ptr as usize);
    set_string_len_cache(ptr as usize, (la + lb) as i64);
    ptr
}

/// In-place string append with exponential capacity growth for self-assignment `s = s + suffix`.
///
/// # Safety
/// `a` and `b` must be valid null-terminated C strings.
#[no_mangle]
#[link_section = ".hudhud_hot.string_append"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_string_append(a: *mut c_char, b: *const c_char) -> *mut c_char {
    if a.is_null() {
        return hudhud_string_concat(a, b);
    }
    let lb = if b.is_null() { 0 } else { hudhud_string_len(b) as usize };
    let (last_ptr, last_cap, last_len) = LAST_APPEND.with(|c| c.get());
    if a as usize == last_ptr && last_ptr != 0 {
        let needed = last_len + lb + 1;
        if last_cap >= needed {
            if lb > 0 {
                memcpy(a.add(last_len) as *mut _, b as *const _, lb);
            }
            *a.add(last_len + lb) = 0;
            LAST_APPEND.with(|c| c.set((last_ptr, last_cap, last_len + lb)));
            set_string_len_cache(last_ptr, (last_len + lb) as i64);
            return a;
        }
        let new_cap = needed.max(last_cap * 2).max(256);
        let new_ptr = rust_alloc_string(new_cap);
        if new_ptr.is_null() {
            return std::ptr::null_mut();
        }
        if last_len > 0 {
            memcpy(new_ptr as *mut _, a as *const _, last_len);
        }
        if lb > 0 {
            memcpy(new_ptr.add(last_len) as *mut _, b as *const _, lb);
        }
        *new_ptr.add(last_len + lb) = 0;
        LAST_APPEND.with(|c| c.set((new_ptr as usize, new_cap, last_len + lb)));
        set_string_len_cache(new_ptr as usize, (last_len + lb) as i64);
        type_ops::register_string(new_ptr as usize);
        return new_ptr;
    }
    let la = strlen(a);
    let cap = (la + lb + 1).max(la * 2).max(256);
    let ptr = rust_alloc_string(cap);
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    if la > 0 {
        memcpy(ptr as *mut _, a as *const _, la);
    }
    if lb > 0 {
        memcpy(ptr.add(la) as *mut _, b as *const _, lb);
    }
    *ptr.add(la + lb) = 0;
    LAST_APPEND.with(|c| c.set((ptr as usize, cap, la + lb)));
    set_string_len_cache(ptr as usize, (la + lb) as i64);
    type_ops::register_string(ptr as usize);
    ptr
}

/// Get the length of a null-terminated string in bytes.
#[no_mangle]
#[link_section = ".hudhud_hot.string_len"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_string_len(s: *const c_char) -> i64 {
    if s.is_null() {
        return 0;
    }
    let u = s as u64;
    if crate::type_ops::is_bigint(u) {
        let real = crate::bigint::untag_bigint(s as *mut crate::bigint::HudBigInt);
        return if !real.is_null() { crate::bigint::to_str(&*real).len() as i64 } else { 1 };
    }
    // Önbellek anahtarı usize şeridinde (32-bit'te budanır — anahtar
    // niteliği için yeterli); BigInt tag denetimi yukarıda u64 ile yapıldı.
    let key = u as usize;
    let idx = ((key >> 4) & 63) as usize;
    STR_LEN_CACHE.with(|c| {
        let mut arr = c.get();
        if arr[idx].0 == key {
            return arr[idx].1;
        }
        let len = strlen(s) as i64;
        arr[idx] = (key, len);
        c.set(arr);
        len
    })
}

/// Compare two null-terminated strings. Returns 1 if equal, 0 if not.
#[no_mangle]
#[link_section = ".hudhud_hot.string_eq"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_string_eq(a: *const c_char, b: *const c_char) -> i64 {
    if a == b {
        return 1;
    }
    if a.is_null() || b.is_null() {
        return 0;
    }
    if crate::type_ops::is_bigint(a as u64) || crate::type_ops::is_bigint(b as u64) {
        return 0;
    }
    let ca = *a;
    let cb = *b;
    if ca != cb {
        return 0;
    }
    if ca == 0 {
        return 1;
    }
    let ca1 = *a.add(1);
    let cb1 = *b.add(1);
    if ca1 != cb1 {
        return 0;
    }
    if ca1 == 0 {
        return 1;
    }
    extern "C" {
        fn strcmp(s1: *const c_char, s2: *const c_char) -> std::ffi::c_int;
    }
    if strcmp(a.add(2), b.add(2)) == 0 { 1 } else { 0 }
}

/// Compare two null-terminated strings lexicographically.
/// Returns <0 if a < b, 0 if a == b, >0 if a > b.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_cmp(a: *const c_char, b: *const c_char) -> i64 {
    if a == b {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }
    extern "C" {
        fn strcmp(s1: *const c_char, s2: *const c_char) -> std::ffi::c_int;
    }
    strcmp(a, b) as i64
}

/// Convert an i64 to a heap-allocated string (for `"Result: " + sum`).
/// Caller must `hudhud_string_free`.
#[no_mangle]
pub extern "C" fn hudhud_int_to_string(v: i64) -> *mut c_char {
    let s = if crate::type_ops::is_bigint(v as u64) {
        unsafe {
            let real = crate::bigint::untag_bigint(v as *mut crate::bigint::HudBigInt);
            if !real.is_null() { crate::bigint::to_str(&*real) } else { "0".to_string() }
        }
    } else {
        format!("{v}")
    };
    match std::ffi::CString::new(s) {
        Ok(c) => {
            let bytes = c.as_bytes();
            let len = bytes.len() as i64;
            let ptr = c.into_raw();
            type_ops::register_string(ptr as usize);
            set_string_len_cache(ptr as usize, len);
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert an f64 to a heap-allocated string.
#[no_mangle]
pub extern "C" fn hudhud_float_to_string(v: f64) -> *mut c_char {
    // VM oracle biçimi: integral float'lar ".0"sız (5, 1010093), ondalıklar
    // kısa gösterim (6.25) — Rust `{}` ikisini de birebir üretir.
    let s = format!("{v}");
    match std::ffi::CString::new(s) {
        Ok(c) => {
            let bytes = c.as_bytes();
            let len = bytes.len() as i64;
            let ptr = c.into_raw();
            type_ops::register_string(ptr as usize);
            set_string_len_cache(ptr as usize, len);
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
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
