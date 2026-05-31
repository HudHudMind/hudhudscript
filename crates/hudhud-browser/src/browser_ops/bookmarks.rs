use super::*;

pub fn browser_bookmarks(args: &[Value16]) -> HudHudResult<Value16> {
    let name = resolve_browser_name(args, 0);

    if name == "firefox" {
        return firefox_bookmarks();
    }
    if is_chromium_based(&name) {
        return chromium_bookmarks(&name);
    }
    if let Ok(v) = firefox_bookmarks() {
        if let Some(a) = v.as_array() {
            if !a.is_empty() {
                return Ok(v);
            }
        }
    }
    chromium_bookmarks(&name)
}

fn firefox_bookmarks() -> HudHudResult<Value16> {
    let profile = firefox_profile_dir()
        .ok_or_else(|| runtime_error("browser.bookmarks: Firefox profile not found"))?;
    let backup_dir = profile.join("bookmarkbackups");
    if backup_dir.exists() {
        if let Ok(mut entries) = std::fs::read_dir(&backup_dir) {
            let mut files: Vec<PathBuf> = entries
                .by_ref()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .map(|e| e == "jsonlz4" || e == "json")
                        .unwrap_or(false)
                })
                .collect();
            files.sort();
            if let Some(latest) = files.last() {
                if latest.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(latest) {
                        return parse_firefox_bookmarks_json(&content);
                    }
                }
            }
        }
    }
    let db_path = profile.join("places.sqlite");
    if db_path.exists() {
        let output = Command::new("sqlite3")
            .arg(db_path.to_string_lossy().as_ref())
            .arg("SELECT b.title, p.url, b.parent FROM moz_bookmarks b JOIN moz_places p ON b.fk = p.id WHERE b.type = 1 AND b.title IS NOT NULL LIMIT 500;")
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut results: Vec<Value16> = Vec::new();
                for line in text.lines() {
                    let parts: Vec<&str> = line.splitn(3, '|').collect();
                    if parts.len() >= 2 {
                        let mut entry = HashMap::new();
                        entry.insert("title".to_string(), Value16::string(parts[0].to_string()));
                        entry.insert("url".to_string(), Value16::string(parts[1].to_string()));
                        entry.insert(
                            "folder".to_string(),
                            Value16::string(parts.get(2).unwrap_or(&"").to_string()),
                        );
                        results.push(Value16::object(entry));
                    }
                }
                return Ok(Value16::array(results));
            }
        }
    }
    Ok(Value16::array(Vec::new()))
}

fn parse_firefox_bookmarks_json(content: &str) -> HudHudResult<Value16> {
    let mut results: Vec<Value16> = Vec::new();
    extract_bookmarks_from_json(content, &mut results);
    Ok(Value16::array(results))
}

fn extract_bookmarks_from_json(content: &str, results: &mut Vec<Value16>) {
    let mut i = 0;
    let bytes = content.as_bytes();
    while i < bytes.len() {
        if let Some(pos) = content[i..].find("\"uri\"") {
            let abs_pos = i + pos;
            if let Some(uri) = extract_json_string(content, abs_pos + 5) {
                let search_start = abs_pos.saturating_sub(500);
                let context = &content[search_start..abs_pos + 5];
                let title = if let Some(t_pos) = context.rfind("\"title\"") {
                    extract_json_string(context, t_pos + 7).unwrap_or_default()
                } else {
                    String::new()
                };
                if !uri.is_empty() && (uri.starts_with("http://") || uri.starts_with("https://")) {
                    let mut entry = HashMap::new();
                    entry.insert("title".to_string(), Value16::string(title));
                    entry.insert("url".to_string(), Value16::string(uri));
                    entry.insert("folder".to_string(), Value16::string(String::new()));
                    results.push(Value16::object(entry));
                }
            }
            i = abs_pos + 5;
        } else {
            break;
        }
    }
}

pub fn extract_json_string(content: &str, after_key: usize) -> Option<String> {
    let rest = content.get(after_key..)?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut result = String::new();
    let mut chars = rest.chars();
    loop {
        match chars.next() {
            Some('\\') => {
                if let Some(c) = chars.next() {
                    result.push(c);
                }
            }
            Some('"') => return Some(result),
            Some(c) => result.push(c),
            None => return None,
        }
    }
}

fn chromium_bookmarks(browser_name: &str) -> HudHudResult<Value16> {
    let config = chromium_config_dir(browser_name).ok_or_else(|| {
        runtime_error(format!(
            "browser.bookmarks: {} config directory not found",
            browser_name
        ))
    })?;
    let bookmarks_file = config.join("Default/Bookmarks");
    if !bookmarks_file.exists() {
        return Ok(Value16::array(Vec::new()));
    }
    let content = std::fs::read_to_string(&bookmarks_file)
        .map_err(|e| runtime_error(format!("browser.bookmarks: cannot read file: {}", e)))?;

    let mut results: Vec<Value16> = Vec::new();
    extract_chromium_bookmarks(&content, &mut results);
    Ok(Value16::array(results))
}

fn extract_chromium_bookmarks(content: &str, results: &mut Vec<Value16>) {
    let mut i = 0;
    while i < content.len() {
        if let Some(pos) = content[i..].find("\"url\"") {
            let abs_pos = i + pos;
            if let Some(url) = extract_json_string(content, abs_pos + 5) {
                let search_start = abs_pos.saturating_sub(500);
                let context = &content[search_start..std::cmp::min(abs_pos + 500, content.len())];
                let name = if let Some(n_pos) = context.rfind("\"name\"") {
                    extract_json_string(context, n_pos + 6).unwrap_or_default()
                } else {
                    String::new()
                };
                if url.starts_with("http://") || url.starts_with("https://") {
                    let mut entry = HashMap::new();
                    entry.insert("title".to_string(), Value16::string(name));
                    entry.insert("url".to_string(), Value16::string(url));
                    entry.insert("folder".to_string(), Value16::string(String::new()));
                    results.push(Value16::object(entry));
                }
            }
            i = abs_pos + 5;
        } else {
            break;
        }
    }
}
