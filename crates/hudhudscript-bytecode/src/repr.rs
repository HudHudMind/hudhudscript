#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReprTag {
    Null = 0,
    Bool = 1,
    Number = 2,       // f64 inline
    Int = 3,          // i64 inline
    Dynamic = 4,      // heap pointer: String, Array, Object, Function, etc.
    InlineString = 5, // short string ≤15 bytes stored directly in payload
}

/// 16-byte compact value representation (Rune-style).
/// Layout: [tag: 4 bits | payload: 124 bits] packed in u128.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Repr(pub u128);

impl Repr {
    // 4-bit tag in the lowest bits (0-3), payload in remaining 124 bits.
    const TAG_MASK: u128 = 0xF; // bits 0-3
    const PAYLOAD_MASK: u128 = !0xF_u128; // bits 4-127

    /// Create a Repr from tag and inline payload (up to 124 bits usable).
    #[inline(always)]
    pub fn new_inline(tag: ReprTag, payload: u64) -> Self {
        let tag_bits = (tag as u8 & 0xF) as u128;
        let payload_bits = ((payload as u128) << 4) & Self::PAYLOAD_MASK;
        Repr(tag_bits | payload_bits)
    }

    /// Get the tag from the lowest 4 bits.
    #[inline(always)]
    pub fn tag(&self) -> ReprTag {
        match (self.0 & Self::TAG_MASK) as u8 {
            0 => ReprTag::Null,
            1 => ReprTag::Bool,
            2 => ReprTag::Number,
            3 => ReprTag::Int,
            4 => ReprTag::Dynamic,
            5 => ReprTag::InlineString,
            _ => panic!("Invalid Repr tag: {}", self.0 & Self::TAG_MASK),
        }
    }

    /// Pack a short string (≤15 bytes) directly into the 124-bit payload.
    /// Layout: [tag:4 | len:4 | bytes...] — little-endian, byte 0 = tag|len.
    #[inline]
    pub fn new_inline_string(s: &str) -> Option<Self> {
        let len = s.len();
        if len > 15 {
            return None;
        }
        let mut payload: u128 = 0;
        payload |= (ReprTag::InlineString as u128) & 0xF;
        payload |= ((len as u128) & 0xF) << 4;
        for (i, &b) in s.as_bytes().iter().enumerate() {
            payload |= (b as u128) << (8 + i * 8);
        }
        Some(Repr(payload))
    }

    /// Extract the inline string as a `&str` borrowing this `Repr`.
    /// SAFETY: valid UTF-8 is guaranteed by `new_inline_string`.
    #[inline]
    pub fn as_inline_string(&self) -> Option<&str> {
        if self.tag() != ReprTag::InlineString {
            return None;
        }
        let len = ((self.0 >> 4) & 0xF) as usize;
        let ptr = self as *const Repr as *const u8;
        let raw = unsafe { std::slice::from_raw_parts(ptr.add(1), len) };
        Some(unsafe { std::str::from_utf8_unchecked(raw) })
    }

    /// Get the 64-bit payload (bits 4-67, right-shifted).
    #[inline(always)]
    pub fn payload_u64(&self) -> u64 {
        ((self.0 & Self::PAYLOAD_MASK) >> 4) as u64
    }

    /// Create a Repr for a heap pointer (Dynamic tag).
    /// Only lower 48 bits of pointer are stored (assumes 48-bit VA space).
    #[inline(always)]
    pub fn new_dynamic(ptr: *const ()) -> Self {
        let addr = ptr as u64;
        debug_assert!(
            addr & 0xFFFF000000000000 == 0,
            "Pointer {:#x} exceeds 48-bit address space",
            addr
        );
        Self::new_inline(ReprTag::Dynamic, addr)
    }

    /// Extract f64 (Number tag) - payload is f64::to_bits().
    /// Int widens to f64 for uniformity with legacy Value::as_number().
    #[inline(always)]
    pub fn as_number(&self) -> Option<f64> {
        match self.tag() {
            ReprTag::Number => Some(f64::from_bits(self.payload_u64())),
            ReprTag::Int => Some(self.payload_u64() as i64 as f64),
            _ => None,
        }
    }

    /// Extract i64 (Int tag) - payload is i64 as u64 (two's complement).
    #[inline(always)]
    pub fn as_int(&self) -> Option<i64> {
        if self.tag() == ReprTag::Int {
            Some(self.payload_u64() as i64)
        } else {
            None
        }
    }

    /// Extract bool (Bool tag) - payload 0=false, non-zero=true.
    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> {
        if self.tag() == ReprTag::Bool {
            Some(self.payload_u64() != 0)
        } else {
            None
        }
    }

    /// Check if null.
    #[inline(always)]
    pub fn is_null(&self) -> bool {
        self.tag() == ReprTag::Null
    }

    /// Extract heap pointer (Dynamic tag).
    #[inline(always)]
    pub fn as_ptr(&self) -> Option<*const ()> {
        if self.tag() == ReprTag::Dynamic {
            Some(self.payload_u64() as *const ())
        } else {
            None
        }
    }

    // ── Constructors ─────────────────────────────────────────────────

    #[inline(always)]
    #[inline(always)]
    pub fn null() -> Self {
        Self::new_inline(ReprTag::Null, 0)
    }

    #[inline(always)]
    pub fn bool_(v: bool) -> Self {
        Self::new_inline(ReprTag::Bool, if v { 1 } else { 0 })
    }

    #[inline(always)]
    pub fn number(v: f64) -> Self {
        Self::new_inline(ReprTag::Number, v.to_bits())
    }

    #[inline(always)]
    pub fn int(v: i64) -> Self {
        Self::new_inline(ReprTag::Int, v as u64)
    }
}

impl std::fmt::Debug for Repr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tag() {
            ReprTag::Null => write!(f, "Repr(Null)"),
            ReprTag::Bool => write!(f, "Repr(Bool({}))", self.as_bool().unwrap()),
            ReprTag::Number => write!(f, "Repr(Number({}))", self.as_number().unwrap()),
            ReprTag::Int => write!(f, "Repr(Int({}))", self.as_int().unwrap()),
            ReprTag::InlineString => write!(f, "Repr(InlineString({:?}))", self.as_inline_string().unwrap()),
            ReprTag::Dynamic => write!(f, "Repr(Dynamic({:p}))", self.as_ptr().unwrap()),
        }
    }
}
