//! Test assertion helper'ları (M4, v0.9.37) — hudunit assert_* yerleşikleri.
//!
//! Başarısızlıkta tanı stderr'e yazılır ve hudhud_throw ile istisna
//! kaldırılır (JIT_EXIT_UNCAUGHT_EXCEPTION yolu). int şeridi (i64) ve
//! assert_approx için f64 şeridi — MathPow ABI kalıbı.

use std::ffi::{c_char, CString};

use crate::type_ops::register_string;

fn throw_assert(msg: String) {
    eprintln!("assertion failed: {msg}");
    if let Ok(c) = CString::new(msg) {
        let p = c.into_raw();
        register_string(p as usize);
        unsafe { crate::exception::hudhud_throw(p as i64) };
    }
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_assert_eq(a: i64, b: i64) {
    if a != b {
        throw_assert(format!("assert_eq: {a} != {b}"));
    }
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_assert_approx(a: f64, b: f64) {
    if (a - b).abs() > 1e-9 {
        throw_assert(format!("assert_approx: {a} !~ {b}"));
    }
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_assert_true(x: i64) {
    if x == 0 {
        throw_assert("assert_true: got false".into());
    }
}

#[no_mangle]
pub unsafe extern "C" fn hudhud_assert_false(x: i64) {
    if x != 0 {
        throw_assert("assert_false: got true".into());
    }
}

#[allow(dead_code)]
fn _unused(_p: *const c_char) {}
