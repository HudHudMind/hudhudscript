//! Memory info query operation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;

pub fn sys_memory(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut total: u64 = 0;
            let mut free: u64 = 0;
            let mut available: u64 = 0;
            let mut buffers: u64 = 0;
            let mut cached: u64 = 0;

            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let val: u64 = parts[1].parse().unwrap_or(0) * 1024; // kB -> bytes
                    match parts[0] {
                        "MemTotal:" => total = val,
                        "MemFree:" => free = val,
                        "MemAvailable:" => available = val,
                        "Buffers:" => buffers = val,
                        "Cached:" => cached = val,
                        _ => {}
                    }
                }
            }

            if available == 0 {
                available = free + buffers + cached;
            }
            let used = total.saturating_sub(available);

            let mut obj = hudhudscript_bytecode::ObjMap::default();
            obj.insert("total".to_string(), Value16::number(total as f64));
            obj.insert("used".to_string(), Value16::number(used as f64));
            obj.insert("free".to_string(), Value16::number(free as f64));
            obj.insert("available".to_string(), Value16::number(available as f64));
            return Ok(Value16::object(obj));
        }
    }

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("total".to_string(), Value16::number(0.0));
    obj.insert("used".to_string(), Value16::number(0.0));
    obj.insert("free".to_string(), Value16::number(0.0));
    obj.insert("available".to_string(), Value16::number(0.0));
    Ok(Value16::object(obj))
}
