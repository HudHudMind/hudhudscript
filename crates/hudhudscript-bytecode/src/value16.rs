use crate::{
    gc, ClassData, DataData, DynamicData, DynamicKind, DynamicObject, FunctionData,
    GeneratorState16, InstanceData, ObjMap, PromiseState16, Repr, ReprTag, ResourceRef, ToolRef,
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
        gc::alloc(DynamicKind::String, DynamicData::String(s))
    }

    /// Create a String value from a &str WITHOUT intermediate heap allocation.
    /// Short strings (≤15 bytes) are stored inline directly from the slice.
    /// Long strings fall through to GC heap (single allocation, no double-copy).
    /// P0.3 fix: eliminates `s.into()` allocation for hot-path string methods.
    #[inline]
    pub fn string_from_str(s: &str) -> Self {
        if s.len() <= 15 {
            if let Some(repr) = Repr::new_inline_string(s) {
                return Value16(repr);
            }
        }
        gc::alloc(DynamicKind::String, DynamicData::String(s.to_string()))
    }

    /// Return a cached single-byte ASCII string (0–127) without heap allocation.
    /// Populates the cache on first call; all subsequent calls are O(1) copies.
    #[inline(always)]
    pub fn string_ascii(b: u8) -> Self {
        let cache = ASCII_CHAR_CACHE.get_or_init(|| {
            let mut arr = [Value16::null(); 128];
            for i in 0..128u8 {
                arr[i as usize] = Value16::string_from_str(&(i as char).to_string());
            }
            arr
        });
        cache[b as usize]
    }

    /// Create a BigInt value. If the value fits in i64, store as Int (fast path).
    /// ISSUE-4: Allocate BigInt on GC heap WITHOUT demotion check.
    /// Use ONLY when the result is known to be too large for i64
    /// (e.g., BigInt×BigInt, BigInt+BigInt). Saves 18M to_i64 calls
    /// in fib_iterative/fib_memo.
    #[inline]
    pub fn bigint_no_demote(v: num_bigint::BigInt) -> Self {
        gc::alloc(DynamicKind::BigInt, DynamicData::BigInt(v))
    }

    /// Allocate BigInt with demotion check (small values → Int).
    #[inline]
    pub fn bigint(v: num_bigint::BigInt) -> Self {
        use num_traits::ToPrimitive;
        if let Some(i) = v.to_i64() {
            return Value16::int(i);
        }
        gc::alloc(DynamicKind::BigInt, DynamicData::BigInt(v))
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

    /// True if the value is a heap-allocated Function.
    #[inline(always)]
    pub fn is_function(&self) -> bool {
        if self.0.tag() != ReprTag::Dynamic {
            return false;
        }
        if let Some(ptr) = self.0.as_ptr() {
            let obj = unsafe { &*(ptr as *const crate::DynamicObject) };
            matches!(obj.kind, crate::DynamicKind::Function)
        } else {
            false
        }
    }

    /// Return the raw pointer identity of this function value (O(1)).
    /// Returns 0 for non-function values. Used by Call Inline Cache
    /// to detect cache hits without type/arity checks.
    #[inline(always)]
    pub fn get_func_ptr(&self) -> usize {
        if self.0.tag() != ReprTag::Dynamic {
            return 0;
        }
        self.0.as_ptr().map(|p| p as usize).unwrap_or(0)
    }

    // ── Extractors ──────────────────────────────────────────────────

    #[inline(always)]
    pub fn as_number(&self) -> Option<f64> {
        if let Some(b) = self.as_bigint() {
            use num_traits::ToPrimitive;
            return b.to_f64();
        }
        self.0.as_number()
    }

    #[inline(always)]
    pub fn as_int(&self) -> Option<i64> {
        if let Some(b) = self.as_bigint() {
            use num_traits::ToPrimitive;
            return b.to_i64();
        }
        self.0.as_int()
    }

    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    /// Create an Array value (heap allocated via DynamicObject).
    #[inline]
    pub fn array(items: Vec<Value16>) -> Self {
        gc::alloc(DynamicKind::Array, DynamicData::Array(items))
    }

    /// Create an Object value (heap allocated via DynamicObject).
    /// Accepts any iterable of `(K, Value16)` pairs where `K: Into<SymId>`
    /// (both `String` and `SymId` work) and materialises an `ObjMap` internally.
    #[inline]
    pub fn object<I, K>(items: I) -> Self
    where
        I: IntoIterator<Item = (K, Value16)>,
        K: Into<crate::sym::SymId>,
    {
        gc::alloc(DynamicKind::Object, DynamicData::Object(
            items.into_iter().map(|(k, v)| (k.into(), v)).collect::<ObjMap>()
        ))
    }

    /// Create a Function value (heap allocated via DynamicObject).
    #[inline]
    pub fn function(func: FunctionData) -> Self {
        gc::alloc(DynamicKind::Function, DynamicData::Function(func))
    }

    /// Create an Instance value (heap allocated via DynamicObject).
    #[inline]
    pub fn instance(inst: InstanceData) -> Self {
        gc::alloc(DynamicKind::Instance, DynamicData::Instance(inst))
    }

    /// Create a Promise value (heap allocated via DynamicObject).
    #[inline]
    pub fn promise(p: PromiseState16) -> Self {
        gc::alloc(DynamicKind::Promise, DynamicData::Promise(p))
    }

    /// Create a Class value (heap allocated via DynamicObject).
    #[inline]
    pub fn class(c: ClassData) -> Self {
        gc::alloc(DynamicKind::Class, DynamicData::Class(c))
    }

    /// Create a Data value (heap allocated via DynamicObject).
    #[inline]
    pub fn data(d: DataData) -> Self {
        gc::alloc(DynamicKind::Data, DynamicData::Data(d))
    }

    /// Create a Set value (heap allocated via DynamicObject).
    #[inline]
    pub fn set(items: Vec<Value16>) -> Self {
        gc::alloc(DynamicKind::Set, DynamicData::Set(items))
    }

    /// Create a Map value (heap allocated via DynamicObject).
    #[inline]
    pub fn map(pairs: Vec<(Value16, Value16)>) -> Self {
        gc::alloc(DynamicKind::Map, DynamicData::Map(pairs))
    }

    /// Create a Generator value (heap allocated via DynamicObject).
    #[inline]
    pub fn generator(state: Arc<Mutex<GeneratorState16>>) -> Self {
        gc::alloc(DynamicKind::Generator, DynamicData::Generator(state))
    }

    /// Create a Tool value (heap allocated via DynamicObject).
    #[inline]
    pub fn tool(t: ToolRef) -> Self {
        gc::alloc(DynamicKind::Tool, DynamicData::Tool(Box::new(t)))
    }

    /// Create a Resource value (heap allocated via DynamicObject).
    #[inline]
    pub fn resource(r: ResourceRef) -> Self {
        gc::alloc(DynamicKind::Resource, DynamicData::Resource(Box::new(r)))
    }

    /// Create an Option value (heap allocated via DynamicObject).
    #[inline]
    pub fn option(v: Option<Value16>) -> Self {
        gc::alloc(DynamicKind::Option, DynamicData::Option(v.map(Box::new)))
    }

    /// Create a Result value (heap allocated via DynamicObject).
    #[inline]
    pub fn result(r: Result<Value16, String>) -> Self {
        gc::alloc(DynamicKind::Result, DynamicData::Result(r.map(Box::new)))
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
