//! Temel C String işlemleri ve uzunluk önbellekleme (F03, v0.9.44).
//!
//! `lib.rs` 400 satır sınırını korumak için string birleştirme, ekleme,
//! uzunluk ve karşılaştırma mantığı bu modüle taşınmıştır.

use std::ffi::{c_char, CString};

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn memcpy(dest: *mut std::ffi::c_void, src: *const std::ffi::c_void, n: usize) -> *mut std::ffi::c_void;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> std::ffi::c_int;
}

thread_local! {
    static STR_LEN_CACHE: std::cell::RefCell<[(usize, i64); 64]> = const { std::cell::RefCell::new([(0, 0); 64]) };
    /// K-Nucleotide deseni: döngü içinde 250 KB'lık ana dizi sürekli alt stringlere
    /// dilimlenir. 64 girişli önbellek her 64 alt stringde bir sıra dolanımıyla ana
    /// diziyi ezmesin diye son uzun string (>256 B) ayrı hücrede kilitlenir.
    static LAST_LONG_STR: std::cell::Cell<(usize, i64)> = const { std::cell::Cell::new((0, 0)) };
    static LAST_APPEND: std::cell::Cell<(usize, usize, usize)> = const { std::cell::Cell::new((0, 0, 0)) };
}

#[inline]
pub(crate) fn set_string_len_cache(ptr: usize, len: i64) {
    if ptr != 0 {
        let idx = (ptr >> 4) & 63;
        STR_LEN_CACHE.with(|c| {
            c.borrow_mut()[idx] = (ptr, len);
        });
        if len > 256 {
            LAST_LONG_STR.with(|ll| ll.set((ptr, len)));
        }
    }
}

/// Rust ayırıcısından (mimalloc) NUL-sonlu string tamponu ayır (F03).
/// Vec::leak → CString::from_raw ile serbest bırakılabilir.
pub(crate) unsafe fn rust_alloc_string(cap: usize) -> *mut c_char {
    let mut buf = Vec::with_capacity(cap);
    buf.resize(cap, 0);
    let arr = buf.leak();
    arr.as_mut_ptr() as *mut c_char
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
    let la = if a_str.is_null() { 0 } else { hudhud_string_len(a_str) as usize };
    let lb = if b_str.is_null() { 0 } else { hudhud_string_len(b_str) as usize };
    let total_len = la + lb;
    if total_len <= 32 {
        let mut stack_buf = [0u8; 32];
        if la > 0 {
            std::ptr::copy_nonoverlapping(a_str as *const u8, stack_buf.as_mut_ptr(), la);
        }
        if lb > 0 {
            std::ptr::copy_nonoverlapping(b_str as *const u8, stack_buf.as_mut_ptr().add(la), lb);
        }
        if !a_tmp.is_null() { crate::hudhud_string_free(a_tmp); }
        if !b_tmp.is_null() { crate::hudhud_string_free(b_tmp); }
        let ptr = crate::string_arena::intern_or_alloc(&stack_buf[..total_len], total_len);
        set_string_len_cache(ptr as usize, total_len as i64);
        return ptr;
    }
    let ptr = crate::string_arena::arena_alloc(total_len);
    if !ptr.is_null() {
        if la > 0 {
            std::ptr::copy_nonoverlapping(a_str as *const u8, ptr as *mut u8, la);
        }
        if lb > 0 {
            std::ptr::copy_nonoverlapping(b_str as *const u8, (ptr as *mut u8).add(la), lb);
        }
        *ptr.add(total_len) = 0;
        if !a_tmp.is_null() { crate::hudhud_string_free(a_tmp); }
        if !b_tmp.is_null() { crate::hudhud_string_free(b_tmp); }
        set_string_len_cache(ptr as usize, total_len as i64);
        return ptr;
    }
    let mut buf = Vec::with_capacity(total_len + 1);
    if la > 0 {
        buf.extend_from_slice(std::slice::from_raw_parts(a_str as *const u8, la));
    }
    if lb > 0 {
        buf.extend_from_slice(std::slice::from_raw_parts(b_str as *const u8, lb));
    }
    buf.push(0);
    if !a_tmp.is_null() { crate::hudhud_string_free(a_tmp); }
    if !b_tmp.is_null() { crate::hudhud_string_free(b_tmp); }
    let arr = buf.leak();
    let ptr = arr.as_mut_ptr() as *mut c_char;
    crate::type_ops::register_string(ptr as usize);
    set_string_len_cache(ptr as usize, total_len as i64);
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
        crate::type_ops::register_string(new_ptr as usize);
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
    crate::type_ops::register_string(ptr as usize);
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
    let key = u as usize;
    // Hızlı yol 1: son uzun string (>256B) eşleşmesi — K-Nucleotide O(1)
    let (last_long, last_len) = LAST_LONG_STR.with(|c| c.get());
    if key == last_long && key != 0 {
        return last_len;
    }
    // Hızlı yol 2: doğrudan 64 girişli önbellek
    let idx = ((key >> 4) & 63) as usize;
    STR_LEN_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache[idx].0 == key {
            return cache[idx].1;
        }
        let len = strlen(s) as i64;
        cache[idx] = (key, len);
        if len > 256 {
            LAST_LONG_STR.with(|ll| ll.set((key, len)));
        }
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
    match CString::new(s) {
        Ok(c) => {
            let bytes = c.as_bytes();
            let len = bytes.len() as i64;
            let ptr = c.into_raw();
            crate::type_ops::register_string(ptr as usize);
            set_string_len_cache(ptr as usize, len);
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert an f64 to a heap-allocated string.
#[no_mangle]
pub extern "C" fn hudhud_float_to_string(v: f64) -> *mut c_char {
    let s = format!("{v}");
    match CString::new(s) {
        Ok(c) => {
            let bytes = c.as_bytes();
            let len = bytes.len() as i64;
            let ptr = c.into_raw();
            crate::type_ops::register_string(ptr as usize);
            set_string_len_cache(ptr as usize, len);
            ptr
        }
        Err(_) => std::ptr::null_mut(),
    }
}
