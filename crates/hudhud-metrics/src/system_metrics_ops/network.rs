//! Network interfaces query operation.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;

use super::utils::runtime_error;

pub fn sys_network_interfaces(_args: &[Value16]) -> HudHudResult<Value16> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
            let mut interfaces = Vec::new();
            for line in content.lines().skip(2) {
                let line = line.trim();
                if let Some((name, rest)) = line.split_once(':') {
                    let fields: Vec<u64> = rest
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    let rx_bytes = fields.first().copied().unwrap_or(0);
                    let tx_bytes = fields.get(8).copied().unwrap_or(0);

                    let mut iface = hudhudscript_bytecode::ObjMap::default();
                    iface.insert("name".to_string(), Value16::string(name.trim().to_string()));
                    iface.insert("rx_bytes".to_string(), Value16::number(rx_bytes as f64));
                    iface.insert("tx_bytes".to_string(), Value16::number(tx_bytes as f64));
                    interfaces.push(Value16::object(iface));
                }
            }
            return Ok(Value16::array(interfaces));
        }
    }
    Err(runtime_error(
        "system.network_interfaces: only supported on Linux (requires /proc/net/dev)",
    ))
}
