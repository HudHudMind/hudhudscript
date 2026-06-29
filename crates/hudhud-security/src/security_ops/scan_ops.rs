//! Filesystem and network scan operations: suid_files, world_writable, open_ports.

use std::collections::HashMap;
use std::process::Command;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::helpers::{require_str, runtime_error};

pub fn sec_suid_files(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "security.suid_files")?.to_string();

    let output = Command::new("find")
        .args([&path, "-perm", "/6000", "-type", "f", "-ls"])
        .output()
        .map_err(|e| runtime_error(format!("security.suid_files: failed to run find: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<Value16> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                let mut obj = hudhudscript_bytecode::ObjMap::default();
                obj.insert(
                    "permissions".to_string(),
                    Value16::string(parts[2].to_string()),
                );
                obj.insert("owner".to_string(), Value16::string(parts[4].to_string()));
                obj.insert("group".to_string(), Value16::string(parts[5].to_string()));
                obj.insert("size".to_string(), Value16::string(parts[6].to_string()));
                let file_path = parts[10..].join(" ");
                obj.insert("path".to_string(), Value16::string(file_path));
                Value16::object(obj)
            } else {
                Value16::string(line.trim().to_string())
            }
        })
        .collect();
    Ok(Value16::array(files))
}

pub fn sec_world_writable(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "security.world_writable")?.to_string();

    let output = Command::new("find")
        .args([&path, "-perm", "-o+w", "-type", "f"])
        .output()
        .map_err(|e| {
            runtime_error(format!(
                "security.world_writable: failed to run find: {}",
                e
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<Value16> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| Value16::string(l.trim().to_string()))
        .collect();
    Ok(Value16::array(files))
}

pub fn sec_open_ports(_args: &[Value16]) -> HudHudResult<Value16> {
    let output = Command::new("ss")
        .args(["-tulnp"])
        .output()
        .or_else(|_| Command::new("netstat").args(["-tulnp"]).output())
        .map_err(|e| {
            runtime_error(format!(
                "security.open_ports: failed to run ss/netstat: {}",
                e
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports: Vec<Value16> = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let protocol = parts[0].to_string();
        let local_addr = parts[4];

        let port_str = local_addr
            .rsplit(':')
            .next()
            .unwrap_or("0")
            .trim_matches('*');
        let port_num: f64 = port_str.parse().unwrap_or(0.0);

        let mut pid = String::new();
        let mut process_name = String::new();
        if let Some(users_col) = parts.last() {
            if users_col.contains("pid=") {
                if let Some(pid_part) = users_col.split("pid=").nth(1) {
                    pid = pid_part
                        .split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap_or("")
                        .to_string();
                }
                if let Some(name_part) = users_col.split("((\"").nth(1) {
                    process_name = name_part.split('"').next().unwrap_or("").to_string();
                }
            }
        }

        let mut entry = hudhudscript_bytecode::ObjMap::default();
        entry.insert("port".to_string(), Value16::number(port_num));
        entry.insert("pid".to_string(), Value16::string(pid));
        entry.insert("process".to_string(), Value16::string(process_name));
        entry.insert("protocol".to_string(), Value16::string(protocol));
        ports.push(Value16::object(entry));
    }
    Ok(Value16::array(ports))
}
