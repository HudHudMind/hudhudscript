//! Shared INI builtin — used by both VM and interpreter.

use crate::format_number;
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
pub enum IniMethodId {
    Parse,
    Stringify,
}

impl std::str::FromStr for IniMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse" => Ok(Self::Parse),
            "stringify" => Ok(Self::Stringify),
            _ => Err(runtime_error(format!("Unknown INI method: {}", s))),
        }
    }
}

impl IniMethodId {
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
        .ok_or_else(|| runtime_error("INI.parse() requires a string argument"))?;

    let mut result: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    let mut current_section = String::new();

    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].trim().to_string();
            if !result.contains_key(&current_section) {
                result.insert(
                    current_section.clone(),
                    Value16::object(hudhudscript_bytecode::ObjMap::default()),
                );
            }
        } else if let Some((key, val)) = line.split_once('=') {
            let key = key.trim().to_string();
            let val_str = val.trim().to_string();
            let value = if let Ok(n) = val_str.parse::<f64>() {
                Value16::number(n)
            } else if val_str == "true" {
                Value16::bool_(true)
            } else if val_str == "false" {
                Value16::bool_(false)
            } else {
                Value16::string(val_str)
            };

            if current_section.is_empty() {
                result.insert(key, value);
            } else if let Some(section) = result.get(&current_section) {
                if let Some(section_obj) = section.as_object() {
                    let mut new_section = section_obj.clone();
                    new_section.insert(key, value);
                    result.insert(current_section.clone(), Value16::object(new_section));
                }
            }
        }
    }
    Ok(Value16::object(result))
}

pub fn stringify(args: &[Value16]) -> HudHudResult<Value16> {
    let obj = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| runtime_error("INI.stringify() requires an object argument"))?;

    let mut output = String::new();
    let mut top_level = Vec::new();
    let mut sections = Vec::new();

    for (key, val) in obj {
        if val.as_object().is_some() {
            sections.push((key.clone(), val.clone()));
        } else {
            top_level.push((key.clone(), val.clone()));
        }
    }

    top_level.sort_by(|a, b| a.0.cmp(&b.0));
    for (key, val) in &top_level {
        output.push_str(&format!("{} = {}\n", key, value_to_ini_string(val)));
    }

    if !top_level.is_empty() && !sections.is_empty() {
        output.push('\n');
    }

    sections.sort_by(|a, b| a.0.cmp(&b.0));
    for (section, val) in &sections {
        output.push_str(&format!("[{}]\n", section));
        if let Some(entries) = val.as_object() {
            let mut sorted: Vec<_> = entries.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for (k, v) in sorted {
                output.push_str(&format!("{} = {}\n", k, value_to_ini_string(v)));
            }
        }
        output.push('\n');
    }
    Ok(Value16::string(output.trim_end().to_string()))
}

fn value_to_ini_string(v: &Value16) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    if let Some(n) = v.as_number() {
        return format_number(n);
    }
    if let Some(b) = v.as_bool() {
        return b.to_string();
    }
    if v.is_null() {
        return String::new();
    }
    v.display_string()
}
