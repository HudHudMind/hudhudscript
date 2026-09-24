//! `HudValue` — the stable C-ABI tagged value (JIT_AOT_ARCHITECTURE
//! §10). 16 bytes: `tag | flags | payload`. The VM's internal Value16
//! stays untouched; conversions happen only at trampoline boundaries.

use std::os::raw::c_uint;

/// Runtime ABI version — rides every artifact and cache key (§11.2).
pub const HUDHUD_RUNTIME_ABI_VERSION: u32 = 1;

/// Value tags. Numeric section mirrors §10 exactly; the trailing error
/// codes are runtime-exit payloads (§18 lanes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum HudTag {
    Null = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    Object = 4,
    String = 5,
    Array = 6,
    Function = 7,
    Agent = 8,
    BigInt = 9,
    /// Runtime error sentinel — never a script-visible value; used by
    /// error helpers and JIT exit statuses.
    Error = 32,
    /// i64 signed overflow (§18).
    ErrOverflow = 33,
    /// Integer division or remainder by zero (§18).
    ErrDivZero = 34,
}

impl HudTag {
    pub fn from_u32(v: u32) -> Option<HudTag> {
        Some(match v {
            0 => HudTag::Null,
            1 => HudTag::Bool,
            2 => HudTag::Int,
            3 => HudTag::Float,
            4 => HudTag::Object,
            5 => HudTag::String,
            6 => HudTag::Array,
            7 => HudTag::Function,
            8 => HudTag::Agent,
            9 => HudTag::BigInt,
            32 => HudTag::Error,
            33 => HudTag::ErrOverflow,
            34 => HudTag::ErrDivZero,
            _ => return None,
        })
    }
}

/// Stable tagged value. `flags` is reserved (0 today).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct HudValue {
    pub tag: c_uint,
    pub flags: c_uint,
    pub payload: u64,
}

impl HudValue {
    pub const fn null() -> HudValue {
        HudValue { tag: HudTag::Null as c_uint, flags: 0, payload: 0 }
    }

    pub const fn bool_(v: bool) -> HudValue {
        HudValue { tag: HudTag::Bool as c_uint, flags: 0, payload: v as u64 }
    }

    pub const fn int(v: i64) -> HudValue {
        HudValue { tag: HudTag::Int as c_uint, flags: 0, payload: v as u64 }
    }

    /// Float payload carries the f64 bit pattern (payload alone — no
    /// NaN-boxing, §10).
    pub const fn float(v: f64) -> HudValue {
        HudValue { tag: HudTag::Float as c_uint, flags: 0, payload: v.to_bits() }
    }

    pub(crate) const fn error(tag: HudTag) -> HudValue {
        HudValue { tag: tag as c_uint, flags: 0, payload: 0 }
    }

    pub fn tag(&self) -> HudTag {
        // Değerler yalnız bu crate'in kurucularından ya da doğrulanmış
        // ABI girişlerinden gelir; bilinmeyen etiket Null'e çökmez —
        // ayrı bir Error'e çevrilir (sessiz yanlış değer YOK).
        HudTag::from_u32(self.tag).unwrap_or(HudTag::Error)
    }

    pub fn as_i64(&self) -> Option<i64> {
        (self.tag() == HudTag::Int).then(|| self.payload as i64)
    }

    pub fn as_f64(&self) -> Option<f64> {
        (self.tag() == HudTag::Float).then(|| f64::from_bits(self.payload))
    }

    pub fn as_bool(&self) -> Option<bool> {
        (self.tag() == HudTag::Bool).then(|| self.payload != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_layout_is_16_bytes_8_aligned() {
        assert_eq!(std::mem::size_of::<HudValue>(), 16);
        assert_eq!(std::mem::align_of::<HudValue>(), 8);
    }

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

    #[test]
    fn error_constructors() {
        assert_eq!(HudValue::error(HudTag::ErrOverflow).tag(), HudTag::ErrOverflow);
        assert_eq!(hudhud_err_overflow_for_test().tag(), HudTag::ErrOverflow);
    }

    fn hudhud_err_overflow_for_test() -> HudValue {
        crate::hudhud_err_overflow()
    }
}
