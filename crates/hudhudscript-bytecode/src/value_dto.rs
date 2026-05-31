use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    ClassData, DataData, FunctionData, GeneratorState, InstanceData, PromiseState, ReprTag,
    ResourceRef, ToolRef, Value16,
};

/// A serialization-friendly DTO for Value that mirrors the pre-Struct-3 memory layout.
/// Used to maintain backward compatibility with older `.hudb` files during serde.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueDto {
    Null,
    Boolean(bool),
    Int(i64),
    Number(f64),
    String(String),
    Array(Vec<ValueDto>),
    Object(Box<std::collections::HashMap<String, ValueDto>>),
    Function(Box<FunctionData>), // NOTE: FunctionData.captures use Arc<RwLock<Value16>> — ValueDto does not model captures because serialization drains them
    Promise(Box<PromiseState>),
    Option(Option<Box<ValueDto>>),
    Result(std::result::Result<Box<ValueDto>, String>),
    Class(Box<ClassData>),
    Instance(Box<InstanceData>),
    Data(Box<DataData>),
    Set(Vec<ValueDto>),
    Map(Vec<(ValueDto, ValueDto)>),
    #[serde(skip)]
    Generator(Arc<Mutex<GeneratorState>>),
    Tool(Box<ToolRef>),
    Resource(Box<ResourceRef>),
}

impl From<&Value16> for ValueDto {
    fn from(v: &Value16) -> Self {
        match v.0.tag() {
            ReprTag::Null => ValueDto::Null,
            ReprTag::Bool => ValueDto::Boolean(v.as_bool().unwrap_or(false)),
            ReprTag::Int => ValueDto::Int(v.as_int().unwrap_or(0)),
            ReprTag::Number => ValueDto::Number(v.as_number().unwrap_or(0.0)),
            ReprTag::InlineString => ValueDto::String(v.as_str().unwrap_or("").to_string()),
            ReprTag::Dynamic => {
                if let Some(s) = v.as_str() {
                    ValueDto::String(s.to_string())
                } else if let Some(arr) = v.as_array() {
                    ValueDto::Array(arr.iter().map(|x| x.into()).collect())
                } else if let Some(obj) = v.as_object() {
                    let map: std::collections::HashMap<String, ValueDto> =
                        obj.iter().map(|(k, v)| (k.clone(), v.into())).collect();
                    ValueDto::Object(Box::new(map))
                } else if let Some(fd) = v.as_function_data() {
                    ValueDto::Function(Box::new(fd.clone()))
                } else if let Some(cd) = v.as_class_data() {
                    ValueDto::Class(Box::new(cd.clone()))
                } else if let Some(inst) = v.as_instance_data() {
                    ValueDto::Instance(Box::new(inst.clone()))
                } else if let Some(set_items) = v.as_set() {
                    ValueDto::Set(set_items.iter().map(|x| x.into()).collect())
                } else if let Some(map_pairs) = v.as_map_pairs() {
                    ValueDto::Map(map_pairs.iter().map(|(k, v)| (k.into(), v.into())).collect())
                } else if let Some(opt) = v.as_option() {
                    ValueDto::Option(opt.map(|x| Box::new(x.into())))
                } else if let Some(res) = v.as_result() {
                    ValueDto::Result(res.map(|x| Box::new(x.into())).map_err(|e| e.clone()))
                } else {
                    // Truly unserializable: Promise, Generator, Tool, Resource
                    ValueDto::Null
                }
            }
        }
    }
}

impl From<ValueDto> for Value16 {
    fn from(dto: ValueDto) -> Self {
        match dto {
            ValueDto::Null => Value16::null(),
            ValueDto::Boolean(b) => Value16::bool_(b),
            ValueDto::Int(i) => Value16::int(i),
            ValueDto::Number(n) => Value16::number(n),
            ValueDto::String(s) => Value16::string(s),
            ValueDto::Array(arr) => Value16::array(arr.into_iter().map(|x| x.into()).collect()),
            ValueDto::Object(obj) => {
                Value16::object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            ValueDto::Function(fd) => Value16::function(*fd),
            ValueDto::Class(cd) => Value16::class(*cd),
            ValueDto::Instance(inst) => Value16::instance(*inst),
            ValueDto::Set(items) => Value16::set(items.into_iter().map(|x| x.into()).collect()),
            ValueDto::Map(pairs) => Value16::map(pairs.into_iter().map(|(k, v)| (k.into(), v.into())).collect()),
            ValueDto::Option(opt) => Value16::option(opt.map(|x| (*x).into())),
            ValueDto::Result(res) => Value16::result(res.map(|x| (*x).into())),
            // Promise/Generator/Tool/Resource — runtime state, not in bytecode constants
            _ => Value16::null(),
        }
    }
}
