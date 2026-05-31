use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use std::process::Command;

use super::{has_gdbus, MPRIS_PATH, MPRIS_PLAYER_IFACE};

pub(crate) fn call_player_method(dest: &str, method: &str) -> SharedResult<()> {
    if has_gdbus() {
        let output = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                dest,
                "--object-path",
                MPRIS_PATH,
                "--method",
                &format!("{MPRIS_PLAYER_IFACE}.{method}"),
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: gdbus failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!("mpris: {method} failed: {stderr}")));
        }
    } else {
        let output = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                &format!("--dest={dest}"),
                MPRIS_PATH,
                &format!("{MPRIS_PLAYER_IFACE}.{method}"),
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: dbus-send failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!("mpris: {method} failed: {stderr}")));
        }
    }
    Ok(())
}

pub(crate) fn get_player_property(dest: &str, property: &str) -> SharedResult<String> {
    if has_gdbus() {
        let output = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                dest,
                "--object-path",
                MPRIS_PATH,
                "--method",
                "org.freedesktop.DBus.Properties.Get",
                MPRIS_PLAYER_IFACE,
                property,
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: gdbus get property failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!(
                "mpris: get {property} failed: {stderr}"
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let output = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                &format!("--dest={dest}"),
                MPRIS_PATH,
                "org.freedesktop.DBus.Properties.Get",
                &format!("string:{MPRIS_PLAYER_IFACE}"),
                &format!("string:{property}"),
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: dbus-send get property failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!(
                "mpris: get {property} failed: {stderr}"
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

pub(crate) fn set_player_property_double(
    dest: &str,
    property: &str,
    value: f64,
) -> SharedResult<()> {
    if has_gdbus() {
        let output = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                dest,
                "--object-path",
                MPRIS_PATH,
                "--method",
                "org.freedesktop.DBus.Properties.Set",
                MPRIS_PLAYER_IFACE,
                property,
                &format!("<{value}>"),
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: gdbus set property failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!(
                "mpris: set {property} failed: {stderr}"
            )));
        }
    } else {
        let output = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                &format!("--dest={dest}"),
                MPRIS_PATH,
                "org.freedesktop.DBus.Properties.Set",
                &format!("string:{MPRIS_PLAYER_IFACE}"),
                &format!("string:{property}"),
                &format!("variant:double:{value}"),
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: dbus-send set property failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!(
                "mpris: set {property} failed: {stderr}"
            )));
        }
    }
    Ok(())
}
