//! Storage hardware detection — reads lsblk output.

use std::collections::HashMap;
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn hw_disk_info(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut disks: Vec<Value16> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("lsblk")
            .args(["-d", "-b", "-n", "-o", "NAME,SIZE,MODEL,ROTA,TYPE"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let device = format!("/dev/{}", parts[0]);
                        let size_bytes: f64 = parts[1].parse().unwrap_or(0.0);
                        let size_gb = (size_bytes / 1_073_741_824.0 * 100.0).round() / 100.0;

                        let dev_type_field = parts.last().copied().unwrap_or("disk");
                        let rota_field = if parts.len() >= 5 {
                            parts[parts.len() - 2]
                        } else {
                            parts.last().copied().unwrap_or("0")
                        };

                        if dev_type_field != "disk" {
                            continue;
                        }

                        let disk_type = if rota_field == "0" { "SSD" } else { "HDD" };

                        let model = if parts.len() > 4 {
                            parts[2..parts.len() - 2].join(" ")
                        } else {
                            String::from("unknown")
                        };

                        let mut disk = HashMap::new();
                        disk.insert("device".to_string(), Value16::string(device));
                        disk.insert("size_gb".to_string(), Value16::number(size_gb));
                        disk.insert("model".to_string(), Value16::string(model));
                        disk.insert("type".to_string(), Value16::string(disk_type.to_string()));
                        disks.push(Value16::object(disk));
                    }
                }
            }
        }
    }

    Ok(Value16::array(disks))
}
