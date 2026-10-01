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

    #[doc(hidden)]
    pub const fn error(tag: HudTag) -> HudValue {
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
