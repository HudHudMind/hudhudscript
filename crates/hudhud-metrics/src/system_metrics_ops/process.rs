//! Process listing query operation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;

use super::utils::runtime_error;

pub fn sys_processes(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(target_os = "linux")]
    {
        let mut procs = Vec::new();
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as f64;
        let clock_ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;

        let uptime_secs = std::fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|c| {
                c.split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or(0.0);

        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if let Ok(pid) = name_str.parse::<u64>() {
                    let stat_path = format!("/proc/{}/stat", pid);
                    if let Ok(stat_content) = std::fs::read_to_string(&stat_path) {
                        if let Some(comm_start) = stat_content.find('(') {
                            if let Some(comm_end) = stat_content.rfind(')') {
                                let proc_name = stat_content[comm_start + 1..comm_end].to_string();
                                let rest = &stat_content[comm_end + 2..];
                                let fields: Vec<&str> = rest.split_whitespace().collect();
                                let utime: f64 =
                                    fields.get(11).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                                let stime: f64 =
                                    fields.get(12).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                                let starttime: f64 =
                                    fields.get(19).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                                let rss: f64 =
                                    fields.get(21).and_then(|s| s.parse().ok()).unwrap_or(0.0);

                                let total_time = utime + stime;
                                let elapsed = uptime_secs - (starttime / clock_ticks);
                                let cpu_percent = if elapsed > 0.0 {
                                    ((total_time / clock_ticks) / elapsed) * 100.0
                                } else {
                                    0.0
                                };
                                let memory_kb = (rss * page_size) / 1024.0;

                                let mut p = HashMap::new();
                                p.insert("pid".to_string(), Value16::number(pid as f64));
                                p.insert("name".to_string(), Value16::string(proc_name));
                                p.insert(
                                    "cpu_percent".to_string(),
                                    Value16::number((cpu_percent * 100.0).round() / 100.0),
                                );
                                p.insert("memory_kb".to_string(), Value16::number(memory_kb));
                                procs.push(Value16::object(p));
                            }
                        }
                    }
                }
            }
        }
        Ok(Value16::array(procs))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(runtime_error(
            "system.processes: only supported on Linux (requires /proc filesystem)",
        ))
    }
}
