use crate::{
    ClassData, DataData, DynamicData, DynamicKind, DynamicObject, FunctionData, GeneratorState16,
    InstanceData, PromiseState16, ReprTag, ResourceRef, ToolRef, Value16,
};
use parking_lot::Mutex;
use std::sync::Arc;

impl Value16 {
    #[inline]
    pub fn type_name_str(&self) -> &'static str {
        match self.0.tag() {
            ReprTag::Null => "null",
            ReprTag::Bool => "boolean",
            ReprTag::Number => "number",
            ReprTag::Int => "number",
            ReprTag::InlineString => "string",
            ReprTag::Dynamic => {
                if self.as_str().is_some() {
                    "string"
                } else if self.as_array().is_some() {
                    "array"
                } else if self.as_set().is_some() {
                    "set"
                } else if self.as_map_pairs().is_some() {
                    "map"
                } else if self.as_object().is_some() {
                    "object"
                } else if self.as_instance_data().is_some() {
                    "instance"
                } else if self.as_function_data().is_some() {
                    "function"
                } else if self.as_generator_state().is_some() {
                    "generator"
                } else if self.as_promise_state().is_some() {
                    "promise"
                } else if self.as_class_data().is_some() {
                    "class"
                } else {
                    "object"
                }
            }
        }
    }

    #[inline]
    pub fn display_string(&self) -> String {
        match self.0.tag() {
            ReprTag::Null => "null".to_string(),
            ReprTag::Bool => self.as_bool().unwrap().to_string(),
            ReprTag::Number => self.as_number().unwrap().to_string(),
            ReprTag::Int => self.as_int().unwrap().to_string(),
            ReprTag::InlineString => self.as_str().unwrap().to_string(),
            ReprTag::Dynamic => {
                if self.as_string().is_some() {
                    self.as_string().unwrap()
                } else {
                    "<dynamic>".to_string()
                }
            }
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    #[inline(always)]
    pub fn is_truthy(&self) -> bool {
        match self.0.tag() {
            ReprTag::Null => false,
            ReprTag::Bool => self.0.payload_u64() != 0,
            ReprTag::Number => {
                let n = f64::from_bits(self.0.payload_u64());
                n != 0.0 && !n.is_nan()
            }
            ReprTag::Int => self.0.payload_u64() as i64 != 0,
            ReprTag::InlineString => {
                // Length is in bits 4-7 of the 128-bit repr; check via payload vs tag
                (self.0.payload_u64() & 0xF0) != 0 // has non-zero length
            },
            ReprTag::Dynamic => {
                if let Some(s) = self.as_str() {
                    !s.is_empty()
                } else {
                    true // arrays, objects, functions are truthy
                }
            }
        }
    }

    pub fn values_equal(&self, other: &Self) -> bool {
        // Fast path: any string (inline or dynamic) compares by content.
        if let (Some(a), Some(b)) = (self.as_str(), other.as_str()) {
            return a == b;
        }
        // Cross-type numeric equality: Int and Number compare by value
        // (mirrors JavaScript/Lua semantics where 5 == 5.0 is true).
        if self.0.tag() != other.0.tag() {
            match (self.0.tag(), other.0.tag()) {
                (ReprTag::Int, ReprTag::Number) => {
                    return (self.0.payload_u64() as i64 as f64) == f64::from_bits(other.0.payload_u64());
                }
                (ReprTag::Number, ReprTag::Int) => {
                    return f64::from_bits(self.0.payload_u64()) == (other.0.payload_u64() as i64 as f64);
                }
                _ => return false,
            }
        }
        match self.0.tag() {
            ReprTag::Null => true,
            ReprTag::Bool => self.0.payload_u64() == other.0.payload_u64(),
            ReprTag::Number => f64::from_bits(self.0.payload_u64()) == f64::from_bits(other.0.payload_u64()),
            ReprTag::Int => self.0.payload_u64() == other.0.payload_u64(),
            ReprTag::InlineString => unreachable!(), // handled by as_str fast path above
            ReprTag::Dynamic => {
                if let (Some(a), Some(b)) = (self.as_array(), other.as_array()) {
                    if a.len() != b.len() {
                        return false;
                    }
                    return a.iter().zip(b.iter()).all(|(x, y)| x.values_equal(y));
                }
                if let (Some(a), Some(b)) = (self.as_set(), other.as_set()) {
                    if a.len() != b.len() {
                        return false;
                    }
                    return a.iter().all(|x| b.iter().any(|y| x.values_equal(y)));
                }
                if let (Some(a), Some(b)) = (self.as_map_pairs(), other.as_map_pairs()) {
                    if a.len() != b.len() {
                        return false;
                    }
                    return a.iter().all(|(xk, xv)| {
                        b.iter()
                            .any(|(yk, yv)| xk.values_equal(yk) && xv.values_equal(yv))
                    });
                }
                if let (Some(a), Some(b)) = (self.as_object(), other.as_object()) {
                    if a.len() != b.len() {
                        return false;
                    }
                    return a
                        .iter()
                        .all(|(k, v)| b.get(k).map(|bv| v.values_equal(bv)).unwrap_or(false));
                }
                false
            }
        }
    }

    // Runtime-specific extractors (default None, override in runtime-specific modules)
    pub fn as_function_params(&self) -> Option<Vec<String>> {
        None
    }
    pub fn as_instance_fields(&self) -> Option<&std::collections::HashMap<String, Value16>> {
        None
    }
    pub fn as_class_name(&self) -> Option<&str> {
        None
    }
    pub fn as_class_methods(&self) -> Option<&std::collections::HashMap<String, Value16>> {
        None
    }
    pub fn as_class_fields(&self) -> Option<&std::collections::HashMap<String, Value16>> {
        None
    }
    pub fn as_class_parent(&self) -> Option<&Value16> {
        None
    }
    pub fn as_instance_class(&self) -> Option<&Value16> {
        None
    }
}
