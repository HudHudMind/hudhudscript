//! Array ABI (Vec<i64> behind opaque handle).

use std::ffi::c_char;

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn malloc(size: usize) -> *mut std::ffi::c_void;
    fn memcpy(dest: *mut std::ffi::c_void, src: *const std::ffi::c_void, n: usize) -> *mut std::ffi::c_void;
}

pub const HUD_ARRAY_MAGIC: u32 = 0x48554441; // 'HUDA'

/// Opaque array handle (Flat C-layout with header magic).
#[repr(C)]
pub struct HudArray {
    pub magic: u32,
    pub _reserved: u32,
    pub ptr: *mut i64,
    pub len: usize,
    pub cap: usize,
}

/// Construct a HudArray directly from an existing Rust Vec<i64>.
pub fn hudhud_array_from_vec(mut v: Vec<i64>) -> *mut HudArray {
    let ptr = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    std::mem::forget(v);
    let p = Box::into_raw(Box::new(HudArray {
        magic: HUD_ARRAY_MAGIC,
        _reserved: 0,
        ptr,
        len,
        cap,
    }));
    crate::type_ops::register_array(p as usize);
    p
}

/// Create a new array with the given initial capacity. Returns an opaque handle.
#[no_mangle]
pub extern "C" fn hudhud_array_new(capacity: i64) -> *mut HudArray {
    let cap = if capacity > 0 { capacity as usize } else { 0 };
    hudhud_array_from_vec(Vec::with_capacity(cap))
}

/// Create a new pre-filled array of given length with initial value.
#[no_mangle]
pub extern "C" fn hudhud_array_filled(length: i64, value: i64) -> *mut HudArray {
    let len = if length > 0 { length as usize } else { 0 };
    hudhud_array_from_vec(vec![value; len])
}

/// Fill an existing array up to length elements with value.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_fill(arr: *mut HudArray, length: i64, value: i64) {
    if !arr.is_null() && length > 0 {
        let a = &mut *arr;
        let mut v = Vec::from_raw_parts(a.ptr, a.len, a.cap);
        v.resize(length as usize, value);
        a.ptr = v.as_mut_ptr();
        a.len = v.len();
        a.cap = v.capacity();
        std::mem::forget(v);
    }
}

/// Push a value onto the array.
/// Fast-path: direct store into pre-allocated buffer when len < cap.
///
/// # Safety
/// `arr` must originate from `hudhud_array_new`.
#[no_mangle]
#[link_section = ".hudhud_hot.array_push"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_array_push(arr: *mut HudArray, value: i64) {
    if !arr.is_null() {
        let a = &mut *arr;
        if a.len < a.cap {
            *a.ptr.add(a.len) = value;
            a.len += 1;
            return;
        }
        hudhud_array_push_slow(a, value);
    }
}

#[cold]
#[inline(never)]
unsafe fn hudhud_array_push_slow(a: &mut HudArray, value: i64) {
    let mut v = Vec::from_raw_parts(a.ptr, a.len, a.cap);
    v.push(value);
    a.ptr = v.as_mut_ptr();
    a.len = v.len();
    a.cap = v.capacity();
    std::mem::forget(v);
}

/// Get the value at index. Returns 0 on out-of-bounds (VM oracle: error; v1: 0).
/// Fast-path: single pointer offset load without Vec overhead.
///
/// # Safety
/// `arr` must originate from `hudhud_array_new`.
#[no_mangle]
#[link_section = ".hudhud_hot.array_get"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_array_get(arr: *mut HudArray, index: i64) -> i64 {
    if arr.is_null() || index < 0 {
        return 0;
    }
    let idx = index as usize;
    let a = &*arr;
    if idx < a.len {
        *a.ptr.add(idx)
    } else {
        0
    }
}

/// Set the value at index (in-bounds: direct store; out-of-bounds: slow path resize).
///
/// # Safety
/// `arr` must originate from `hudhud_array_new`.
#[no_mangle]
#[link_section = ".hudhud_hot.array_set"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_array_set(arr: *mut HudArray, index: i64, value: i64) {
    if arr.is_null() || index < 0 {
        return;
    }
    let idx = index as usize;
    let a = &mut *arr;
    if idx < a.len {
        *a.ptr.add(idx) = value;
    } else {
        hudhud_array_set_slow(a, idx, value);
    }
}

#[cold]
#[inline(never)]
unsafe fn hudhud_array_set_slow(a: &mut HudArray, idx: usize, value: i64) {
    let mut v = Vec::from_raw_parts(a.ptr, a.len, a.cap);
    if idx >= v.len() {
        v.resize(idx + 1, 0);
    }
    v[idx] = value;
    a.ptr = v.as_mut_ptr();
    a.len = v.len();
    a.cap = v.capacity();
    std::mem::forget(v);
}

/// Get the length of the array.
///
/// # Safety
/// `arr` must originate from `hudhud_array_new`.
#[no_mangle]
#[link_section = ".hudhud_hot.array_len"]
#[inline(never)]
pub unsafe extern "C" fn hudhud_array_len(arr: *mut HudArray) -> i64 {
    if arr.is_null() {
        0
    } else {
        (*arr).len as i64
    }
}

/// Pop the last value off the array. Returns 0 when empty.
///
/// # Safety
/// `arr` must originate from `hudhud_array_new`.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_pop(arr: *mut HudArray) -> i64 {
    if arr.is_null() || (*arr).len == 0 {
        return 0;
    }
    let a = &mut *arr;
    let mut v = Vec::from_raw_parts(a.ptr, a.len, a.cap);
    let val = v.pop().unwrap_or(0);
    a.ptr = v.as_mut_ptr();
    a.len = v.len();
    a.cap = v.capacity();
    std::mem::forget(v);
    val
}

/// Free an array.
///
/// # Safety
/// `arr` must originate from `hudhud_array_new` and freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_free(arr: *mut HudArray) {
    if !arr.is_null() {
        crate::type_ops::unregister_array(arr as usize);
        let a = Box::from_raw(arr);
        let v = Vec::from_raw_parts(a.ptr, a.len, a.cap);
        drop(v);
        drop(a);
    }
}

/// Join elements of an array of string handles with a separator.
///
/// # Safety
/// `arr` must originate from `hudhud_array_new`. `sep` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_join(arr: *const HudArray, sep: *const c_char) -> *mut c_char {
    if arr.is_null() {
        return std::ptr::null_mut();
    }
    let sep_bytes = if sep.is_null() { &[][..] } else { std::ffi::CStr::from_ptr(sep).to_bytes() };
    let a = &*arr;
    let slice = std::slice::from_raw_parts(a.ptr, a.len);
    let mut total_len = 0;
    for (idx, &item) in slice.iter().enumerate() {
        if idx > 0 {
            total_len += sep_bytes.len();
        }
        if item != 0 {
            total_len += strlen(item as *const c_char);
        }
    }
    let ptr = malloc(total_len + 1) as *mut c_char;
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    let mut offset = 0;
    for (idx, &item) in slice.iter().enumerate() {
        if idx > 0 && !sep_bytes.is_empty() {
            memcpy(ptr.add(offset) as *mut _, sep_bytes.as_ptr() as *const _, sep_bytes.len());
            offset += sep_bytes.len();
        }
        if item != 0 {
            let el_len = strlen(item as *const c_char);
            if el_len > 0 {
                memcpy(ptr.add(offset) as *mut _, item as *const _, el_len);
                offset += el_len;
            }
        }
    }
    *ptr.add(offset) = 0;
    crate::type_ops::register_string(ptr as usize);
    ptr
}

/// Extract a slice of array from index `start` to `end`.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_slice(
    arr: *const HudArray,
    start: i64,
    end: i64,
) -> *mut HudArray {
    if arr.is_null() {
        return hudhud_array_new(0);
    }
    let a = &*arr;
    let len = a.len;
    let st = if start < 0 { (len as i64 + start).max(0) as usize } else { (start as usize).min(len) };
    let en = if end < 0 { (len as i64 + end).max(0) as usize } else { (end as usize).min(len).max(st) };
    if st >= en {
        return hudhud_array_new(0);
    }
    let slice = std::slice::from_raw_parts(a.ptr.add(st), en - st);
    hudhud_array_from_vec(slice.to_vec())
}

/// Reverse the elements of an array in-place.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_reverse(arr: *mut HudArray) {
    if !arr.is_null() {
        let a = &mut *arr;
        if a.len > 1 {
            let slice = std::slice::from_raw_parts_mut(a.ptr, a.len);
            slice.reverse();
        }
    }
}

/// Concatenate two arrays into a new array.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_concat(
    a: *const HudArray,
    b: *const HudArray,
) -> *mut HudArray {
    let mut result = Vec::new();
    if !a.is_null() {
        let a_ref = &*a;
        result.extend_from_slice(std::slice::from_raw_parts(a_ref.ptr, a_ref.len));
    }
    if !b.is_null() {
        let b_ref = &*b;
        result.extend_from_slice(std::slice::from_raw_parts(b_ref.ptr, b_ref.len));
    }
    hudhud_array_from_vec(result)
}

/// Find index of item in array, or -1 if not found.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_index_of(arr: *const HudArray, item: i64) -> i64 {
    if arr.is_null() {
        return -1;
    }
    let a = &*arr;
    let slice = std::slice::from_raw_parts(a.ptr, a.len);
    for (i, &val) in slice.iter().enumerate() {
        if val == item {
            return i as i64;
        }
    }
    -1
}

/// Check if array contains item. Returns 1 if true, 0 if false.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_includes(arr: *const HudArray, item: i64) -> i64 {
    if hudhud_array_index_of(arr, item) >= 0 { 1 } else { 0 }
}

/// Remove and return the first element of an array.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_shift(arr: *mut HudArray) -> i64 {
    if arr.is_null() || (*arr).len == 0 {
        return 0;
    }
    let a = &mut *arr;
    let mut v = Vec::from_raw_parts(a.ptr, a.len, a.cap);
    let val = v.remove(0);
    a.ptr = v.as_mut_ptr();
    a.len = v.len();
    a.cap = v.capacity();
    std::mem::forget(v);
    val
}

/// Prepend an element to the beginning of an array.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_unshift(arr: *mut HudArray, val: i64) {
    if !arr.is_null() {
        let a = &mut *arr;
        let mut v = Vec::from_raw_parts(a.ptr, a.len, a.cap);
        v.insert(0, val);
        a.ptr = v.as_mut_ptr();
        a.len = v.len();
        a.cap = v.capacity();
        std::mem::forget(v);
    }
}

/// Sort array elements in-place using fast introsort.
#[no_mangle]
pub unsafe extern "C" fn hudhud_array_sort(arr: *mut HudArray) {
    if !arr.is_null() {
        let a = &mut *arr;
        if a.len > 1 {
            let slice = std::slice::from_raw_parts_mut(a.ptr, a.len);
            slice.sort_unstable();
        }
    }
}

