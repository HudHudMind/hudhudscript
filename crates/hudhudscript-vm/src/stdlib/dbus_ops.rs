//! Shared D-Bus client (gdbus or dbus-send) — Kural 7.

use hudhudscript_bytecode::shared_value::{runtime_error, type_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;
use std::process::Command;

pub fn call_dbus_method(method: &str, args: &[Value16]) -> SharedResult<Value16> {
    match method {
        "call" => dbus_call(args),
        "system_call" => dbus_system_call(args),
        "session_call" => dbus_session_call(args),
        "get_property" => dbus_get_property(args),
        "list_names" => dbus_list_names(args),
        "network_status" => dbus_network_status(args),
        "bluetooth_powered" => dbus_bluetooth_powered(args),
        "battery_percentage" => dbus_battery_percentage(args),
        _ => Err(runtime_error(format!("Unknown dbus method: {}", method))),
    }
}

fn has_gdbus() -> bool {
    Command::new("gdbus")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn value_to_gvariant(val: &Value16) -> String {
    if let Some(s) = val.as_str() {
        return format!("'{}'", s);
    }
    if let Some(n) = val.as_number() {
        return if n == (n as i64) as f64 {
            format!("{}", n as i64)
        } else {
            format!("{}", n)
        };
    }
    if let Some(b) = val.as_bool() {
        return if b { "true" } else { "false" }.to_string();
    }
    if let Some(arr) = val.as_array() {
        let inner: Vec<String> = arr.iter().map(|x| value_to_gvariant(x)).collect();
        return format!("[{}]", inner.join(", "));
    }
    "''".to_string()
}

fn value_to_dbus_send_args(val: &Value16) -> Vec<String> {
    if let Some(s) = val.as_str() {
        return vec![format!("string:{}", s)];
    }
    if let Some(n) = val.as_number() {
        return if n == (n as i64) as f64 {
            vec![format!("int32:{}", n as i64)]
        } else {
            vec![format!("double:{}", n)]
        };
    }
    if let Some(b) = val.as_bool() {
        return vec![format!("boolean:{}", b)];
    }
    if let Some(arr) = val.as_array() {
        return arr
            .iter()
            .flat_map(|x| value_to_dbus_send_args(x))
            .collect();
    }
    vec![]
}

fn run_dbus_call(
    bus: &str,
    dest: &str,
    path: &str,
    interface: &str,
    method: &str,
    args: &[Value16],
) -> SharedResult<String> {
    let bus_flag = match bus {
        "system" => "--system",
        "session" => "--session",
        _ => {
            return Err(runtime_error(format!(
                "dbus: unknown bus type '{}', expected 'system' or 'session'",
                bus
            )));
        }
    };

    if has_gdbus() {
        let mut cmd = Command::new("gdbus");
        cmd.arg("call")
            .arg(bus_flag)
            .arg("--dest")
            .arg(dest)
            .arg("--object-path")
            .arg(path)
            .arg("--method")
            .arg(format!("{}.{}", interface, method));
        for arg in args {
            cmd.arg(value_to_gvariant(arg));
        }
        let output = cmd
            .output()
            .map_err(|e| runtime_error(format!("dbus: failed to execute gdbus: {}", e)))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(runtime_error(format!(
                "dbus: gdbus call failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    } else {
        let mut cmd = Command::new("dbus-send");
        cmd.arg(bus_flag)
            .arg("--print-reply")
            .arg(format!("--dest={}", dest))
            .arg(path)
            .arg(format!("{}.{}", interface, method));
        for arg in args {
            for dbus_arg in value_to_dbus_send_args(arg) {
                cmd.arg(&dbus_arg);
            }
        }
        let output = cmd
            .output()
            .map_err(|e| runtime_error(format!("dbus: failed to execute dbus-send: {}", e)))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(runtime_error(format!(
                "dbus: dbus-send call failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    }
}

fn run_dbus_get_property(
    bus: &str,
    dest: &str,
    path: &str,
    interface: &str,
    property: &str,
) -> SharedResult<String> {
    let bus_flag = match bus {
        "system" => "--system",
        "session" => "--session",
        _ => {
            return Err(runtime_error(format!(
                "dbus: unknown bus type '{}', expected 'system' or 'session'",
                bus
            )));
        }
    };

    if has_gdbus() {
        let output = Command::new("gdbus")
            .arg("call")
            .arg(bus_flag)
            .arg("--dest")
            .arg(dest)
            .arg("--object-path")
            .arg(path)
            .arg("--method")
            .arg("org.freedesktop.DBus.Properties.Get")
            .arg(format!("'{}'", interface))
            .arg(format!("'{}'", property))
            .output()
            .map_err(|e| runtime_error(format!("dbus: failed to execute gdbus: {}", e)))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(runtime_error(format!(
                "dbus: gdbus get_property failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    } else {
        let output = Command::new("dbus-send")
            .arg(bus_flag)
            .arg("--print-reply")
            .arg(format!("--dest={}", dest))
            .arg(path)
            .arg("org.freedesktop.DBus.Properties.Get")
            .arg(format!("string:{}", interface))
            .arg(format!("string:{}", property))
            .output()
            .map_err(|e| runtime_error(format!("dbus: failed to execute dbus-send: {}", e)))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(runtime_error(format!(
                "dbus: dbus-send get_property failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    }
}

fn parse_dbus_output(raw: &str) -> Value16 {
    let trimmed = raw.trim();
    if let Ok(n) = trimmed.parse::<f64>() {
        return Value16::number(n);
    }
    if trimmed == "true" || trimmed.contains("boolean true") {
        return Value16::boolean(true);
    }
    if trimmed == "false" || trimmed.contains("boolean false") {
        return Value16::boolean(false);
    }
    Value16::string(trimmed.to_string())
}

pub fn dbus_call(args: &[Value16]) -> SharedResult<Value16> {
    let bus = require_str(args, 0, "dbus.call")?.to_string();
    let dest = require_str(args, 1, "dbus.call")?.to_string();
    let path = require_str(args, 2, "dbus.call")?.to_string();
    let interface = require_str(args, 3, "dbus.call")?.to_string();
    let method = require_str(args, 4, "dbus.call")?.to_string();
    let empty: Vec<Value16> = Vec::new();
    let call_args = args
        .get(5)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or(empty);
    let raw = run_dbus_call(&bus, &dest, &path, &interface, &method, &call_args)?;
    Ok(parse_dbus_output(&raw))
}

pub fn dbus_system_call(args: &[Value16]) -> SharedResult<Value16> {
    let dest = require_str(args, 0, "dbus.system_call")?.to_string();
    let path = require_str(args, 1, "dbus.system_call")?.to_string();
    let interface = require_str(args, 2, "dbus.system_call")?.to_string();
    let method = require_str(args, 3, "dbus.system_call")?.to_string();
    let empty: Vec<Value16> = Vec::new();
    let call_args = args
        .get(4)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or(empty);
    let raw = run_dbus_call("system", &dest, &path, &interface, &method, &call_args)?;
    Ok(parse_dbus_output(&raw))
}

pub fn dbus_session_call(args: &[Value16]) -> SharedResult<Value16> {
    let dest = require_str(args, 0, "dbus.session_call")?.to_string();
    let path = require_str(args, 1, "dbus.session_call")?.to_string();
    let interface = require_str(args, 2, "dbus.session_call")?.to_string();
    let method = require_str(args, 3, "dbus.session_call")?.to_string();
    let empty: Vec<Value16> = Vec::new();
    let call_args = args
        .get(4)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or(empty);
    let raw = run_dbus_call("session", &dest, &path, &interface, &method, &call_args)?;
    Ok(parse_dbus_output(&raw))
}

pub fn dbus_get_property(args: &[Value16]) -> SharedResult<Value16> {
    let bus = require_str(args, 0, "dbus.get_property")?.to_string();
    let dest = require_str(args, 1, "dbus.get_property")?.to_string();
    let path = require_str(args, 2, "dbus.get_property")?.to_string();
    let interface = require_str(args, 3, "dbus.get_property")?.to_string();
    let property = require_str(args, 4, "dbus.get_property")?.to_string();
    let raw = run_dbus_get_property(&bus, &dest, &path, &interface, &property)?;
    Ok(parse_dbus_output(&raw))
}

pub fn dbus_list_names(args: &[Value16]) -> SharedResult<Value16> {
    let bus = require_str(args, 0, "dbus.list_names")?.to_string();
    let empty: Vec<Value16> = Vec::new();
    let raw = run_dbus_call(
        &bus,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "ListNames",
        &empty,
    )?;

    let mut names: Vec<Value16> = Vec::new();
    for segment in raw.split('\'') {
        let s = segment.trim();
        if !s.is_empty() && s.contains('.') && !s.starts_with('(') && !s.starts_with('[') {
            names.push(Value16::string(s.to_string()));
        }
    }
    if names.is_empty() {
        for line in raw.lines() {
            let trimmed = line.trim().trim_matches('"');
            if trimmed.contains('.') && !trimmed.contains("array") && !trimmed.contains("string") {
                names.push(Value16::string(trimmed.to_string()));
            }
        }
    }
    Ok(Value16::array(names))
}

pub fn dbus_network_status(_args: &[Value16]) -> SharedResult<Value16> {
    let raw = run_dbus_get_property(
        "system",
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        "org.freedesktop.NetworkManager",
        "Connectivity",
    );
    match raw {
        Ok(output) => {
            let label = if output.contains('4') {
                "full"
            } else if output.contains('3') {
                "limited"
            } else if output.contains('2') {
                "portal"
            } else if output.contains('1') {
                "none"
            } else {
                "unknown"
            };
            Ok(Value16::string(label.to_string()))
        }
        Err(_) => Ok(Value16::string("unavailable".to_string())),
    }
}

pub fn dbus_bluetooth_powered(_args: &[Value16]) -> SharedResult<Value16> {
    let raw = run_dbus_get_property(
        "system",
        "org.bluez",
        "/org/bluez/hci0",
        "org.bluez.Adapter1",
        "Powered",
    );
    match raw {
        Ok(output) => Ok(Value16::boolean(output.contains("true"))),
        Err(_) => Ok(Value16::boolean(false)),
    }
}

pub fn dbus_battery_percentage(_args: &[Value16]) -> SharedResult<Value16> {
    let raw = run_dbus_get_property(
        "system",
        "org.freedesktop.UPower",
        "/org/freedesktop/UPower/devices/DisplayDevice",
        "org.freedesktop.UPower.Device",
        "Percentage",
    );
    match raw {
        Ok(output) => {
            for token in output.split_whitespace() {
                let cleaned = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
                if let Ok(n) = cleaned.parse::<f64>() {
                    return Ok(Value16::number(n));
                }
            }
            Ok(Value16::number(-1.0))
        }
        Err(_) => Ok(Value16::number(-1.0)),
    }
}

fn require_str<'a>(args: &'a [Value16], idx: usize, op: &str) -> SharedResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), op)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            op, idx
        ))),
    }
}
