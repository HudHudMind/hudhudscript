use super::*;

pub fn browser_tabs(args: &[Value16]) -> HudHudResult<Value16> {
    let name = resolve_browser_name(args, 0);

    if name == "firefox" {
        return firefox_tabs();
    }
    if is_chromium_based(&name) {
        return chromium_tabs(&name);
    }
    if let Ok(v) = firefox_tabs() {
        if let Some(a) = v.as_array() {
            if !a.is_empty() {
                return Ok(v);
            }
        }
    }
    chromium_tabs(&name)
}

fn firefox_tabs() -> HudHudResult<Value16> {
    let profile = match firefox_profile_dir() {
        Some(p) => p,
        None => return Ok(Value16::array(Vec::new())),
    };
    let recovery = profile.join("sessionstore-backups/recovery.jsonlz4");
    let recovery_json = profile.join("sessionstore-backups/recovery.js");
    let session_file = profile.join("sessionstore.js");

    for path in &[&recovery_json, &session_file] {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                return parse_firefox_session_tabs(&content);
            }
        }
    }

    if recovery.exists() {
        let output = Command::new("python3")
            .arg("-c")
            .arg(format!(
                "import lz4.block, sys; f=open('{}','rb'); f.read(8); \
                 print(lz4.block.decompress(f.read()).decode('utf-8', errors='replace'))",
                recovery.to_string_lossy()
            ))
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let content = String::from_utf8_lossy(&out.stdout).to_string();
                return parse_firefox_session_tabs(&content);
            }
        }
    }

    Ok(Value16::array(Vec::new()))
}

fn parse_firefox_session_tabs(content: &str) -> HudHudResult<Value16> {
    let mut results: Vec<Value16> = Vec::new();
    let mut i = 0;
    while i < content.len() {
        if let Some(pos) = content[i..].find("\"url\"") {
            let abs_pos = i + pos;
            if let Some(url) = extract_json_string(content, abs_pos + 5) {
                if url.starts_with("http://") || url.starts_with("https://") {
                    let search_start = abs_pos.saturating_sub(300);
                    let context = &content[search_start..abs_pos];
                    let title = if let Some(t_pos) = context.rfind("\"title\"") {
                        extract_json_string(context, t_pos + 7).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let mut entry = hudhudscript_bytecode::ObjMap::default();
                    entry.insert("title".to_string(), Value16::string(title));
                    entry.insert("url".to_string(), Value16::string(url));
                    results.push(Value16::object(entry));
                }
            }
            i = abs_pos + 5;
        } else {
            break;
        }
    }
    let mut seen = std::collections::HashSet::new();
    let mut unique: Vec<Value16> = Vec::new();
    for item in results {
        if let Some(obj) = item.as_object() {
            if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
                if seen.insert(url.to_string()) {
                    unique.push(item.clone());
                }
            }
        }
    }
    Ok(Value16::array(unique))
}

fn chromium_tabs(browser_name: &str) -> HudHudResult<Value16> {
    let config = match chromium_config_dir(browser_name) {
        Some(c) => c,
        None => return Ok(Value16::array(Vec::new())),
    };

    let prefs_path = config.join("Default/Preferences");
    if prefs_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&prefs_path) {
            let mut results: Vec<Value16> = Vec::new();
            let mut i = 0;
            while i < content.len() {
                if let Some(pos) = content[i..].find("\"url\"") {
                    let abs_pos = i + pos;
                    if let Some(url) = extract_json_string(&content, abs_pos + 5) {
                        if url.starts_with("http://") || url.starts_with("https://") {
                            let mut entry = hudhudscript_bytecode::ObjMap::default();
                            entry.insert("title".to_string(), Value16::string(String::new()));
                            entry.insert("url".to_string(), Value16::string(url));
                            results.push(Value16::object(entry));
                        }
                    }
                    i = abs_pos + 5;
                } else {
                    break;
                }
            }
            if !results.is_empty() {
                return Ok(Value16::array(results));
            }
        }
    }

    Ok(Value16::array(Vec::new()))
}
