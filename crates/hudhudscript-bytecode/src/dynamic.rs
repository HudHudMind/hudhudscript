use crate::{
    ClassData, DataData, FunctionData, GeneratorState16, InstanceData, PromiseState16, Repr,
    ResourceRef, ToolRef, Value16,
};
use parking_lot::Mutex;
use std::sync::Arc;
/// Heap-allocated dynamic object for String, Array, Object, Function, etc.
/// Wrapped by Repr with Dynamic tag.
pub struct DynamicObject {
    pub kind: DynamicKind,
    pub data: DynamicData,
}

pub enum DynamicKind {
    String,
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
}

pub enum DynamicData {
    String(String),
    Array(Vec<Value16>),
    Object(std::collections::HashMap<String, Value16>),
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
}
