//! Memory hardware detection — reads /proc/meminfo and dmidecode.

use std::collections::HashMap;
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn hw_memory_info(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut total_mb: f64 = 0.0;
    let mut type_hint = String::from("unknown");

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    let rest = rest.trim();
                    if let Some(kb_str) =
                        rest.strip_suffix("kB").or_else(|| rest.strip_suffix("KB"))
                    {
                        if let Ok(kb) = kb_str.trim().parse::<u64>() {
                            total_mb = (kb as f64 / 1024.0 * 100.0).round() / 100.0;
                        }
                    }
                    break;
                }
            }
        }

        if let Ok(output) = Command::new("dmidecode").args(["-t", "memory"]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let line = line.trim();
                    if let Some(rest) = line.strip_prefix("Type:") {
                        let t = rest.trim();
                        if t != "Unknown" && !t.is_empty() {
                            type_hint = t.to_string();
                            break;
                        }
                    }
                }
            }
        }
    }

    let mut obj = HashMap::new();
    obj.insert("total_mb".to_string(), Value16::number(total_mb));
    obj.insert("type_hint".to_string(), Value16::string(type_hint));
    Ok(Value16::object(obj))
}
