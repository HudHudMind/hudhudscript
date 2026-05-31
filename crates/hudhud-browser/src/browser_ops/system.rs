use super::*;

pub fn browser_default_browser(_args: &[Value16]) -> HudHudResult<Value16> {
    Ok(Value16::string(detect_default_browser_name()))
}

pub fn browser_installed_browsers(_args: &[Value16]) -> HudHudResult<Value16> {
    let candidates: &[(&str, &[&str])] = &[
        (
            "firefox",
            &[
                "/usr/bin/firefox",
                "/snap/bin/firefox",
                "/usr/lib/firefox/firefox",
            ],
        ),
        (
            "chromium",
            &[
                "/usr/bin/chromium",
                "/usr/bin/chromium-browser",
                "/snap/bin/chromium",
            ],
        ),
        (
            "google-chrome",
            &[
                "/usr/bin/google-chrome",
                "/usr/bin/google-chrome-stable",
                "/opt/google/chrome/google-chrome",
            ],
        ),
        (
            "brave",
            &[
                "/usr/bin/brave-browser",
                "/usr/bin/brave",
                "/snap/bin/brave",
            ],
        ),
        (
            "edge",
            &["/usr/bin/microsoft-edge", "/usr/bin/microsoft-edge-stable"],
        ),
        ("vivaldi", &["/usr/bin/vivaldi", "/usr/bin/vivaldi-stable"]),
        ("opera", &["/usr/bin/opera"]),
    ];

    let mut results: Vec<Value16> = Vec::new();
    for (name, paths) in candidates {
        for path in *paths {
            if Path::new(path).exists() {
                let version = get_browser_version(path);
                let mut entry = HashMap::new();
                entry.insert("name".to_string(), Value16::string(name.to_string()));
                entry.insert("path".to_string(), Value16::string(path.to_string()));
                entry.insert("version".to_string(), Value16::string(version));
                results.push(Value16::object(entry));
                break;
            }
        }
    }
    Ok(Value16::array(results))
}

fn get_browser_version(path: &str) -> String {
    if let Ok(output) = Command::new(path).arg("--version").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}
