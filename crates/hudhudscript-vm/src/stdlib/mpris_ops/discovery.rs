use hudhudscript_bytecode::shared_value::{runtime_error, type_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::process::Command;

use super::{MPRIS_PATH, MPRIS_PLAYER_IFACE, MPRIS_PREFIX};

pub(crate) fn has_gdbus() -> bool {
    Command::new("gdbus")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn list_mpris_players() -> SharedResult<Vec<String>> {
    let raw = if has_gdbus() {
        let output = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                "org.freedesktop.DBus",
                "--object-path",
                "/org/freedesktop/DBus",
                "--method",
                "org.freedesktop.DBus.ListNames",
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: failed to execute gdbus: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!(
                "mpris: gdbus ListNames failed: {stderr}"
            )));
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        let output = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                "--dest=org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus.ListNames",
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: failed to execute dbus-send: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!(
                "mpris: dbus-send ListNames failed: {stderr}"
            )));
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    };

    let mut players = Vec::new();
    for segment in raw.split('\'') {
        let s = segment.trim();
        if s.starts_with(MPRIS_PREFIX) {
            players.push(s.to_string());
        }
    }

    if players.is_empty() {
        for line in raw.lines() {
            let trimmed = line.trim().trim_matches('"');
            if trimmed.starts_with(MPRIS_PREFIX) {
                players.push(trimmed.to_string());
            }
        }
    }

    Ok(players)
}

pub fn resolve_player(args: &[Value16], idx: usize, op: &str) -> SharedResult<String> {
    match args.get(idx) {
        Some(v) if v.is_null() => {
            let players = list_mpris_players()?;
            players.into_iter().next().ok_or_else(|| {
                runtime_error(format!("{op}: no MPRIS media players found on session bus"))
            })
        }
        Some(v) => {
            if let Some(s) = v.as_str() {
                if s.is_empty() {
                    let players = list_mpris_players()?;
                    return players.into_iter().next().ok_or_else(|| {
                        runtime_error(format!("{op}: no MPRIS media players found on session bus"))
                    });
                }
                if s.starts_with("org.mpris.MediaPlayer2.") {
                    Ok(s.to_string())
                } else {
                    Ok(format!("{MPRIS_PREFIX}{s}"))
                }
            } else {
                Err(type_error("string or null", v.type_name_str(), op))
            }
        }
        None => {
            let players = list_mpris_players()?;
            players.into_iter().next().ok_or_else(|| {
                runtime_error(format!("{op}: no MPRIS media players found on session bus"))
            })
        }
    }
}

pub fn short_name(bus_name: &str) -> String {
    bus_name
        .strip_prefix(MPRIS_PREFIX)
        .unwrap_or(bus_name)
        .to_string()
}
