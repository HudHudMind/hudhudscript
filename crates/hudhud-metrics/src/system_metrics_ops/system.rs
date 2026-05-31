//! Load average, uptime, and hostname query operations.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn sys_load_average(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<f64> = content
                .split_whitespace()
                .take(3)
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() == 3 {
                return Ok(Value16::array(
                    parts.into_iter().map(Value16::number).collect(),
                ));
            }
        }
    }
    Ok(Value16::array(vec![
        Value16::number(0.0),
        Value16::number(0.0),
        Value16::number(0.0),
    ]))
}

pub fn sys_uptime(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
            if let Some(secs) = content.split_whitespace().next() {
                if let Ok(val) = secs.parse::<f64>() {
                    return Ok(Value16::number(val));
                }
            }
        }
    }
    Ok(Value16::number(0.0))
}

pub fn sys_hostname(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(unix)]
    {
        let mut buf = [0u8; 256];
        let result = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
        if result == 0 {
            let hostname = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const libc::c_char) };
            return Ok(Value16::string(hostname.to_string_lossy().to_string()));
        }
        Ok(Value16::string("unknown".to_string()))
    }

    #[cfg(not(unix))]
    {
        let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());
        Ok(Value16::string(hostname))
    }
}
