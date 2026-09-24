//! Object ABI (v1: HashMap<String, i64> behind opaque handle; GC §17 pending).
//!
//! i64-handle lane'de değerler i64 olarak taşınır (int/bool doğrudan,
//! string/array/object opak handle). Anahtarlar C string olarak verilir;
//! property isimleri derleme zamanında bilinir (ConstString → handle).

use std::collections::HashMap;
use std::ffi::{c_char, CStr};

pub const HUD_OBJECT_MAGIC: u32 = 0x4855444F; // 'HUDO'

extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> std::ffi::c_int;
}

#[inline(always)]
unsafe fn keys_match(k1: *const c_char, k2: *const c_char) -> bool {
    if k1 == k2 {
        return true;
    }
    if k1.is_null() || k2.is_null() {
        return false;
    }
    strcmp(k1, k2) == 0
}

/// Opaque object handle with 4 fast inline slots and overflow HashMap.
#[repr(C)]
pub struct HudObject {
    pub magic: u32,
    pub num_fields: u16,
    pub keys: [*const c_char; 4],
    pub values: [i64; 4],
    pub overflow: *mut HashMap<String, i64>,
}

unsafe fn key_from<'a>(key: *const c_char) -> Option<&'a str> {
    if key.is_null() {
        return None;
    }
    CStr::from_ptr(key).to_str().ok()
}

/// Create a new empty object. Returns an opaque handle.
#[no_mangle]
pub extern "C" fn hudhud_object_new() -> *mut HudObject {
    let ptr = Box::into_raw(Box::new(HudObject {
        magic: HUD_OBJECT_MAGIC,
        num_fields: 0,
        keys: [std::ptr::null(); 4],
        values: [0; 4],
        overflow: std::ptr::null_mut(),
    }));
    crate::type_ops::register_object(ptr as usize);
    ptr
}

/// Set a property. Null handle or invalid key is a no-op.
/// Fast-path: up to 4 inline slots without HashMap allocation or string cloning.
///
/// # Safety
/// `obj` must originate from `hudhud_object_new`.
#[no_mangle]
pub unsafe extern "C" fn hudhud_object_set(obj: *mut HudObject, key: *const c_char, value: i64) {
    if obj.is_null() || key.is_null() {
        return;
    }
    let o = &mut *obj;
    // Fast path: update existing inline property
    for i in 0..o.num_fields as usize {
        if keys_match(o.keys[i], key) {
            o.values[i] = value;
            return;
        }
    }
    // Fast path: append new property into inline slots
    let n = o.num_fields as usize;
    if n < 4 {
        o.keys[n] = key;
        o.values[n] = value;
        o.num_fields = (n + 1) as u16;
        return;
    }
    // Slow path: overflow to HashMap
    if let Some(k) = key_from(key) {
        if o.overflow.is_null() {
            o.overflow = Box::into_raw(Box::new(HashMap::new()));
        }
        (*o.overflow).insert(k.to_string(), value);
    }
}

/// Get a property. Returns 0 when missing (VM oracle: Null; v1 lane: 0).
/// Fast-path: inline slot search with pointer/strcmp comparison.
///
/// # Safety
/// `obj` must originate from `hudhud_object_new`.
#[no_mangle]
pub unsafe extern "C" fn hudhud_object_get(obj: *mut HudObject, key: *const c_char) -> i64 {
    if obj.is_null() || key.is_null() {
        return 0;
    }
    let magic = *(obj as *const u32);
    if magic != HUD_OBJECT_MAGIC {
        if magic == crate::array::HUD_ARRAY_MAGIC {
            if let Some(k) = key_from(key) {
                if k == "length" || k == "len" {
                    return crate::array::hudhud_array_len(obj as *mut crate::array::HudArray);
                }
            }
        } else if crate::type_ops::is_string(obj as u64) {
            if let Some(k) = key_from(key) {
                if k == "length" || k == "len" {
                    return crate::hudhud_string_len(obj as *const c_char);
                }
            }
        }
        return 0;
    }
    let o = &*obj;
    for i in 0..o.num_fields as usize {
        if keys_match(o.keys[i], key) {
            return o.values[i];
        }
    }
    if !o.overflow.is_null() {
        if let Some(k) = key_from(key) {
            return (*o.overflow).get(k).copied().unwrap_or(0);
        }
    }
    0
}

/// Property existence test: 1 = var, 0 = yok.
///
/// # Safety
/// `obj` must originate from `hudhud_object_new`.
#[no_mangle]
pub unsafe extern "C" fn hudhud_object_has(obj: *mut HudObject, key: *const c_char) -> i64 {
    if obj.is_null() || key.is_null() {
        return 0;
    }
    let o = &*obj;
    for i in 0..o.num_fields as usize {
        if keys_match(o.keys[i], key) {
            return 1;
        }
    }
    if !o.overflow.is_null() {
        if let Some(k) = key_from(key) {
            return (*o.overflow).contains_key(k) as i64;
        }
    }
    0
}

/// Property count.
///
/// # Safety
/// `obj` must originate from `hudhud_object_new`.
#[no_mangle]
pub unsafe extern "C" fn hudhud_object_len(obj: *mut HudObject) -> i64 {
    if obj.is_null() {
        return 0;
    }
    let o = &*obj;
    let extra = if o.overflow.is_null() { 0 } else { (*o.overflow).len() };
    (o.num_fields as usize + extra) as i64
}

/// Free an object.
///
/// # Safety
/// `obj` must originate from `hudhud_object_new` and freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn hudhud_object_free(obj: *mut HudObject) {
    if !obj.is_null() {
        crate::type_ops::unregister_object(obj as usize);
        let o = Box::from_raw(obj);
        if !o.overflow.is_null() {
            drop(Box::from_raw(o.overflow));
        }
        drop(o);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn key(s: &str) -> *const c_char {
        CString::new(s).unwrap().into_raw() as *const c_char
    }

    #[test]
    fn set_get_roundtrip() {
        unsafe {
            let obj = hudhud_object_new();
            let k = key("deger");
            hudhud_object_set(obj, k, 42);
            assert_eq!(hudhud_object_get(obj, k), 42);
            assert_eq!(hudhud_object_get(obj, key("yok")), 0);
            assert_eq!(hudhud_object_has(obj, k), 1);
            assert_eq!(hudhud_object_len(obj), 1);
            hudhud_object_free(obj);
        }
    }

    #[test]
    fn overwrite_and_null_safety() {
        unsafe {
            let obj = hudhud_object_new();
            let k = key("x");
            hudhud_object_set(obj, k, 1);
            hudhud_object_set(obj, k, 9);
            assert_eq!(hudhud_object_get(obj, k), 9);
            assert_eq!(hudhud_object_len(obj), 1);
            // null handle'lar sessizce güvenli
            assert_eq!(hudhud_object_get(std::ptr::null_mut(), k), 0);
            assert_eq!(hudhud_object_len(std::ptr::null_mut()), 0);
            hudhud_object_free(obj);
        }
    }
}

// ── Handle dönüşümleri (i64-handle lane ABI kenarları) ─────────────────

/// Opak handle ↔ i64 kimlik dönüşümü. gccjit gibi C-tipi backend'lerde
/// pointer→int cast kısıtlıdır; handle'lar i64 şeridinde taşınır, ABI
/// kenarında bu helper ile dönüşür. Maliyeti bir makine komutu değildir
/// (identity), tüm backend'lerde aynı semantiği garanti eder.
#[no_mangle]
pub extern "C" fn hudhud_ptr_to_i64(p: *mut core::ffi::c_void) -> i64 {
    p as i64
}

/// i64 → opak handle (hudhud_ptr_to_i64'nun tersi).
#[no_mangle]
pub extern "C" fn hudhud_i64_to_ptr(v: i64) -> *mut core::ffi::c_void {
    v as *mut core::ffi::c_void
}

#[cfg(test)]
mod handle_tests {
    #[test]
    fn ptr_i64_roundtrip() {
        let x: i64 = 0x1234_5678;
        unsafe {
            let p = super::hudhud_i64_to_ptr(x);
            assert_eq!(super::hudhud_ptr_to_i64(p), x);
        }
    }
}

/// Dallanmasız seçim: c != 0 ? a : b. §18 bayrak select zincirleri ve
/// Div/Rem güvenli bölen seçimi için (gccjit gibi select rvalue'su
/// sunmayan backend'ler).
#[no_mangle]
pub extern "C" fn hudhud_select_i64(c: i64, a: i64, b: i64) -> i64 {
    if c != 0 { a } else { b }
}

#[cfg(test)]
mod select_tests {
    #[test]
    fn select_semantics() {
        assert_eq!(super::hudhud_select_i64(1, 10, 20), 10);
        assert_eq!(super::hudhud_select_i64(0, 10, 20), 20);
        assert_eq!(super::hudhud_select_i64(-5, 10, 20), 10);
    }
}

// ── Date/Math builtin helper'ları (benchmark zamanlama/matematik şeridi) ──


#[no_mangle]
pub extern "C" fn hudhud_math_sin(v: f64) -> f64 {
    v.sin()
}

#[no_mangle]
pub extern "C" fn hudhud_math_sqrt(v: f64) -> f64 {
    v.sqrt()
}

#[no_mangle]
pub extern "C" fn hudhud_math_cos(v: f64) -> f64 {
    v.cos()
}

#[no_mangle]
pub extern "C" fn hudhud_math_floor(v: f64) -> f64 { v.floor() }

#[no_mangle]
pub extern "C" fn hudhud_math_abs(v: f64) -> f64 { v.abs() }

#[no_mangle]
pub extern "C" fn hudhud_math_pow(v: f64, e: f64) -> f64 { v.powf(e) }

#[no_mangle]
pub extern "C" fn hudhud_math_min(a: f64, b: f64) -> f64 { if a < b { a } else { b } }

#[no_mangle]
pub extern "C" fn hudhud_math_max(a: f64, b: f64) -> f64 { if a > b { a } else { b } }

// ── Modül-geneli bağlamlar (global let'ler) ──────────────────────────
// Tek HudObject; erişim mevcut ObjectGet/ObjectSet ABI'siyle isim
// anahtarı üzerinden yapılır — backend başına yalnız handle helper'ı.

use std::sync::OnceLock;

static GLOBALS: OnceLock<i64> = OnceLock::new();

/// Modül-global deposunun handle'ı (ilk çağrıda yaratılır; i64 handle).
#[no_mangle]
pub extern "C" fn hudhud_globals() -> i64 {
    *GLOBALS.get_or_init(|| hudhud_object_new() as i64)
}

pub const MAX_GLOBAL_SLOTS: usize = 65536;
/// #[no_mangle]: JIT bu sembolü data-import olarak declare eder ve
/// global erişimi SATIR İÇİ yükler (hudhud_global_get çağrısı ~9M/koşu
/// ödettiydi — game_of_life: her a[i] öncesi global_get(a) çağrısı).
#[no_mangle]
pub static mut GLOBAL_SLOTS: [i64; MAX_GLOBAL_SLOTS] = [0; MAX_GLOBAL_SLOTS];

/// Slot-indeksli global değişken okuma (O(1), allocation ve string lookup yok).
#[no_mangle]
pub extern "C" fn hudhud_global_get(slot: i64) -> i64 {
    let idx = slot as usize;
    if idx < MAX_GLOBAL_SLOTS {
        unsafe { *GLOBAL_SLOTS.get_unchecked(idx) }
    } else {
        0
    }
}

/// Slot-indeksli global değişken yazma (O(1), allocation ve string lookup yok).
#[no_mangle]
pub extern "C" fn hudhud_global_set(slot: i64, value: i64) {
    let idx = slot as usize;
    if idx < MAX_GLOBAL_SLOTS {
        unsafe {
            *GLOBAL_SLOTS.get_unchecked_mut(idx) = value;
        }
    }
}

/// Global slotların raw pointer'ı (JIT doğrudan bellek erişimi için).
#[no_mangle]
pub extern "C" fn hudhud_global_ptr() -> *mut i64 {
    unsafe { GLOBAL_SLOTS.as_mut_ptr() }
}

/// Modül-global slot deposunu sıfırlar.
#[no_mangle]
pub extern "C" fn hudhud_globals_reset() {
    unsafe {
        GLOBAL_SLOTS = [0; MAX_GLOBAL_SLOTS];
    }
}

