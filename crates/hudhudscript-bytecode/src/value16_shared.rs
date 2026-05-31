use crate::{ReprTag, Value16};

impl std::fmt::Debug for Value16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value16({:?})", self.0)
    }
}

impl std::fmt::Display for Value16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.tag() {
            ReprTag::Null => write!(f, "null"),
            ReprTag::Bool => write!(f, "{}", self.as_bool().unwrap()),
            ReprTag::Number => write!(f, "{}", self.as_number().unwrap()),
            ReprTag::Int => write!(f, "{}", self.as_int().unwrap()),
            ReprTag::InlineString => write!(f, "{}", self.as_str().unwrap()),
            ReprTag::Dynamic => write!(f, "<dynamic>"),
        }
    }
}

// ===================================================================
// STRUCT-3d-b: Serde bridge for Value16
// Uses existing Value serialize/deserialize (backward compat)
// ===================================================================
// Native Value16 serde (no Value bridge)
// ===================================================================
