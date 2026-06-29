//! Network and USB device detection — reads /sys/class/net and lsusb.

use std::collections::HashMap;
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

pub fn hw_network_adapters(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut adapters: Vec<Value16> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        let net_dir = std::path::Path::new("/sys/class/net");
        if let Ok(entries) = std::fs::read_dir(net_dir) {
            for entry in entries.flatten() {
                let iface_name = entry.file_name().to_string_lossy().to_string();
                let iface_path = entry.path();

                let mac = std::fs::read_to_string(iface_path.join("address"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let speed = std::fs::read_to_string(iface_path.join("speed"))
                    .ok()
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .unwrap_or(0.0);

                let driver = std::fs::read_link(iface_path.join("device/driver"))
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                    .unwrap_or_default();

                let mut adapter = hudhudscript_bytecode::ObjMap::default();
                adapter.insert("name".to_string(), Value16::string(iface_name));
                adapter.insert("driver".to_string(), Value16::string(driver));
                adapter.insert("mac".to_string(), Value16::string(mac));
                adapter.insert("speed".to_string(), Value16::number(speed));
                adapters.push(Value16::object(adapter));
            }
        }
    }

    Ok(Value16::array(adapters))
}

pub fn hw_usb_devices(_args: &[Value16]) -> HudHudResult<Value16> {
    let mut devices: Vec<Value16> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("lsusb").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    let mut bus = String::new();
                    let mut device_num = String::new();
                    let mut vendor = String::new();
                    let mut product = String::new();

                    if let Some(rest) = line.strip_prefix("Bus ") {
                        if let Some((b, rest)) = rest.split_once(' ') {
                            bus = b.to_string();
                            if let Some(rest) = rest.strip_prefix("Device ") {
                                if let Some((d, rest)) = rest.split_once(':') {
                                    device_num = d.to_string();
                                    if let Some(rest) = rest.trim().strip_prefix("ID ") {
                                        if let Some((id_part, desc)) = rest.split_once(' ') {
                                            let id_parts: Vec<&str> = id_part.split(':').collect();
                                            vendor = id_parts.first().unwrap_or(&"").to_string();
                                            product = id_parts.get(1).unwrap_or(&"").to_string();
                                            let desc = desc.trim();
                                            if !desc.is_empty() {
                                                product = format!("{} ({})", product, desc);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mut dev = hudhudscript_bytecode::ObjMap::default();
                    dev.insert("vendor".to_string(), Value16::string(vendor));
                    dev.insert("product".to_string(), Value16::string(product));
                    dev.insert("bus".to_string(), Value16::string(bus));
                    dev.insert("device".to_string(), Value16::string(device_num));
                    devices.push(Value16::object(dev));
                }
            }
        }
    }

    Ok(Value16::array(devices))
}
