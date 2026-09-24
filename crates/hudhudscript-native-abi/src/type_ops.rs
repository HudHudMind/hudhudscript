//! Dynamic type introspection (typeof) and runtime handle checks.

use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_char;

thread_local! {
    static STRING_REGISTRY: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());
}

#[inline(always)]
pub fn register_object(_ptr: usize) {}

#[inline(always)]
pub fn unregister_object(_ptr: usize) {}

#[inline(always)]
pub fn register_array(_ptr: usize) {}

#[inline(always)]
pub fn unregister_array(_ptr: usize) {}

#[inline(always)]
pub fn register_string(ptr: usize) {
    if ptr != 0 {
        STRING_REGISTRY.with(|reg| reg.borrow_mut().insert(ptr));
    }
}

#[inline(always)]
pub fn unregister_string(ptr: usize) {
    if ptr != 0 {
        STRING_REGISTRY.with(|reg| reg.borrow_mut().remove(&ptr));
    }
}

#[inline(always)]
pub fn register_bigint(_ptr: usize) {}

#[inline(always)]
pub fn unregister_bigint(_ptr: usize) {}

#[inline(always)]
pub fn is_bigint(raw: u64) -> bool {
    // F02: etiket + MAGIC — inline düzeyde hızlı (H.6: fast-path önce).
    // fib(1000) döngüsünde her toplamada 2 kez çağrılır; branch predictor
    // hep aynı yolu tahmin eder → ~1ns. MAGIC read L1-cached.
    // Kodlama 64-bit ham kelime üzerindedir — 32-bit hedeflerde usize
    // daralır ve tag kaybolurdu; u64 bit-korur taşır (v0.9.25).
    if (raw & 0xFFFF_0000_0000_0000) != crate::bigint::HUD_BIGINT_TAG {
        return false;
    }
    let ptr = (raw & crate::bigint::HUD_BIGINT_PTR_MASK) as *const u32;
    // HudBigInt::magic ilk 4 bayt — hizalı read, cache-friendly
    unsafe { *ptr == crate::bigint::HUD_BIGINT_MAGIC }
}

#[inline(always)]
pub fn is_object(raw: u64) -> bool {
    if raw < 0x0040_0000 || raw >= 0x8000_0000_0000 || (raw & 7) != 0 {
        return false;
    }
    unsafe { *(raw as *const u32) == crate::object::HUD_OBJECT_MAGIC }
}

#[inline(always)]
pub fn is_array(raw: u64) -> bool {
    if raw < 0x0040_0000 || raw >= 0x8000_0000_0000 || (raw & 7) != 0 {
        return false;
    }
    unsafe { *(raw as *const u32) == crate::array::HUD_ARRAY_MAGIC }
}

#[inline(always)]
pub fn is_string(raw: u64) -> bool {
    if raw == 0 {
        return false;
    }
    let ascii_start = crate::string_ops::ASCII_CHARS.as_ptr() as u64;
    let ascii_end = ascii_start + 512;
    if raw >= ascii_start && raw < ascii_end {
        return true;
    }
    let ptr = raw as usize;
    STRING_REGISTRY.with(|reg| reg.borrow().contains(&ptr))
}

static TYPE_NULL: &[u8] = b"null\0";
static TYPE_OBJECT: &[u8] = b"object\0";
static TYPE_ARRAY: &[u8] = b"array\0";
static TYPE_STRING: &[u8] = b"string\0";
static TYPE_BIGINT: &[u8] = b"bigint\0";
static TYPE_NUMBER: &[u8] = b"number\0";

/// Runtime `typeof(value)`. Returns static null-terminated C-string.
#[no_mangle]
pub extern "C" fn hudhud_typeof(val: i64) -> *const c_char {
    if val == 0 {
        return TYPE_NULL.as_ptr() as *const c_char;
    }
    let u = val as u64;
    if is_bigint(u) {
        return TYPE_BIGINT.as_ptr() as *const c_char;
    }
    if is_object(u) {
        return TYPE_OBJECT.as_ptr() as *const c_char;
    }
    if is_array(u) {
        return TYPE_ARRAY.as_ptr() as *const c_char;
    }
    if is_string(u) {
        return TYPE_STRING.as_ptr() as *const c_char;
    }
    TYPE_NUMBER.as_ptr() as *const c_char
}

/// Register a string pointer into the runtime string registry.
#[no_mangle]
pub extern "C" fn hudhud_register_string(ptr: *const c_char) {
    register_string(ptr as usize);
}
