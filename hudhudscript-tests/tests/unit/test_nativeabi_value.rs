//! Tests for hudhudscript-native-abi `HudValue` — the stable C-ABI tagged
//! value: 16-byte layout, tag round-trips, int/float/bool constructors.

use hudhudscript_native_abi::{hudhud_err_overflow, HudTag, HudValue};

// ── C layout ─────────────────────────────────────────────────────

#[test]
fn c_layout_is_16_bytes_8_aligned() {
    assert_eq!(std::mem::size_of::<HudValue>(), 16);
    assert_eq!(std::mem::align_of::<HudValue>(), 8);
}

// ── tag round-trips ──────────────────────────────────────────────

#[test]
fn tag_roundtrip_and_unknown_collapses_to_error() {
    for t in [
        HudTag::Null, HudTag::Bool, HudTag::Int, HudTag::Float, HudTag::Object,
        HudTag::String, HudTag::Array, HudTag::Function, HudTag::Agent,
        HudTag::Error, HudTag::ErrOverflow, HudTag::ErrDivZero,
    ] {
        assert_eq!(HudTag::from_u32(t as u32), Some(t));
    }
    assert_eq!(HudTag::from_u32(999), None);
    let junk = HudValue { tag: 999, flags: 0, payload: 7 };
    assert_eq!(junk.tag(), HudTag::Error);
}

// ── numeric / bool / null constructors ───────────────────────────

#[test]
fn int_roundtrip_including_bounds() {
    for v in [0i64, 1, -1, i64::MIN, i64::MAX, i64::MIN + 1, i64::MAX - 1] {
        assert_eq!(HudValue::int(v).as_i64(), Some(v));
    }
    assert_eq!(HudValue::int(5).as_f64(), None);
    assert_eq!(HudValue::int(5).as_bool(), None);
}

#[test]
fn float_carries_exact_bits() {
    for v in [0.0f64, -0.0, 1.5, f64::INFINITY, f64::NAN] {
        let h = HudValue::float(v);
        let back = h.as_f64().unwrap();
        assert!(back.to_bits() == v.to_bits(), "bits must be exact for {v}");
    }
}

#[test]
fn bool_and_null() {
    assert_eq!(HudValue::bool_(true).as_bool(), Some(true));
    assert_eq!(HudValue::bool_(false).as_bool(), Some(false));
    assert_eq!(HudValue::null().tag(), HudTag::Null);
}

// ── error constructors ───────────────────────────────────────────

#[test]
fn error_constructors() {
    assert_eq!(HudValue::error(HudTag::ErrOverflow).tag(), HudTag::ErrOverflow);
    assert_eq!(hudhud_err_overflow_for_test().tag(), HudTag::ErrOverflow);
}

fn hudhud_err_overflow_for_test() -> HudValue {
    hudhud_err_overflow()
}
