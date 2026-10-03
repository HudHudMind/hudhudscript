//! Tests for hudhudscript-native-abi object ABI — inline-slot property
//! storage, null-handle safety, i64 handle conversions, branchless select.

use std::ffi::{c_char, CString};

use hudhudscript_native_abi::{
    hudhud_i64_to_ptr, hudhud_object_free, hudhud_object_get, hudhud_object_has,
    hudhud_object_len, hudhud_object_new, hudhud_object_set, hudhud_ptr_to_i64,
    hudhud_select_i64,
};

fn key(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw() as *const c_char
}

// ── property set/get ─────────────────────────────────────────────

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

// ── handle conversions ───────────────────────────────────────────

#[test]
fn ptr_i64_roundtrip() {
    let x: i64 = 0x1234_5678;
    unsafe {
        let p = hudhud_i64_to_ptr(x);
        assert_eq!(hudhud_ptr_to_i64(p), x);
    }
}

// ── branchless select ────────────────────────────────────────────

#[test]
fn select_semantics() {
    assert_eq!(hudhud_select_i64(1, 10, 20), 10);
    assert_eq!(hudhud_select_i64(0, 10, 20), 20);
    assert_eq!(hudhud_select_i64(-5, 10, 20), 10);
}
