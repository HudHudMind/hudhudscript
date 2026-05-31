use hudhudscript_bytecode::shared_value::{runtime_error, type_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

use super::{
    extract_double_from_property, extract_int64_from_property, extract_metadata_int64,
    extract_metadata_string, extract_string_from_property, get_player_property, has_gdbus,
    list_mpris_players, resolve_player, set_player_property_double, short_name, MPRIS_PATH,
    MPRIS_PLAYER_IFACE, MPRIS_PREFIX,
};
use std::process::Command;

pub fn mpris_players(_args: &[Value16]) -> SharedResult<Value16> {
    let players = list_mpris_players()?;
    let names: Vec<Value16> = players
        .iter()
        .map(|p| Value16::string(short_name(p)))
        .collect();
    Ok(Value16::array(names))
}

pub fn mpris_play(args: &[Value16]) -> SharedResult<Value16> {
    let dest = resolve_player(args, 0, "mpris.play")?;
    super::player_ops::call_player_method(&dest, "Play")?;
    Ok(Value16::boolean(true))
}

pub fn mpris_pause(args: &[Value16]) -> SharedResult<Value16> {
    let dest = resolve_player(args, 0, "mpris.pause")?;
    super::player_ops::call_player_method(&dest, "Pause")?;
    Ok(Value16::boolean(true))
}

pub fn mpris_play_pause(args: &[Value16]) -> SharedResult<Value16> {
    let dest = resolve_player(args, 0, "mpris.play_pause")?;
    super::player_ops::call_player_method(&dest, "PlayPause")?;
    Ok(Value16::boolean(true))
}

pub fn mpris_stop(args: &[Value16]) -> SharedResult<Value16> {
    let dest = resolve_player(args, 0, "mpris.stop")?;
    super::player_ops::call_player_method(&dest, "Stop")?;
    Ok(Value16::boolean(true))
}

pub fn mpris_next(args: &[Value16]) -> SharedResult<Value16> {
    let dest = resolve_player(args, 0, "mpris.next")?;
    super::player_ops::call_player_method(&dest, "Next")?;
    Ok(Value16::boolean(true))
}

pub fn mpris_previous(args: &[Value16]) -> SharedResult<Value16> {
    let dest = resolve_player(args, 0, "mpris.previous")?;
    super::player_ops::call_player_method(&dest, "Previous")?;
    Ok(Value16::boolean(true))
}

pub fn mpris_status(args: &[Value16]) -> SharedResult<Value16> {
    let dest = resolve_player(args, 0, "mpris.status")?;
    let mut obj = HashMap::new();

    let status_raw = get_player_property(&dest, "PlaybackStatus").unwrap_or_default();
    obj.insert(
        "playback_status".to_string(),
        Value16::string(extract_string_from_property(&status_raw)),
    );

    let metadata_raw = get_player_property(&dest, "Metadata").unwrap_or_default();

    obj.insert(
        "title".to_string(),
        Value16::string(extract_metadata_string(&metadata_raw, "xesam:title")),
    );
    obj.insert(
        "artist".to_string(),
        Value16::string(extract_metadata_string(&metadata_raw, "xesam:artist")),
    );
    obj.insert(
        "album".to_string(),
        Value16::string(extract_metadata_string(&metadata_raw, "xesam:album")),
    );

    let length_us = extract_metadata_int64(&metadata_raw, "mpris:length");
    obj.insert(
        "length".to_string(),
        Value16::number(length_us as f64 / 1_000_000.0),
    );

    let position_raw = get_player_property(&dest, "Position").unwrap_or_default();
    let position_us = extract_int64_from_property(&position_raw);
    obj.insert(
        "position".to_string(),
        Value16::number(position_us as f64 / 1_000_000.0),
    );

    obj.insert("player".to_string(), Value16::string(short_name(&dest)));

    Ok(Value16::object(obj))
}

pub fn mpris_volume(args: &[Value16]) -> SharedResult<Value16> {
    let first_is_num = args
        .first()
        .map(|v| v.as_number().is_some())
        .unwrap_or(false);

    let (player_arg, level_arg) = if first_is_num {
        (None, args.first())
    } else {
        (args.first(), args.get(1))
    };

    let dest = match player_arg {
        Some(v) if v.is_null() => {
            let players = list_mpris_players()?;
            players.into_iter().next().ok_or_else(|| {
                runtime_error("mpris.volume: no MPRIS media players found on session bus")
            })?
        }
        Some(v) => {
            if let Some(s) = v.as_str() {
                if s.is_empty() {
                    let players = list_mpris_players()?;
                    players.into_iter().next().ok_or_else(|| {
                        runtime_error("mpris.volume: no MPRIS media players found on session bus")
                    })?
                } else if s.starts_with("org.mpris.MediaPlayer2.") {
                    s.to_string()
                } else {
                    format!("{MPRIS_PREFIX}{s}")
                }
            } else {
                return Err(type_error(
                    "string, number, or null",
                    v.type_name_str(),
                    "mpris.volume",
                ));
            }
        }
        None => {
            let players = list_mpris_players()?;
            players.into_iter().next().ok_or_else(|| {
                runtime_error("mpris.volume: no MPRIS media players found on session bus")
            })?
        }
    };

    match level_arg {
        Some(v) => {
            if v.is_null() {
                let raw = get_player_property(&dest, "Volume")?;
                Ok(Value16::number(extract_double_from_property(&raw)))
            } else if let Some(level) = v.as_number() {
                let clamped = level.clamp(0.0, 1.0);
                set_player_property_double(&dest, "Volume", clamped)?;
                Ok(Value16::number(clamped))
            } else {
                Err(type_error(
                    "number or null",
                    v.type_name_str(),
                    "mpris.volume",
                ))
            }
        }
        None => {
            let raw = get_player_property(&dest, "Volume")?;
            Ok(Value16::number(extract_double_from_property(&raw)))
        }
    }
}

pub fn mpris_seek(args: &[Value16]) -> SharedResult<Value16> {
    let first_is_num = args.first().and_then(|v| v.as_number()).is_some();

    let (dest, offset_us) = if first_is_num {
        let n = args.first().unwrap().as_number().unwrap();
        let players = list_mpris_players()?;
        let dest = players.into_iter().next().ok_or_else(|| {
            runtime_error("mpris.seek: no MPRIS media players found on session bus")
        })?;
        (dest, (n * 1_000_000.0) as i64)
    } else {
        let dest = resolve_player(args, 0, "mpris.seek")?;
        let offset = match args.get(1) {
            Some(v) => v
                .as_number()
                .map(|n| (n * 1_000_000.0) as i64)
                .ok_or_else(|| type_error("number", v.type_name_str(), "mpris.seek"))?,
            None => {
                return Err(runtime_error(
                    "mpris.seek: offset_seconds argument required",
                ));
            }
        };
        (dest, offset)
    };

    if has_gdbus() {
        let output = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                &dest,
                "--object-path",
                MPRIS_PATH,
                "--method",
                &format!("{MPRIS_PLAYER_IFACE}.Seek"),
                &format!("{offset_us}"),
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: gdbus Seek failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!("mpris: Seek failed: {stderr}")));
        }
    } else {
        let output = Command::new("dbus-send")
            .args([
                "--session",
                "--print-reply",
                &format!("--dest={}", dest),
                MPRIS_PATH,
                &format!("{MPRIS_PLAYER_IFACE}.Seek"),
                &format!("int64:{offset_us}"),
            ])
            .output()
            .map_err(|e| runtime_error(format!("mpris: dbus-send Seek failed: {e}")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(runtime_error(format!("mpris: Seek failed: {stderr}")));
        }
    }
    Ok(Value16::boolean(true))
}
