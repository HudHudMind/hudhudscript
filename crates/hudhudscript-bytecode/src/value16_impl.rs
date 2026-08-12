use crate::{
    ClassData, DataData, DynamicData, DynamicKind, DynamicObject, FunctionData, GeneratorState16,
    InstanceData, ObjMap, PromiseState16, ReprTag, ResourceRef, ToolRef, Value16,
};
use parking_lot::Mutex;
use std::sync::Arc;

impl PartialEq for Value16 {
    fn eq(&self, other: &Self) -> bool {
        self.values_equal(other)
    }
}

impl Eq for Value16 {}

impl Default for Value16 {
    fn default() -> Self {
        Value16::null()
    }
}

impl Value16 {
    #[inline(always)]
    pub fn as_str(&self) -> Option<&str> {
        if let Some(s) = self.0.as_inline_string() {
            return Some(s);
        }
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if crate::dynamic::is_string_kind(obj.kind) {
                if let DynamicData::String(s) = &obj.data {
                    return Some(s.as_str());
                }
            }
        }
        None
    }

    /// Unchecked string access — caller guarantees String type.
    /// For hot string benchmarks (palindrome, strcat, strrev).
    #[inline(always)]
    pub fn as_str_unchecked(&self) -> &str {
        if let Some(s) = self.0.as_inline_string() {
            return s;
        }
        debug_assert!(self.0.tag() == ReprTag::Dynamic);
        let ptr = self.0.as_ptr().unwrap();
        let obj = unsafe { &*(ptr as *const DynamicObject) };
        debug_assert!(crate::dynamic::is_string_kind(obj.kind));
        if let DynamicData::String(ref s) = obj.data {
            s.as_str()
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    #[inline(always)]
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if self.0.tag() == ReprTag::InlineString {
            return None; // inline strings are immutable
        }
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if crate::dynamic::is_string_kind(obj.kind) {
                if let DynamicData::String(ref mut s) = obj.data {
                    return Some(s);
                }
            }
        }
        None
    }

    /// P4: downgrade StringAscii→String after non-ASCII append.
    #[inline]
    pub fn downgrade_string_ascii(&mut self) {
        if self.0.tag() == ReprTag::Dynamic {
            if let Some(ptr) = self.0.as_ptr() {
                let obj = unsafe { &mut *(ptr as *mut DynamicObject) };
                if obj.kind == DynamicKind::StringAscii {
                    obj.kind = DynamicKind::String;
                }
            }
        }
    }

    /// P4: check if this is a DynamicObject with StringAscii kind.
    #[inline]
    pub fn is_dynamic_string_ascii(&self) -> bool {
        if self.0.tag() == ReprTag::Dynamic {
            if let Some(ptr) = self.0.as_ptr() {
                let obj = unsafe { &*(ptr as *const DynamicObject) };
                return obj.kind == DynamicKind::StringAscii;
            }
        }
        false
    }

    /// KMP1: character-length in O(1) for ASCII strings, O(n) for Unicode.
    /// Inline strings and StringAscii: byte_len == char_len, use s.len().
    /// General String: fall back to chars().count().
    #[inline]
    pub fn str_char_len(&self) -> Option<usize> {
        // Inline string: short, byte len == char count.
        if let Some(s) = self.0.as_inline_string() {
            return Some(s.len());
        }
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if let DynamicData::String(s) = &obj.data {
                if obj.kind == DynamicKind::StringAscii {
                    return Some(s.len());  // O(1): byte len == char len
                }
                if obj.kind == DynamicKind::String {
                    return Some(s.len());  // O(1): byte len (existing semantics)
                }
            }
        }
        None
    }

    #[inline(always)]
    pub fn as_array(&self) -> Option<&Vec<Value16>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Array) {
                if let DynamicData::Array(a) = &obj.data {
                    return Some(a);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value16>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Array) {
                if let DynamicData::Array(ref mut a) = obj.data {
                    return Some(a);
                }
            }
        }
        None
    }

    /// Unchecked array access — caller must guarantee is_dynamic+Array.
    /// For hot matrix/vector benchmarks where type is known from context.
    #[inline(always)]
    pub fn as_array_unchecked(&self) -> &Vec<Value16> {
        debug_assert!(self.0.tag() == ReprTag::Dynamic);
        let obj = unsafe { &*(self.0.as_ptr().unwrap() as *const DynamicObject) };
        debug_assert!(matches!(obj.kind, DynamicKind::Array));
        if let DynamicData::Array(ref a) = obj.data {
            a
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    /// Unchecked mutable array access.
    #[inline(always)]
    pub fn as_array_mut_unchecked(&mut self) -> &mut Vec<Value16> {
        debug_assert!(self.0.tag() == ReprTag::Dynamic);
        let obj = unsafe { &mut *(self.0.as_ptr().unwrap() as *mut DynamicObject) };
        debug_assert!(matches!(obj.kind, DynamicKind::Array));
        if let DynamicData::Array(ref mut a) = obj.data {
            a
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    #[inline]
    pub fn as_object_mut_unchecked(&mut self) -> &mut ObjMap {
        debug_assert!(self.0.tag() == ReprTag::Dynamic);
        let obj = unsafe { &mut *(self.0.as_ptr().unwrap() as *mut DynamicObject) };
        debug_assert!(matches!(obj.kind, DynamicKind::Object));
        if let DynamicData::Object(ref mut m) = obj.data {
            m
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    /// Fast combined array element access: type check + bounds check + clone.
    /// Returns None if not an array or out of bounds.
    #[inline(always)]
    pub fn array_get(&self, idx: usize) -> Option<&Value16> {
        if self.0.tag() == ReprTag::Dynamic {
            let obj = unsafe { &*(self.0.as_ptr()? as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Array) {
                if let DynamicData::Array(ref a) = obj.data {
                    return a.get(idx);
                }
            }
        }
        None
    }

    /// Fast combined array mutation: type check + auto-extend + write.
    #[inline]
    pub fn array_set(&mut self, idx: usize, val: Value16) -> bool {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = match self.0.as_ptr() {
                Some(p) => p,
                None => return false,
            };
            let obj = unsafe { &mut *(ptr as *mut DynamicObject) };
            if matches!(obj.kind, DynamicKind::Array) {
                if let DynamicData::Array(ref mut a) = obj.data {
                    if idx >= a.len() {
                        a.resize(idx + 1, Value16::null());
                    }
                    a[idx] = val;
                    return true;
                }
            }
        }
        false
    }

    #[inline(always)]
    pub fn as_object(&self) -> Option<&ObjMap> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Object) {
                if let DynamicData::Object(o) = &obj.data {
                    return Some(o);
                }
            }
        }
        None
    }

    pub fn as_object_mut(&mut self) -> Option<&mut ObjMap> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Object) {
                if let DynamicData::Object(ref mut o) = obj.data {
                    return Some(o);
                }
            }
        }
        None
    }

    pub fn as_set_mut(&mut self) -> Option<&mut Vec<Value16>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Set) {
                if let DynamicData::Set(ref mut v) = obj.data { return Some(v); }
            }
        }
        None
    }

    pub fn as_map_mut(&mut self) -> Option<&mut Vec<(Value16, Value16)>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Map) {
                if let DynamicData::Map(ref mut v) = obj.data { return Some(v); }
            }
        }
        None
    }

    pub fn as_data_mut(&mut self) -> Option<&mut DataData> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Data) {
                if let DynamicData::Data(ref mut d) = obj.data { return Some(d); }
            }
        }
        None
    }

    pub fn as_instance_mut(&mut self) -> Option<&mut InstanceData> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Instance) {
                if let DynamicData::Instance(ref mut i) = obj.data { return Some(i); }
            }
        }
        None
    }

    pub fn as_result_mut(&mut self) -> Option<&mut Result<Box<Value16>, String>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Result) {
                if let DynamicData::Result(ref mut r) = obj.data { return Some(r); }
            }
        }
        None
    }

    pub fn as_option_mut(&mut self) -> Option<&mut Option<Box<Value16>>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()? as *mut DynamicObject;
            let obj = unsafe { &mut *ptr };
            if matches!(obj.kind, DynamicKind::Option) {
                if let DynamicData::Option(ref mut o) = obj.data { return Some(o); }
            }
        }
        None
    }

    #[inline(always)]
    pub fn as_function_data(&self) -> Option<&FunctionData> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Function) {
                if let DynamicData::Function(f) = &obj.data {
                    return Some(f);
                }
            }
        }
        None
    }
    /// Raw pointer to the FunctionData on the GC heap. Stable across GC.
    pub fn as_function_data_ptr(&self) -> Option<*const FunctionData> {
        self.as_function_data().map(|fd| fd as *const FunctionData)
    }

    #[inline]
    pub fn as_instance_data(&self) -> Option<&InstanceData> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Instance) {
                if let DynamicData::Instance(i) = &obj.data {
                    return Some(i);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_promise_state(&self) -> Option<&PromiseState16> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Promise) {
                if let DynamicData::Promise(p) = &obj.data {
                    return Some(p);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_class_data(&self) -> Option<&ClassData> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Class) {
                if let DynamicData::Class(c) = &obj.data {
                    return Some(c);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_data_data(&self) -> Option<&DataData> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Data) {
                if let DynamicData::Data(d) = &obj.data {
                    return Some(d);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_set(&self) -> Option<&Vec<Value16>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Set) {
                if let DynamicData::Set(s) = &obj.data {
                    return Some(s);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_map_pairs(&self) -> Option<&Vec<(Value16, Value16)>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Map) {
                if let DynamicData::Map(m) = &obj.data {
                    return Some(m);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_bigint(&self) -> Option<&num_bigint::BigInt> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::BigInt) {
                if let DynamicData::BigInt(b) = &obj.data {
                    return Some(b);
                }
            }
        }
        None
    }

    #[inline(always)]
    pub fn as_bigint_unchecked(&self) -> &num_bigint::BigInt {
        let ptr = self.0.as_ptr().unwrap();
        let obj = unsafe { &*(ptr as *const DynamicObject) };
        if let DynamicData::BigInt(ref b) = obj.data { b } else { unreachable!() }
    }

    #[inline]
    pub fn to_bigint_value(&self) -> Option<num_bigint::BigInt> {
        if let Some(i) = self.as_int() {
            return Some(num_bigint::BigInt::from(i));
        }
        self.as_bigint().cloned()
    }

    #[inline]
    pub fn as_generator_state(&self) -> Option<&Arc<Mutex<GeneratorState16>>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Generator) {
                if let DynamicData::Generator(g) = &obj.data {
                    return Some(g);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_tool_ref(&self) -> Option<&ToolRef> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Tool) {
                if let DynamicData::Tool(t) = &obj.data {
                    return Some(t);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_resource_ref(&self) -> Option<&ResourceRef> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Resource) {
                if let DynamicData::Resource(r) = &obj.data {
                    return Some(r);
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_option(&self) -> Option<Option<&Value16>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Option) {
                if let DynamicData::Option(o) = &obj.data {
                    return Some(o.as_ref().map(|b| b.as_ref()));
                }
            }
        }
        None
    }

    #[inline]
    pub fn as_result(&self) -> Option<Result<&Value16, &String>> {
        if self.0.tag() == ReprTag::Dynamic {
            let ptr = self.0.as_ptr()?;
            let obj = unsafe { &*(ptr as *const DynamicObject) };
            if matches!(obj.kind, DynamicKind::Result) {
                if let DynamicData::Result(r) = &obj.data {
                    return Some(r.as_ref().map(|b| b.as_ref()));
                }
            }
        }
        None
    }
}
