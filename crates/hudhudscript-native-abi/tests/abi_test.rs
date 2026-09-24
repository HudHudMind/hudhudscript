//! C-ABI helper behavior tests (value display/print/error helpers) —
//! driven exactly the way native code will call them.

use hudhudscript_native_abi::{hudhud_print, hudhud_string_free, hudhud_value_display, HudValue};

unsafe fn display_of(v: HudValue) -> Option<String> {
    let p = hudhud_value_display(v);
    if p.is_null() {
        return None;
    }
    let s = std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned();
    hudhud_string_free(p);
    Some(s)
}

#[test]
fn display_int_bool_null() {
    assert_eq!(unsafe { display_of(HudValue::int(5)) }.as_deref(), Some("5"));
    assert_eq!(unsafe { display_of(HudValue::int(-42)) }.as_deref(), Some("-42"));
    assert_eq!(unsafe { display_of(HudValue::int(i64::MIN)) }.as_deref(), Some("-9223372036854775808"));
    assert_eq!(unsafe { display_of(HudValue::bool_(true)) }.as_deref(), Some("true"));
    assert_eq!(unsafe { display_of(HudValue::bool_(false)) }.as_deref(), Some("false"));
    assert_eq!(unsafe { display_of(HudValue::null()) }.as_deref(), Some("null"));
}

#[test]
fn display_unsupported_tags_return_null_pointer() {
    // Float/String biçimleme, motor birebir biçimi bağlanana kadar
    // yoktur: sessiz yanlış çıktı YERİNE açık NULL.
    assert!(unsafe { hudhud_value_display(HudValue::float(1.5)) }.is_null());
    let string_like = HudValue { tag: 5, flags: 0, payload: 0x10 };
    assert!(unsafe { hudhud_value_display(string_like) }.is_null());
}

#[test]
fn print_reports_status() {
    unsafe {
        assert_eq!(hudhud_print(HudValue::int(7)), 0);
        assert_eq!(hudhud_print(HudValue::null()), 0);
        assert_eq!(hudhud_print(HudValue::float(2.5)), 1);
    }
}

#[test]
fn free_accepts_null() {
    unsafe { hudhud_string_free(std::ptr::null_mut()) };
}

#[test]
fn print_int_outputs_value() {
    // stdout'a yazamaz test içinde; davranış: rc=0, panik yok
    let rc = unsafe { hudhudscript_native_abi::hudhud_print_int(42) };
    assert_eq!(rc, 0);
}
