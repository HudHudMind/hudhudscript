use super::*;

pub(super) fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

pub(super) fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

pub(super) fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "~".to_string())
}

pub(super) fn resolve_browser_name(args: &[Value16], index: usize) -> String {
    match args.get(index).and_then(|v| v.as_str()) {
        Some(s) if !s.is_empty() => s.to_lowercase(),
        _ => detect_default_browser_name(),
    }
}

pub(super) fn detect_default_browser_name() -> String {
    if let Ok(output) = Command::new("xdg-settings")
        .args(["get", "default-web-browser"])
        .output()
    {
        if output.status.success() {
            let desktop = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase();
            if desktop.contains("firefox") {
                return "firefox".to_string();
            }
            if desktop.contains("chrom") {
                return "chromium".to_string();
            }
            if desktop.contains("google") {
                return "chrome".to_string();
            }
            if desktop.contains("brave") {
                return "brave".to_string();
            }
            if desktop.contains("edge") {
                return "edge".to_string();
            }
            let name = desktop.trim_end_matches(".desktop").to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "firefox".to_string()
}

pub(super) fn is_chromium_based(name: &str) -> bool {
    matches!(
        name,
        "chrome" | "chromium" | "google-chrome" | "brave" | "edge" | "vivaldi" | "opera"
    )
}

pub(super) fn firefox_profile_dir() -> Option<PathBuf> {
    let home = home_dir();
    let mozilla_dir = Path::new(&home).join(".mozilla/firefox");
    let profiles_ini = mozilla_dir.join("profiles.ini");
    if let Ok(content) = std::fs::read_to_string(&profiles_ini) {
        let mut current_path: Option<String> = None;
        let mut current_is_relative = true;
        let mut current_is_default = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                if current_is_default {
                    if let Some(ref p) = current_path {
                        return if current_is_relative {
                            Some(mozilla_dir.join(p))
                        } else {
                            Some(PathBuf::from(p))
                        };
                    }
                }
                current_path = None;
                current_is_relative = true;
                current_is_default = false;
            } else if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Path" => current_path = Some(value.trim().to_string()),
                    "IsRelative" => current_is_relative = value.trim() == "1",
                    "Default" => {
                        if value.trim() == "1" {
                            current_is_default = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        if current_is_default {
            if let Some(ref p) = current_path {
                return if current_is_relative {
                    Some(mozilla_dir.join(p))
                } else {
                    Some(PathBuf::from(p))
                };
            }
        }
        let mut first_path: Option<String> = None;
        let mut first_relative = true;
        for line in content.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Path" if first_path.is_none() => {
                        first_path = Some(value.trim().to_string());
                    }
                    "IsRelative" if first_path.is_some() && first_relative => {
                        first_relative = value.trim() == "1";
                    }
                    _ => {}
                }
            }
        }
        if let Some(ref p) = first_path {
            return if first_relative {
                Some(mozilla_dir.join(p))
            } else {
                Some(PathBuf::from(p))
            };
        }
    }
    if let Ok(entries) = std::fs::read_dir(&mozilla_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".default") || name.ends_with(".default-release") {
                return Some(entry.path());
            }
        }
    }
    None
}

pub(super) fn chromium_config_dir(browser_name: &str) -> Option<PathBuf> {
    let home = home_dir();
    let config = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));
    let dir_name = match browser_name {
        "chrome" | "google-chrome" => "google-chrome",
        "chromium" => "chromium",
        "brave" => "BraveSoftware/Brave-Browser",
        "edge" => "microsoft-edge",
        "vivaldi" => "vivaldi",
        "opera" => "opera",
        _ => "chromium",
    };
    let path = Path::new(&config).join(dir_name);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}
