use super::*;

pub(super) fn require_str(args: &[Value16], idx: usize, method: &str) -> HudHudResult<String> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| type_error("string", v.type_name_str(), method)),
        None => Err(type_error("string", "missing", method)),
    }
}

pub(super) fn require_num(args: &[Value16], idx: usize, method: &str) -> HudHudResult<f64> {
    match args.get(idx) {
        Some(v) => v
            .as_number()
            .ok_or_else(|| type_error("number", v.type_name_str(), method)),
        None => Err(type_error("number", "missing", method)),
    }
}

pub(super) fn run_cmd(cmd: &mut Command, context: &str) -> HudHudResult<Vec<u8>> {
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("{}: {}", context, e)))?;

    if !output.status.success() {
        return Err(runtime_error(format!(
            "{} failed: {}",
            context,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(output.stdout)
}

pub(super) fn file_size(path: &str) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

pub fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = json.find(&pattern)?;
    let after_key = &json[idx + pattern.len()..];
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let trimmed = after_colon.trim_start();
    if let Some(content) = trimmed.strip_prefix('"') {
        let end = content.find('"')?;
        Some(content[..end].to_string())
    } else {
        None
    }
}

pub fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\"", key);
    let idx = json.find(&pattern)?;
    let after_key = &json[idx + pattern.len()..];
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let trimmed = after_colon.trim_start();
    let end = trimmed
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(trimmed.len());
    trimmed[..end].parse::<f64>().ok()
}
