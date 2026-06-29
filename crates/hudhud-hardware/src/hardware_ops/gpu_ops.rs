//! GPU hardware detection — reads lspci output.

use std::collections::HashMap;
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn hw_gpu_info(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut gpus: Vec<Value16> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("lspci").args(["-v", "-nn"]).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut current_name = String::new();
                let mut current_driver = String::new();
                let mut current_memory_mb: f64 = 0.0;
                let mut in_vga = false;

                for line in text.lines() {
                    if !line.starts_with('\t') && !line.starts_with(' ') {
                        if in_vga && !current_name.is_empty() {
                            let mut gpu = hudhudscript_bytecode::ObjMap::default();
                            gpu.insert("name".to_string(), Value16::string(current_name.clone()));
                            gpu.insert(
                                "driver".to_string(),
                                Value16::string(current_driver.clone()),
                            );
                            gpu.insert("memory_mb".to_string(), Value16::number(current_memory_mb));
                            gpus.push(Value16::object(gpu));
                        }
                        current_name.clear();
                        current_driver.clear();
                        current_memory_mb = 0.0;

                        let lower = line.to_lowercase();
                        in_vga = lower.contains("vga")
                            || lower.contains("3d controller")
                            || lower.contains("display controller");
                        if in_vga {
                            if let Some((_addr, rest)) = line.split_once(": ") {
                                current_name = rest.trim().to_string();
                            }
                        }
                    } else if in_vga {
                        let trimmed = line.trim();
                        if let Some(rest) = trimmed.strip_prefix("Kernel driver in use:") {
                            current_driver = rest.trim().to_string();
                        }
                        if trimmed.contains("Memory") && trimmed.contains("prefetchable") {
                            if let Some(size_start) = trimmed.find("[size=") {
                                let after = &trimmed[size_start + 6..];
                                if let Some(end) = after.find(']') {
                                    let size_str = &after[..end];
                                    if let Some(mb_str) = size_str.strip_suffix('M') {
                                        if let Ok(mb) = mb_str.parse::<f64>() {
                                            if mb > current_memory_mb {
                                                current_memory_mb = mb;
                                            }
                                        }
                                    } else if let Some(gb_str) = size_str.strip_suffix('G') {
                                        if let Ok(gb) = gb_str.parse::<f64>() {
                                            let mb = gb * 1024.0;
                                            if mb > current_memory_mb {
                                                current_memory_mb = mb;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if in_vga && !current_name.is_empty() {
                    let mut gpu = hudhudscript_bytecode::ObjMap::default();
                    gpu.insert("name".to_string(), Value16::string(current_name));
                    gpu.insert("driver".to_string(), Value16::string(current_driver));
                    gpu.insert("memory_mb".to_string(), Value16::number(current_memory_mb));
                    gpus.push(Value16::object(gpu));
                }
            }
        }
    }

    Ok(Value16::array(gpus))
}
