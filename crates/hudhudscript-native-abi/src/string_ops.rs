//! Extended string operations for native ABI (split, indexOf, substring, to_int, char_at).

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::HudArray;

extern "C" {
    fn malloc(size: usize) -> *mut std::ffi::c_void;
    fn memcpy(dest: *mut std::ffi::c_void, src: *const std::ffi::c_void, n: usize) -> *mut std::ffi::c_void;
}

pub(crate) static ASCII_CHARS: [[u8; 2]; 256] = {
    let mut table = [[0u8; 2]; 256];
    let mut b = 0usize;
    while b < 256 {
        table[b][0] = b as u8;
        table[b][1] = 0;
        b += 1;
    }
    table
};

/// String indexing: s[i] -> single-character string handle.
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[no_mangle]
#[link_section = ".hudhud_hot.string_char_at"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_string_char_at(s: *const c_char, i: i64) -> *mut c_char {
    if s.is_null() || i < 0 {
        return ASCII_CHARS[0].as_ptr() as *mut c_char;
    }
    let b = *s.add(i as usize);
    if b == 0 {
        return ASCII_CHARS[0].as_ptr() as *mut c_char;
    }
    ASCII_CHARS[b as u8 as usize].as_ptr() as *mut c_char
}

/// Extract a substring from byte index `start` to `end`.
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_substring(
    s: *const c_char,
    start: i64,
    end: i64,
) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let len = crate::hudhud_string_len(s) as usize;
    let st = if start < 0 { 0 } else { (start as usize).min(len) };
    let en = if end < 0 { st } else { (end as usize).min(len).max(st) };
    let sub_len = en - st;
    if sub_len == 1 {
        let b = *s.add(st);
        return ASCII_CHARS[b as u8 as usize].as_ptr() as *mut c_char;
    }
    let ptr = malloc(sub_len + 1) as *mut c_char;
    if !ptr.is_null() {
        if sub_len > 0 {
            memcpy(ptr as *mut _, s.add(st) as *const _, sub_len);
        }
        *ptr.add(sub_len) = 0;
        crate::set_string_len_cache(ptr as usize, sub_len as i64);
        crate::type_ops::register_string(ptr as usize);
    }
    ptr
}

/// Parse string to integer (used by toNumber/parseInt).
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_to_int(s: *const c_char) -> i64 {
    if s.is_null() {
        return 0;
    }
    let cstr = CStr::from_ptr(s);
    let slice = cstr.to_str().unwrap_or("0").trim();
    slice.parse::<i64>().unwrap_or(0)
}

/// Split a string by a delimiter into an array of string handles.
///
/// # Safety
/// `s` and `delim` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_split(
    s: *const c_char,
    delim: *const c_char,
) -> *mut HudArray {
    if s.is_null() {
        return crate::array::hudhud_array_new(0);
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => {
            return crate::array::hudhud_array_new(0);
        }
    };
    let delim_str = if delim.is_null() {
        ""
    } else {
        CStr::from_ptr(delim).to_str().unwrap_or("")
    };

    let parts: Vec<i64> = if delim_str.is_empty() {
        s_str
            .chars()
            .map(|c| {
                let mut buf = [0u8; 4];
                let s_char = c.encode_utf8(&mut buf);
                let len = s_char.len();
                let ptr = malloc(len + 1) as *mut c_char;
                if !ptr.is_null() {
                    memcpy(ptr as *mut _, s_char.as_ptr() as *const _, len);
                    *ptr.add(len) = 0;
                    crate::type_ops::register_string(ptr as usize);
                }
                ptr as i64
            })
            .collect()
    } else {
        s_str
            .split(delim_str)
            .map(|part| {
                let len = part.len();
                let ptr = malloc(len + 1) as *mut c_char;
                if !ptr.is_null() {
                    memcpy(ptr as *mut _, part.as_ptr() as *const _, len);
                    *ptr.add(len) = 0;
                    crate::type_ops::register_string(ptr as usize);
                }
                ptr as i64
            })
            .collect()
    };

    crate::array::hudhud_array_from_vec(parts)
}

/// Find index of first occurrence of needle in string. Returns -1 if not found.
///
/// # Safety
/// `s` and `needle` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_index_of(
    s: *const c_char,
    needle: *const c_char,
) -> i64 {
    if s.is_null() || needle.is_null() {
        return -1;
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let needle_str = match CStr::from_ptr(needle).to_str() {
        Ok(n) => n,
        Err(_) => return -1,
    };
    s_str.find(needle_str).map(|idx| idx as i64).unwrap_or(-1)
}

/// Trim leading and trailing whitespace from a string.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_trim(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s.trim(),
        Err(_) => return std::ptr::null_mut(),
    };
    match CString::new(s_str) {
        Ok(c) => {
            let raw = c.into_raw();
            crate::type_ops::register_string(raw as usize);
            raw
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Check if string starts with prefix. Returns 1 if true, 0 if false.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_starts_with(
    s: *const c_char,
    prefix: *const c_char,
) -> i64 {
    if s.is_null() || prefix.is_null() {
        return 0;
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let prefix_str = match CStr::from_ptr(prefix).to_str() {
        Ok(p) => p,
        Err(_) => return 0,
    };
    if s_str.starts_with(prefix_str) { 1 } else { 0 }
}

/// Check if string ends with suffix. Returns 1 if true, 0 if false.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_ends_with(
    s: *const c_char,
    suffix: *const c_char,
) -> i64 {
    if s.is_null() || suffix.is_null() {
        return 0;
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let suffix_str = match CStr::from_ptr(suffix).to_str() {
        Ok(p) => p,
        Err(_) => return 0,
    };
    if s_str.ends_with(suffix_str) { 1 } else { 0 }
}

/// Check if string contains needle. Returns 1 if true, 0 if false.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_contains(
    s: *const c_char,
    needle: *const c_char,
) -> i64 {
    if hudhud_string_index_of(s, needle) >= 0 { 1 } else { 0 }
}

/// Replace all occurrences of `from` with `to`.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_replace(
    s: *const c_char,
    from: *const c_char,
    to: *const c_char,
) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let from_str = if from.is_null() { "" } else { CStr::from_ptr(from).to_str().unwrap_or("") };
    let to_str = if to.is_null() { "" } else { CStr::from_ptr(to).to_str().unwrap_or("") };
    let replaced = s_str.replace(from_str, to_str);
    match CString::new(replaced) {
        Ok(c) => {
            let raw = c.into_raw();
            crate::type_ops::register_string(raw as usize);
            raw
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert string to lowercase.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_to_lower(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s.to_lowercase(),
        Err(_) => return std::ptr::null_mut(),
    };
    match CString::new(s_str) {
        Ok(c) => {
            let raw = c.into_raw();
            crate::type_ops::register_string(raw as usize);
            raw
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert string to uppercase.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_to_upper(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s.to_uppercase(),
        Err(_) => return std::ptr::null_mut(),
    };
    match CString::new(s_str) {
        Ok(c) => {
            let raw = c.into_raw();
            crate::type_ops::register_string(raw as usize);
            raw
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Get Unicode character code at byte or char index.
#[no_mangle]
pub unsafe extern "C" fn hudhud_string_char_code_at(s: *const c_char, idx: i64) -> i64 {
    if s.is_null() || idx < 0 {
        return -1;
    }
    let s_str = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    s_str.chars().nth(idx as usize).map(|c| c as i64).unwrap_or(-1)
}

