//! Shared YAML builtin — used by both VM and interpreter.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YamlMethodId {
    Parse,
    Stringify,
}

impl std::str::FromStr for YamlMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse" => Ok(Self::Parse),
            "stringify" => Ok(Self::Stringify),
            _ => Err(runtime_error(format!("Unknown YAML method: {}", s))),
        }
    }
}

impl YamlMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Parse => parse(args),
            Self::Stringify => stringify(args),
        }
    }
}

pub fn parse(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("YAML.parse() requires a string argument"))?;
    let yaml_val: serde_yaml::Value =
        serde_yaml::from_str(s).map_err(|e| runtime_error(format!("YAML.parse error: {}", e)))?;
    Ok(yaml_to_value(&yaml_val))
}

pub fn stringify(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("YAML.stringify() requires an argument"))?;
    let yaml_val = value_to_yaml(val)?;
    let s = serde_yaml::to_string(&yaml_val)
        .map_err(|e| runtime_error(format!("YAML.stringify error: {}", e)))?;
    Ok(Value16::string(s))
}

fn yaml_to_value(y: &serde_yaml::Value) -> Value16 {
    match y {
        serde_yaml::Value::Null => Value16::null(),
        serde_yaml::Value::Bool(b) => Value16::bool_(*b),
        serde_yaml::Value::Number(n) => Value16::number(n.as_f64().unwrap_or(0.0)),
        serde_yaml::Value::String(s) => Value16::string(s.clone()),
        serde_yaml::Value::Sequence(arr) => {
            Value16::array(arr.iter().map(|v| yaml_to_value(v)).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = HashMap::new();
            for (k, v) in map {
                if let Some(key) = k.as_str() {
                    obj.insert(key.to_string(), yaml_to_value(v));
                }
            }
            Value16::object(obj)
        }
        serde_yaml::Value::Tagged(t) => yaml_to_value(&t.value),
    }
}

fn value_to_yaml(v: &Value16) -> HudHudResult<serde_yaml::Value> {
    if v.is_null() {
        return Ok(serde_yaml::Value::Null);
    }
    if let Some(b) = v.as_bool() {
        return Ok(serde_yaml::Value::Bool(b));
    }
    if let Some(n) = v.as_number() {
        return Ok(serde_yaml::Value::Number(serde_yaml::Number::from(n)));
    }
    if let Some(s) = v.as_str() {
        return Ok(serde_yaml::Value::String(s.to_string()));
    }
    if let Some(arr) = v.as_array() {
        let items: HudHudResult<Vec<_>> = arr.iter().map(value_to_yaml).collect();
        return Ok(serde_yaml::Value::Sequence(items?));
    }
    if let Some(obj) = v.as_object() {
        let mut map = serde_yaml::Mapping::new();
        for (k, val) in obj {
            map.insert(serde_yaml::Value::String(k.clone()), value_to_yaml(val)?);
        }
        return Ok(serde_yaml::Value::Mapping(map));
    }
    Err(runtime_error(format!(
        "Cannot convert {} to YAML",
        v.type_name_str()
    )))
}
