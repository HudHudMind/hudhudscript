//! Shared CSV builtin — used by both VM and interpreter.

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
pub enum CsvMethodId {
    Parse,
    Stringify,
}

impl std::str::FromStr for CsvMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parse" => Ok(Self::Parse),
            "stringify" => Ok(Self::Stringify),
            _ => Err(runtime_error(format!("Unknown CSV method: {}", s))),
        }
    }
}

impl CsvMethodId {
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
        .ok_or_else(|| runtime_error("CSV.parse() requires a string argument"))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(s.as_bytes());

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| runtime_error(format!("CSV.parse error: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .map(|k| k.to_string()).collect();

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| runtime_error(format!("CSV.parse error: {}", e)))?;
        let mut obj = hudhudscript_bytecode::ObjMap::default();
        for (i, field) in record.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| i.to_string());
            let val = if let Ok(n) = field.parse::<f64>() {
                Value16::number(n)
            } else if field == "true" {
                Value16::bool_(true)
            } else if field == "false" {
                Value16::bool_(false)
            } else {
                Value16::string(field.to_string())
            };
            obj.insert(key, val);
        }
        rows.push(Value16::object(obj));
    }
    Ok(Value16::array(rows))
}

pub fn stringify(args: &[Value16]) -> HudHudResult<Value16> {
    let arr = args
        .first()
        .and_then(|v| v.as_array())
        .ok_or_else(|| runtime_error("CSV.stringify expects an array of objects"))?;

    if arr.is_empty() {
        return Ok(Value16::string(String::new()));
    }

    let headers: Vec<String> = arr[0]
        .as_object()
        .ok_or_else(|| runtime_error("CSV.stringify expects an array of objects"))?
        .keys()
        .map(|k| k.to_string())
        .collect();

    let mut sorted_headers = headers;
    sorted_headers.sort();

    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record(&sorted_headers)
        .map_err(|e| runtime_error(format!("CSV.stringify error: {}", e)))?;

    for row in arr {
        if let Some(obj) = row.as_object() {
            let fields: Vec<String> = sorted_headers
                .iter()
                .map(|h| match obj.get(h) {
                    Some(v) => {
                        if let Some(s) = v.as_str() {
                            s.to_string()
                        } else if let Some(n) = v.as_number() {
                            crate::format_number(n)
                        } else if let Some(b) = v.as_bool() {
                            b.to_string()
                        } else {
                            String::new()
                        }
                    }
                    None => String::new(),
                })
                .collect();
            wtr.write_record(&fields)
                .map_err(|e| runtime_error(format!("CSV.stringify error: {}", e)))?;
        }
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| runtime_error(format!("CSV.stringify error: {}", e)))?;
    let s = String::from_utf8(bytes)
        .map_err(|e| runtime_error(format!("CSV.stringify error: {}", e)))?;
    Ok(Value16::string(s))
}
