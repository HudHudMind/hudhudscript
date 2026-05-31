//! Audio and display hardware detection — reads /proc/asound/cards, aplay, xrandr, /sys/class/drm, lspci.

use std::collections::HashMap;
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn hw_audio_devices(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut devices: Vec<Value16> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/asound/cards") {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
                {
                    let name = if let Some((_before, after)) = trimmed.split_once(" - ") {
                        after.trim().to_string()
                    } else {
                        trimmed.to_string()
                    };

                    let dev_type = if name.to_lowercase().contains("hdmi")
                        || name.to_lowercase().contains("display")
                    {
                        "hdmi"
                    } else {
                        "analog"
                    };

                    let mut dev = HashMap::new();
                    dev.insert("name".to_string(), Value16::string(name));
                    dev.insert("type".to_string(), Value16::string(dev_type.to_string()));
                    devices.push(Value16::object(dev));
                }
            }
        }

        if devices.is_empty() {
            if let Ok(output) = Command::new("aplay").arg("-l").output() {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    for line in text.lines() {
                        if line.starts_with("card ") {
                            let name = line.to_string();
                            let dev_type = if name.to_lowercase().contains("hdmi") {
                                "hdmi"
                            } else {
                                "analog"
                            };
                            let mut dev = HashMap::new();
                            dev.insert("name".to_string(), Value16::string(name));
                            dev.insert("type".to_string(), Value16::string(dev_type.to_string()));
                            devices.push(Value16::object(dev));
                        }
                    }
                }
            }
        }
    }

    Ok(Value16::array(devices))
}

pub fn hw_display_info(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut resolution = String::from("unknown");
    let mut driver = String::from("unknown");

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("xrandr").arg("--current").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.contains('*') {
                        let trimmed = line.trim();
                        if let Some(res) = trimmed.split_whitespace().next() {
                            resolution = res.to_string();
                            break;
                        }
                    }
                }
            }
        }

        if resolution == "unknown" {
            let drm_dir = std::path::Path::new("/sys/class/drm");
            if let Ok(entries) = std::fs::read_dir(drm_dir) {
                for entry in entries.flatten() {
                    let modes_path = entry.path().join("modes");
                    if let Ok(content) = std::fs::read_to_string(&modes_path) {
                        if let Some(first_mode) = content.lines().next() {
                            let mode = first_mode.trim();
                            if !mode.is_empty() {
                                resolution = mode.to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new("lspci").args(["-k"]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut in_vga = false;
                for line in text.lines() {
                    if !line.starts_with('\t') && !line.starts_with(' ') {
                        let lower = line.to_lowercase();
                        in_vga = lower.contains("vga")
                            || lower.contains("3d controller")
                            || lower.contains("display controller");
                    } else if in_vga {
                        let trimmed = line.trim();
                        if let Some(rest) = trimmed.strip_prefix("Kernel driver in use:") {
                            driver = rest.trim().to_string();
                            break;
                        }
                    }
                }
            }
        }
    }

    let mut obj = HashMap::new();
    obj.insert("resolution".to_string(), Value16::string(resolution));
    obj.insert("driver".to_string(), Value16::string(driver));
    Ok(Value16::object(obj))
}
