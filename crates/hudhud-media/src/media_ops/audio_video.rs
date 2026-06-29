use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;

use super::util;

pub fn audio_info(args: &[Value16]) -> HudHudResult<Value16> {
    let path = util::require_str(args, 0, "media.audio_info")?;

    let output = util::run_cmd(
        std::process::Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-show_format")
            .arg("-show_streams")
            .arg("-of")
            .arg("json")
            .arg(&path),
        "media.audio_info",
    )?;
    let json_str = String::from_utf8_lossy(&output);
    parse_audio_json(&json_str)
}

fn parse_audio_json(json_str: &str) -> HudHudResult<Value16> {
    let mut m = hudhudscript_bytecode::ObjMap::default();
    if let Some(v) = util::extract_json_string(json_str, "format_name") {
        m.insert("format".to_string(), Value16::string(v));
    }
    if let Some(v) = util::extract_json_string(json_str, "duration") {
        if let Ok(d) = v.parse::<f64>() {
            m.insert("duration".to_string(), Value16::number(d));
        }
    }
    if let Some(v) = util::extract_json_string(json_str, "bit_rate") {
        if let Ok(b) = v.parse::<f64>() {
            m.insert("bit_rate".to_string(), Value16::number(b));
        }
    }
    Ok(Value16::object(m))
}

pub fn video_info(args: &[Value16]) -> HudHudResult<Value16> {
    let path = util::require_str(args, 0, "media.video_info")?;

    let output = util::run_cmd(
        std::process::Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-show_format")
            .arg("-show_streams")
            .arg("-of")
            .arg("json")
            .arg(&path),
        "media.video_info",
    )?;
    let json_str = String::from_utf8_lossy(&output);
    parse_video_json(&json_str)
}

fn parse_video_json(json_str: &str) -> HudHudResult<Value16> {
    let mut m = hudhudscript_bytecode::ObjMap::default();
    if let Some(v) = util::extract_json_string(json_str, "format_name") {
        m.insert("format".to_string(), Value16::string(v));
    }
    if let Some(v) = util::extract_json_string(json_str, "duration") {
        if let Ok(d) = v.parse::<f64>() {
            m.insert("duration".to_string(), Value16::number(d));
        }
    }
    if let Some(v) = util::extract_json_string(json_str, "width") {
        if let Ok(w) = v.parse::<f64>() {
            m.insert("width".to_string(), Value16::number(w));
        }
    }
    if let Some(v) = util::extract_json_string(json_str, "height") {
        if let Ok(h) = v.parse::<f64>() {
            m.insert("height".to_string(), Value16::number(h));
        }
    }
    if let Some(v) = util::extract_json_string(json_str, "bit_rate") {
        if let Ok(b) = v.parse::<f64>() {
            m.insert("bit_rate".to_string(), Value16::number(b));
        }
    }
    Ok(Value16::object(m))
}

pub fn transcode(args: &[Value16]) -> HudHudResult<Value16> {
    let input_path = util::require_str(args, 0, "media.transcode")?;
    let output_path = util::require_str(args, 1, "media.transcode")?;
    let format = util::require_str(args, 2, "media.transcode")?;

    let output = util::run_cmd(
        std::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(&input_path)
            .arg("-f")
            .arg(&format)
            .arg(&output_path),
        "media.transcode",
    )?;
    let _ = output;
    Ok(Value16::string(output_path))
}
