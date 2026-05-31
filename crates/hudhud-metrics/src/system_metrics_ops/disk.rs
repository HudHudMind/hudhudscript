//! Disk usage query operation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;

use super::utils::runtime_error;

pub fn sys_disk_usage(args: &[Value16]) -> HudHudResult<Value16> {
    let path = args
        .first()
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "/".to_string());

    #[cfg(unix)]
    {
        use std::ffi::CString;
        let c_path = CString::new(path.as_str())
            .map_err(|e| runtime_error(format!("Invalid path: {}", e)))?;

        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            let ret = libc::statvfs(c_path.as_ptr(), &mut stat);
            if ret == 0 {
                let block_size = stat.f_frsize as u64;
                let total = stat.f_blocks as u64 * block_size;
                let free = stat.f_bfree as u64 * block_size;
                let avail = stat.f_bavail as u64 * block_size;
                let used = total.saturating_sub(free);
                let percent = if total > 0 {
                    (used as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                let mut obj = HashMap::new();
                obj.insert("total".to_string(), Value16::number(total as f64));
                obj.insert("used".to_string(), Value16::number(used as f64));
                obj.insert("free".to_string(), Value16::number(avail as f64));
                obj.insert(
                    "percent".to_string(),
                    Value16::number((percent * 100.0).round() / 100.0),
                );
                return Ok(Value16::object(obj));
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = path;
    }

    let mut obj = HashMap::new();
    obj.insert("total".to_string(), Value16::number(0.0));
    obj.insert("used".to_string(), Value16::number(0.0));
    obj.insert("free".to_string(), Value16::number(0.0));
    obj.insert("percent".to_string(), Value16::number(0.0));
    Ok(Value16::object(obj))
}
