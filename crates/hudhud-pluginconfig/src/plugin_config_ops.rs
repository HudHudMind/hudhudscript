//! Shared per-plugin configuration — load, get, set, save, merge, defaults
//! (Kural 7 — single source of truth for VM + interpreter).
//!
//! Sources (merged in order): system TOML, user TOML, env vars.

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

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    Load,
    Get,
    Set,
    Save,
    Merge,
    Watch,
    Defaults,
    Paths,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "load" => Ok(Self::Load),
            "get" => Ok(Self::Get),
            "set" => Ok(Self::Set),
            "save" => Ok(Self::Save),
            "merge" => Ok(Self::Merge),
            "watch" => Ok(Self::Watch),
            "defaults" => Ok(Self::Defaults),
            "paths" => Ok(Self::Paths),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::Load => config_load(args),
        ScriptMethodId::Get => config_get(args),
        ScriptMethodId::Set => config_set(args),
        ScriptMethodId::Save => config_save(args),
        ScriptMethodId::Merge => config_merge(args),
        ScriptMethodId::Watch => config_watch(args),
        ScriptMethodId::Defaults => config_defaults(args),
        ScriptMethodId::Paths => config_paths(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn config_load(args: &[Value16]) -> HudHudResult<Value16> {
    let plugin_name = require_str(args, 0, "PluginConfig.load")?.to_string();

    let system_path = format!("/etc/hudhud/plugins/{}.toml", plugin_name);
    let user_path = match std::env::var("HOME") {
        Ok(home) => format!("{}/.config/hudhud/plugins/{}.toml", home, plugin_name),
        Err(_) => format!("~/.config/hudhud/plugins/{}.toml", plugin_name),
    };

    let mut config: HashMap<String, Value16> = HashMap::new();

    if let Ok(content) = std::fs::read_to_string(&system_path) {
        if let Ok(parsed) = content.parse::<toml::Table>() {
            for (k, v) in parsed {
                config.insert(k, toml_to_value(&v));
            }
        }
    }
    if let Ok(content) = std::fs::read_to_string(&user_path) {
        if let Ok(parsed) = content.parse::<toml::Table>() {
            for (k, v) in parsed {
                config.insert(k, toml_to_value(&v));
            }
        }
    }

    let prefix = format!(
        "HUDHUD_PLUGIN_{}_",
        plugin_name.to_uppercase().replace('-', "_")
    );
    for (key, val) in std::env::vars() {
        if let Some(suffix) = key.strip_prefix(&prefix) {
            config.insert(suffix.to_lowercase(), Value16::string(val));
        }
    }

    config.insert("__plugin".to_string(), Value16::string(plugin_name));
    config.insert("__system_path".to_string(), Value16::string(system_path));
    config.insert("__user_path".to_string(), Value16::string(user_path));
    Ok(Value16::object(config))
}

pub fn toml_to_value(v: &toml::Value) -> Value16 {
    match v {
        toml::Value::String(s) => Value16::string(s.clone()),
        toml::Value::Integer(n) => Value16::number(*n as f64),
        toml::Value::Float(n) => Value16::number(*n),
        toml::Value::Boolean(b) => Value16::bool_(*b),
        toml::Value::Array(arr) => Value16::array(arr.iter().map(|x| toml_to_value(x)).collect()),
        toml::Value::Table(tbl) => {
            let mut map = HashMap::new();
            for (k, v) in tbl {
                map.insert(k.clone(), toml_to_value(v));
            }
            Value16::object(map)
        }
        toml::Value::Datetime(dt) => Value16::string(dt.to_string()),
    }
}

pub fn value_to_toml(v: &Value16) -> toml::Value {
    if v.is_null() {
        return toml::Value::String("null".to_string());
    }
    if let Some(b) = v.as_bool() {
        return toml::Value::Boolean(b);
    }
    if let Some(n) = v.as_number() {
        return if n.fract() == 0.0 {
            toml::Value::Integer(n as i64)
        } else {
            toml::Value::Float(n)
        };
    }
    if let Some(s) = v.as_str() {
        return toml::Value::String(s.to_string());
    }
    if let Some(arr) = v.as_array() {
        return toml::Value::Array(arr.iter().map(|x| value_to_toml(x)).collect());
    }
    if let Some(obj) = v.as_object() {
        let mut tbl = toml::map::Map::new();
        for (k, v) in obj {
            if !k.starts_with("__") {
                tbl.insert(k.clone(), value_to_toml(v));
            }
        }
        return toml::Value::Table(tbl);
    }
    toml::Value::String(v.display_string())
}

pub fn config_get(args: &[Value16]) -> HudHudResult<Value16> {
    let config = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| runtime_error("PluginConfig.get: expected config object"))?;
    let key = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("PluginConfig.get: expected key string"))?;

    let parts: Vec<&str> = key.split('.').collect();
    let mut current: Value16 = Value16::object(config.clone());
    for part in &parts {
        let next = current.as_object().and_then(|o| o.get(*part)).cloned();
        match next {
            Some(v) => current = v,
            None => return Ok(Value16::null()),
        }
    }
    Ok(current)
}

pub fn config_set(args: &[Value16]) -> HudHudResult<Value16> {
    let mut config = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| runtime_error("PluginConfig.set: expected config object"))?
        .clone();
    let key = args
        .get(1)
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("PluginConfig.set: expected key string"))?
        .to_string();
    let value = args.get(2).cloned().unwrap_or_else(Value16::null);
    config.insert(key, value);
    Ok(Value16::object(config))
}

pub fn config_save(args: &[Value16]) -> HudHudResult<Value16> {
    let config = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| runtime_error("PluginConfig.save: expected config object"))?;

    let path = args
        .get(1)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            config
                .get("__user_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| {
            runtime_error("PluginConfig.save: no path specified and no __user_path in config")
        })?;

    let mut toml_table = toml::map::Map::new();
    for (k, v) in config {
        if k.starts_with("__") {
            continue;
        }
        toml_table.insert(k.clone(), value_to_toml(v));
    }
    let content = toml::to_string_pretty(&toml::Value::Table(toml_table))
        .map_err(|e| runtime_error(format!("PluginConfig.save: serialize error: {}", e)))?;

    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, content)
        .map_err(|e| runtime_error(format!("PluginConfig.save: write error: {}", e)))?;

    let mut result = HashMap::new();
    result.insert("saved".to_string(), Value16::bool_(true));
    result.insert("path".to_string(), Value16::string(path));
    Ok(Value16::object(result))
}

pub fn config_merge(args: &[Value16]) -> HudHudResult<Value16> {
    let mut base = args
        .first()
        .and_then(|v| v.as_object())
        .ok_or_else(|| runtime_error("PluginConfig.merge: expected base config object"))?
        .clone();
    let overlay = args
        .get(1)
        .and_then(|v| v.as_object())
        .ok_or_else(|| runtime_error("PluginConfig.merge: expected overlay config object"))?;
    for (k, v) in overlay {
        base.insert(k.clone(), v.clone());
    }
    Ok(Value16::object(base))
}

pub fn config_watch(args: &[Value16]) -> HudHudResult<Value16> {
    let path = args
        .first()
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(obj) = v.as_object() {
                obj.get("__user_path")
                    .and_then(|p| p.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let mut result = HashMap::new();
    result.insert("watching".to_string(), Value16::bool_(true));
    result.insert("path".to_string(), Value16::string(path));
    Ok(Value16::object(result))
}

pub fn config_defaults(args: &[Value16]) -> HudHudResult<Value16> {
    let plugin_name = require_str(args, 0, "PluginConfig.defaults")?.to_string();
    let defaults = args
        .get(1)
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let mut config = defaults;
    config.insert("__plugin".to_string(), Value16::string(plugin_name));
    config.insert("__defaults_applied".to_string(), Value16::bool_(true));
    Ok(Value16::object(config))
}

pub fn config_paths(args: &[Value16]) -> HudHudResult<Value16> {
    let plugin_name = require_str(args, 0, "PluginConfig.paths")?.to_string();
    let system_path = format!("/etc/hudhud/plugins/{}.toml", plugin_name);
    let user_path = match std::env::var("HOME") {
        Ok(home) => format!("{}/.config/hudhud/plugins/{}.toml", home, plugin_name),
        Err(_) => format!("~/.config/hudhud/plugins/{}.toml", plugin_name),
    };
    let mut result = HashMap::new();
    result.insert("system".to_string(), Value16::string(system_path));
    result.insert("user".to_string(), Value16::string(user_path));
    Ok(Value16::object(result))
}

fn require_str<'a>(args: &'a [Value16], idx: usize, method: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            method, idx
        ))),
    }
}
