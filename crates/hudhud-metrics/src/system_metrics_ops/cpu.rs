//! CPU count and usage query operations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn sys_cpu_count(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(unix)]
    {
        let cpus = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
        Ok(Value16::number(if cpus > 0 { cpus as f64 } else { 1.0 }))
    }

    #[cfg(not(unix))]
    {
        let cpus = std::env::var("NUMBER_OF_PROCESSORS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(1);
        Ok(Value16::number(if cpus > 0 { cpus as f64 } else { 1.0 }))
    }
}

pub fn sys_cpu_usage(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(target_os = "linux")]
    {
        let snap1 = read_cpu_stat();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let snap2 = read_cpu_stat();

        if let (Some((idle1, total1)), Some((idle2, total2))) = (snap1, snap2) {
            let idle_delta = idle2 - idle1;
            let total_delta = total2 - total1;
            if total_delta > 0 {
                let usage = (1.0 - idle_delta as f64 / total_delta as f64) * 100.0;
                return Ok(Value16::number((usage * 100.0).round() / 100.0));
            }
        }
        Ok(Value16::number(0.0))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(Value16::number(0.0))
    }
}

#[cfg(target_os = "linux")]
fn read_cpu_stat() -> Option<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;
    if !line.starts_with("cpu ") {
        return None;
    }
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    if fields.len() < 4 {
        return None;
    }
    let idle = fields[3] + fields.get(4).copied().unwrap_or(0);
    let total: u64 = fields.iter().sum();
    Some((idle, total))
}
