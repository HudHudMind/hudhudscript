use crate::{
    ClassData, DataData, FunctionData, GeneratorState16, InstanceData, ObjMap,
    PromiseState16, Repr, ResourceRef, ToolRef, Value16,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
/// Heap-allocated dynamic object for String, Array, Object, Function, etc.
/// Wrapped by Repr with Dynamic tag.
pub struct DynamicObject {
    pub kind: DynamicKind,
    pub marked: std::cell::Cell<bool>,
    pub data: DynamicData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicKind {
    String,
    StringAscii,
    Array,
    Object,
    Function,
    Promise,
    Class,
    Instance,
    Data,
    Set,
    Map,
    Generator,
    Tool,
    Resource,
    Option,
    Result,
    BigInt,
}

/// Returns true for String OR StringAscii — use instead of direct
/// DynamicKind::String comparisons (P4: StringAscii is a string).
#[inline]
pub fn is_string_kind(k: DynamicKind) -> bool {
    matches!(k, DynamicKind::String | DynamicKind::StringAscii)
}

pub enum DynamicData {
    String(String),
    Array(Vec<Value16>),
    Object(ObjMap),
    Function(FunctionData),
    Instance(InstanceData),
    Promise(PromiseState16),
    Class(ClassData),
    Data(DataData),
    Set(Vec<Value16>),
    Map(Vec<(Value16, Value16)>),
    Generator(Arc<Mutex<GeneratorState16>>),
    Tool(Box<ToolRef>),
    Resource(Box<ResourceRef>),
    Option(Option<Box<Value16>>),
    Result(Result<Box<Value16>, String>),
    BigInt(num_bigint::BigInt),
}
