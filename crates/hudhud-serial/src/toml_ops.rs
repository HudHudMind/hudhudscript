//! Shared TOML builtin — used by both VM and interpreter.

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
pub enum TomlMethodId {
    Parse,
    Stringify,
}

impl std::str::FromStr for TomlMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse" => Ok(Self::Parse),
            "stringify" => Ok(Self::Stringify),
            _ => Err(runtime_error(format!("Unknown TOML method: {}", s))),
        }
    }
}

impl TomlMethodId {
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
        .ok_or_else(|| runtime_error("TOML.parse() requires a string argument"))?;
    let toml_val: toml::Value =
        toml::from_str(s).map_err(|e| runtime_error(format!("TOML.parse error: {}", e)))?;
    Ok(toml_to_value(&toml_val))
}

pub fn stringify(args: &[Value16]) -> HudHudResult<Value16> {
    let val = args
        .first()
        .ok_or_else(|| runtime_error("TOML.stringify() requires an argument"))?;
    let toml_val = value_to_toml(val)?;
    let s = toml::to_string_pretty(&toml_val)
        .map_err(|e| runtime_error(format!("TOML.stringify error: {}", e)))?;
    Ok(Value16::string(s))
}

fn toml_to_value(t: &toml::Value) -> Value16 {
    match t {
        toml::Value::String(s) => Value16::string(s.clone()),
        toml::Value::Integer(n) => Value16::number(*n as f64),
        toml::Value::Float(f) => Value16::number(*f),
        toml::Value::Boolean(b) => Value16::bool_(*b),
        toml::Value::Datetime(dt) => Value16::string(dt.to_string()),
        toml::Value::Array(arr) => Value16::array(arr.iter().map(|v| toml_to_value(v)).collect()),
        toml::Value::Table(tbl) => {
            let mut obj = HashMap::new();
            for (k, v) in tbl {
                obj.insert(k.clone(), toml_to_value(v));
            }
            Value16::object(obj)
        }
    }
}

fn value_to_toml(v: &Value16) -> HudHudResult<toml::Value> {
    if v.is_null() {
        return Ok(toml::Value::String("null".to_string()));
    }
    if let Some(b) = v.as_bool() {
        return Ok(toml::Value::Boolean(b));
    }
    if let Some(n) = v.as_number() {
        return if n.fract() == 0.0 && n.is_finite() && n >= i64::MIN as f64 && n <= i64::MAX as f64
        {
            Ok(toml::Value::Integer(n as i64))
        } else {
            Ok(toml::Value::Float(n))
        };
    }
    if let Some(s) = v.as_str() {
        return Ok(toml::Value::String(s.to_string()));
    }
    if let Some(arr) = v.as_array() {
        let items: HudHudResult<Vec<_>> = arr.iter().map(value_to_toml).collect();
        return Ok(toml::Value::Array(items?));
    }
    if let Some(obj) = v.as_object() {
        let mut tbl = toml::map::Map::new();
        for (k, val) in obj {
            tbl.insert(k.clone(), value_to_toml(val)?);
        }
        return Ok(toml::Value::Table(tbl));
    }
    Err(runtime_error(format!(
        "Cannot convert {} to TOML",
        v.type_name_str()
    )))
}
