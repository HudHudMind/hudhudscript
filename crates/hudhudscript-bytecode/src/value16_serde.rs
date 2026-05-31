use crate::{Value16, ValueDto};
use serde::{Deserialize, Serialize};

impl Serialize for Value16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let dto = ValueDto::from(self);
        dto.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Value16 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dto = ValueDto::deserialize(deserializer)?;
        Ok(Value16::from(dto))
    }
}

// ===================================================================
// STRUCT-3d-b-0001: Type Alias Migration
// Default Value = Value16 for the new VM stack era.
// Legacy code can use `Value` (the 32B enum) directly.
