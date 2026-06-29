//! CPU hardware detection — reads /proc/cpuinfo and uname.

use std::collections::HashMap;
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn hw_cpu_info(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut model = String::from("unknown");
    let mut cores: u64 = 0;
    let mut threads: u64 = 0;
    let mut frequency_mhz: f64 = 0.0;
    let mut architecture = String::from("unknown");

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            let mut seen_processors: u64 = 0;
            let mut core_ids = std::collections::HashSet::new();

            for line in content.lines() {
                let line = line.trim();
                if let Some((key, val)) = line.split_once(':') {
                    let key = key.trim();
                    let val = val.trim();
                    match key {
                        "model name" => {
                            if model == "unknown" {
                                model = val.to_string();
                            }
                        }
                        "processor" => {
                            seen_processors += 1;
                        }
                        "core id" => {
                            core_ids.insert(val.to_string());
                        }
                        "cpu MHz" => {
                            if let Ok(mhz) = val.parse::<f64>() {
                                if frequency_mhz == 0.0 {
                                    frequency_mhz = (mhz * 100.0).round() / 100.0;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            threads = seen_processors;
            cores = if core_ids.is_empty() {
                threads
            } else {
                core_ids.len() as u64
            };
        }

        if let Ok(output) = Command::new("uname").arg("-m").output() {
            if output.status.success() {
                architecture = String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("model".to_string(), Value16::string(model));
    obj.insert("cores".to_string(), Value16::number(cores as f64));
    obj.insert("threads".to_string(), Value16::number(threads as f64));
    obj.insert("frequency_mhz".to_string(), Value16::number(frequency_mhz));
    obj.insert("architecture".to_string(), Value16::string(architecture));
    Ok(Value16::object(obj))
}
