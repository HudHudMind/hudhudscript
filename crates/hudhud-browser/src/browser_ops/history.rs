use super::*;

pub fn browser_history(args: &[Value16]) -> HudHudResult<Value16> {
    let name = resolve_browser_name(args, 0);
    let count = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n as usize)
        .unwrap_or(50);

    if name == "firefox" {
        return firefox_history(count);
    }
    if is_chromium_based(&name) {
        return chromium_history(&name, count);
    }
    if let Ok(v) = firefox_history(count) {
        if let Some(a) = v.as_array() {
            if !a.is_empty() {
                return Ok(v);
            }
        }
    }
    chromium_history(&name, count)
}

fn firefox_history(count: usize) -> HudHudResult<Value16> {
    let profile = firefox_profile_dir()
        .ok_or_else(|| runtime_error("browser.history: Firefox profile not found"))?;
    let db_path = profile.join("places.sqlite");
    if !db_path.exists() {
        return Ok(Value16::array(Vec::new()));
    }
    let query = format!(
        "SELECT p.title, p.url, h.visit_date FROM moz_historyvisits h \
         JOIN moz_places p ON h.place_id = p.id \
         ORDER BY h.visit_date DESC LIMIT {};",
        count
    );
    let output = Command::new("sqlite3")
        .arg(db_path.to_string_lossy().as_ref())
        .arg(&query)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let results: Vec<Value16> = text
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(3, '|').collect();
                    if parts.len() >= 2 {
                        let mut entry = hudhudscript_bytecode::ObjMap::default();
                        entry.insert("title".to_string(), Value16::string(parts[0].to_string()));
                        entry.insert("url".to_string(), Value16::string(parts[1].to_string()));
                        let visit_time = parts
                            .get(2)
                            .and_then(|s| s.parse::<f64>().ok())
                            .map(|us| us / 1_000_000.0)
                            .unwrap_or(0.0);
                        entry.insert("visit_time".to_string(), Value16::number(visit_time));
                        Some(Value16::object(entry))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(Value16::array(results))
        }
        _ => Ok(Value16::array(Vec::new())),
    }
}

fn chromium_history(browser_name: &str, count: usize) -> HudHudResult<Value16> {
    let config = chromium_config_dir(browser_name).ok_or_else(|| {
        runtime_error(format!(
            "browser.history: {} config directory not found",
            browser_name
        ))
    })?;
    let db_path = config.join("Default/History");
    if !db_path.exists() {
        return Ok(Value16::array(Vec::new()));
    }
    let tmp_path = std::env::temp_dir().join("hudhud_browser_history.sqlite");
    if std::fs::copy(&db_path, &tmp_path).is_err() {
        return Ok(Value16::array(Vec::new()));
    }
    let query = format!(
        "SELECT title, url, last_visit_time FROM urls ORDER BY last_visit_time DESC LIMIT {};",
        count
    );
    let output = Command::new("sqlite3")
        .arg(tmp_path.to_string_lossy().as_ref())
        .arg(&query)
        .output();
    let _ = std::fs::remove_file(&tmp_path);
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let results: Vec<Value16> = text
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.splitn(3, '|').collect();
                    if parts.len() >= 2 {
                        let mut entry = hudhudscript_bytecode::ObjMap::default();
                        entry.insert("title".to_string(), Value16::string(parts[0].to_string()));
                        entry.insert("url".to_string(), Value16::string(parts[1].to_string()));
                        let visit_time = parts
                            .get(2)
                            .and_then(|s| s.parse::<f64>().ok())
                            .map(|us| (us - 11_644_473_600_000_000.0) / 1_000_000.0)
                            .unwrap_or(0.0);
                        entry.insert("visit_time".to_string(), Value16::number(visit_time));
                        Some(Value16::object(entry))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(Value16::array(results))
        }
        _ => Ok(Value16::array(Vec::new())),
    }
}
