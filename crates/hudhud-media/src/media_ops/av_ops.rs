use super::*;

pub fn audio_info(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "media.audio_info")?;

    let output = run_cmd(
        Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(&path),
        "media.audio_info",
    )?;
    let text = String::from_utf8_lossy(&output);
    parse_audio_json(&text)
}

fn parse_audio_json(json_str: &str) -> HudHudResult<Value16> {
    let duration = extract_json_string(json_str, "duration")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let format = extract_json_string(json_str, "format_name").unwrap_or_default();
    let bit_rate = extract_json_string(json_str, "bit_rate")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let channels = extract_json_number(json_str, "channels").unwrap_or(0.0);
    let sample_rate = extract_json_string(json_str, "sample_rate")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("duration".to_string(), Value16::number(duration));
    m.insert("format".to_string(), Value16::string(format));
    m.insert("bitrate".to_string(), Value16::number(bit_rate));
    m.insert("channels".to_string(), Value16::number(channels));
    m.insert("sample_rate".to_string(), Value16::number(sample_rate));
    Ok(Value16::object(m))
}

pub fn video_info(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "media.video_info")?;

    let output = run_cmd(
        Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(&path),
        "media.video_info",
    )?;
    let text = String::from_utf8_lossy(&output);
    parse_video_json(&text)
}

fn parse_video_json(json_str: &str) -> HudHudResult<Value16> {
    let duration = extract_json_string(json_str, "duration")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let width = extract_json_number(json_str, "width").unwrap_or(0.0);
    let height = extract_json_number(json_str, "height").unwrap_or(0.0);
    let format = extract_json_string(json_str, "format_name").unwrap_or_default();
    let codec = extract_json_string(json_str, "codec_name").unwrap_or_default();

    let fps = extract_json_string(json_str, "r_frame_rate")
        .and_then(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                let num: f64 = parts[0].parse().ok()?;
                let den: f64 = parts[1].parse().ok()?;
                if den > 0.0 {
                    Some(num / den)
                } else {
                    None
                }
            } else {
                s.parse::<f64>().ok()
            }
        })
        .unwrap_or(0.0);

    let mut m = hudhudscript_bytecode::ObjMap::default();
    m.insert("duration".to_string(), Value16::number(duration));
    m.insert("width".to_string(), Value16::number(width));
    m.insert("height".to_string(), Value16::number(height));
    m.insert("format".to_string(), Value16::string(format));
    m.insert("codec".to_string(), Value16::string(codec));
    m.insert("fps".to_string(), Value16::number(fps));
    Ok(Value16::object(m))
}

pub fn transcode(args: &[Value16]) -> HudHudResult<Value16> {
    let input = require_str(args, 0, "media.transcode")?;
    let output = require_str(args, 1, "media.transcode")?;

    let mut extra_args: Vec<String> = Vec::new();
    if let Some(opts) = args.get(2).and_then(|v| v.as_object()) {
        if let Some(c) = opts.get("codec").and_then(|v| v.as_str()) {
            extra_args.push("-c:v".to_string());
            extra_args.push(c.to_string());
        }
        if let Some(ac) = opts.get("audio_codec").and_then(|v| v.as_str()) {
            extra_args.push("-c:a".to_string());
            extra_args.push(ac.to_string());
        }
        if let Some(b) = opts.get("bitrate").and_then(|v| v.as_str()) {
            extra_args.push("-b:v".to_string());
            extra_args.push(b.to_string());
        }
        if let Some(ab) = opts.get("audio_bitrate").and_then(|v| v.as_str()) {
            extra_args.push("-b:a".to_string());
            extra_args.push(ab.to_string());
        }
        if let Some(crf) = opts.get("crf").and_then(|v| v.as_number()) {
            extra_args.push("-crf".to_string());
            extra_args.push((crf as i64).to_string());
        }
        if let Some(preset) = opts.get("preset").and_then(|v| v.as_str()) {
            extra_args.push("-preset".to_string());
            extra_args.push(preset.to_string());
        }
    }

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-i").arg(&input);
    for a in &extra_args {
        cmd.arg(a);
    }
    cmd.arg(&output);

    run_cmd(&mut cmd, "media.transcode")?;
    Ok(Value16::string(output))
}

pub fn thumbnail(args: &[Value16]) -> HudHudResult<Value16> {
    let video_path = require_str(args, 0, "media.thumbnail")?;
    let output_path = require_str(args, 1, "media.thumbnail")?;
    let time_seconds = require_num(args, 2, "media.thumbnail")?;

    run_cmd(
        Command::new("ffmpeg")
            .arg("-y")
            .arg("-ss")
            .arg(format!("{}", time_seconds))
            .arg("-i")
            .arg(&video_path)
            .arg("-frames:v")
            .arg("1")
            .arg(&output_path),
        "media.thumbnail",
    )?;
    Ok(Value16::string(output_path))
}
