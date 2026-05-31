use crate::{
    ClassData, DataData, DynamicData, DynamicKind, DynamicObject, FunctionData, GeneratorState16,
    InstanceData, PromiseState16, Repr, ReprTag, ResourceRef, ToolRef,
};
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};

/// Global cache for single-byte ASCII strings (0–127). Populated on first
/// call to `Value16::string_ascii`. Eliminates heap allocation for the
/// common case of indexing an ASCII string one character at a time.
static ASCII_CHAR_CACHE: OnceLock<[Value16; 128]> = OnceLock::new();

#[derive(Clone, Copy, Hash)]
pub struct Value16(pub Repr);

impl From<&Value16> for Value16 {
    fn from(v: &Value16) -> Self {
        *v
    }
}

impl Value16 {
    // ── Inline Constructors ──────────────────────────────────────────

    #[inline(always)]
    pub fn null() -> Self {
        Value16(Repr::null())
    }

    #[inline(always)]
    pub fn bool_(v: bool) -> Self {
        Value16(Repr::bool_(v))
    }

    #[inline(always)]
    pub fn boolean(v: bool) -> Self {
        Self::bool_(v)
    }

    #[inline(always)]
    pub fn number(v: f64) -> Self {
        Value16(Repr::number(v))
    }

    #[inline(always)]
    pub fn int(v: i64) -> Self {
        Value16(Repr::int(v))
    }

    /// Create a String value.  Short strings (≤15 bytes) are stored inline
    /// in the 124-bit Repr payload — zero heap allocation.
    #[inline]
    pub fn string(s: impl Into<String>) -> Self {
        let s: String = s.into();
        if s.len() <= 15 {
            if let Some(repr) = Repr::new_inline_string(&s) {
                return Value16(repr);
            }
        }
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::String,
            data: DynamicData::String(s),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Return a cached single-byte ASCII string (0–127) without heap allocation.
    /// Populates the cache on first call; all subsequent calls are O(1) copies.
    #[inline(always)]
    pub fn string_ascii(b: u8) -> Self {
        let cache = ASCII_CHAR_CACHE.get_or_init(|| {
            let mut arr = [Value16::null(); 128];
            for i in 0..128u8 {
                arr[i as usize] = Value16::string((i as char).to_string());
            }
            arr
        });
        cache[b as usize]
    }

    // ── Type Tests ───────────────────────────────────────────────────

    #[inline(always)]
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    #[inline(always)]
    pub fn is_number(&self) -> bool {
        self.0.tag() == ReprTag::Number
    }

    #[inline(always)]
    pub fn is_int(&self) -> bool {
        self.0.tag() == ReprTag::Int
    }

    /// Raw i64 payload WITHOUT tag check. Caller MUST verify is_int() first.
    #[inline(always)]
    pub fn as_int_unchecked(&self) -> i64 {
        self.0.payload_u64() as i64
    }

    /// Raw f64 payload WITHOUT tag check. Caller MUST verify is_number() first.
    #[inline(always)]
    pub fn as_number_unchecked(&self) -> f64 {
        f64::from_bits(self.0.payload_u64())
    }

    /// Return (tag, raw_payload) in a single call.
    /// Caller matches on tag to decide Int/Number/other path.
    #[inline(always)]
    pub fn split_tag(&self) -> (ReprTag, u64) {
        (self.0.tag(), self.0.payload_u64())
    }

    /// Fast numeric extraction: returns Some(f64) for Number or Int tags
    /// with a SINGLE split_tag call avoiding double tag check.
    /// Callgrind shows this as one of the hottest paths in arithmetic
    /// benchmarks (Ir 3B+ per run).
    #[inline(always)]
    pub fn as_number_fast(&self) -> Option<f64> {
        let (tag, payload) = self.split_tag();
        match tag {
            ReprTag::Number => Some(f64::from_bits(payload)),
            ReprTag::Int => Some(payload as i64 as f64),
            _ => None,
        }
    }

    /// Fast integer extraction with single split.
    #[inline(always)]
    pub fn as_int_fast(&self) -> Option<i64> {
        let (tag, payload) = self.split_tag();
        match tag {
            ReprTag::Int => Some(payload as i64),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_bool(&self) -> bool {
        self.0.tag() == ReprTag::Bool
    }

    #[inline(always)]
    pub fn is_dynamic(&self) -> bool {
        self.0.tag() == ReprTag::Dynamic
    }

    // ── Extractors ──────────────────────────────────────────────────

    #[inline(always)]
    pub fn as_number(&self) -> Option<f64> {
        self.0.as_number()
    }

    #[inline(always)]
    pub fn as_int(&self) -> Option<i64> {
        self.0.as_int()
    }

    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    /// Create an Array value (heap allocated via DynamicObject).
    #[inline]
    pub fn array(items: Vec<Value16>) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Array,
            data: DynamicData::Array(items),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create an Object value (heap allocated via DynamicObject).
    #[inline]
    pub fn object(map: std::collections::HashMap<String, Value16>) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Object,
            data: DynamicData::Object(map),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Function value (heap allocated via DynamicObject).
    #[inline]
    pub fn function(func: FunctionData) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Function,
            data: DynamicData::Function(func),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create an Instance value (heap allocated via DynamicObject).
    #[inline]
    pub fn instance(inst: InstanceData) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Instance,
            data: DynamicData::Instance(inst),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Promise value (heap allocated via DynamicObject).
    #[inline]
    pub fn promise(p: PromiseState16) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Promise,
            data: DynamicData::Promise(p),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Class value (heap allocated via DynamicObject).
    #[inline]
    pub fn class(c: ClassData) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Class,
            data: DynamicData::Class(c),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Data value (heap allocated via DynamicObject).
    #[inline]
    pub fn data(d: DataData) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Data,
            data: DynamicData::Data(d),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Set value (heap allocated via DynamicObject).
    #[inline]
    pub fn set(items: Vec<Value16>) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Set,
            data: DynamicData::Set(items),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Map value (heap allocated via DynamicObject).
    #[inline]
    pub fn map(pairs: Vec<(Value16, Value16)>) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Map,
            data: DynamicData::Map(pairs),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Generator value (heap allocated via DynamicObject).
    #[inline]
    pub fn generator(state: Arc<Mutex<GeneratorState16>>) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Generator,
            data: DynamicData::Generator(state),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Tool value (heap allocated via DynamicObject).
    #[inline]
    pub fn tool(t: ToolRef) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Tool,
            data: DynamicData::Tool(Box::new(t)),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Resource value (heap allocated via DynamicObject).
    #[inline]
    pub fn resource(r: ResourceRef) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Resource,
            data: DynamicData::Resource(Box::new(r)),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create an Option value (heap allocated via DynamicObject).
    #[inline]
    pub fn option(v: Option<Value16>) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Option,
            data: DynamicData::Option(v.map(Box::new)),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    /// Create a Result value (heap allocated via DynamicObject).
    #[inline]
    pub fn result(r: Result<Value16, String>) -> Self {
        let obj = Box::new(DynamicObject {
            kind: DynamicKind::Result,
            data: DynamicData::Result(r.map(Box::new)),
        });
        Value16(Repr::new_dynamic(Box::into_raw(obj) as *const ()))
    }

    #[inline(always)]
    pub fn as_string(&self) -> Option<String> {
        if let Some(s) = self.0.as_inline_string() {
            return Some(s.to_string());
        }
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::String) {
                if let DynamicData::String(s) = &obj.data {
                    return Some(s.clone());
                }
            }
        }
        None
    }
}
