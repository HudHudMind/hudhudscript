//! Shared security-scanning builtins — suid_files, check_ssl,
//! world_writable, open_ports, failed_logins, check_permissions.
//!
//! Single source of truth for the VM and interpreter runtimes (Kural 7).
//! Wraps system tools: find, openssl, ss/netstat, journalctl, stat.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}
use super::*;
use std::collections::HashMap;
use std::process::Command;

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

pub fn sec_failed_logins(args: &[Value16]) -> HudHudResult<Value16> {
    let count = if args.is_empty() {
        10.0
    } else {
        args[0].as_number().ok_or_else(|| {
            type_error("number", args[0].type_name_str(), "security.failed_logins")
        })?
    };
    let count_str = format!("{}", count as u64);

    let output = Command::new("journalctl")
        .args([
            "_SYSTEMD_UNIT=sshd.service",
            "-n",
            &count_str,
            "--no-pager",
            "-o",
            "short-iso",
            "--grep",
            "Failed password",
        ])
        .output();

    let lines = match output {
        Ok(ref out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        _ => match Command::new("sh")
            .args([
                "-c",
                &format!(
                    "grep 'Failed password' /var/log/auth.log 2>/dev/null | tail -n {}",
                    count as u64
                ),
            ])
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => String::new(),
        },
    };

    let mut entries: Vec<Value16> = Vec::new();
    for line in lines.lines() {
        if line.trim().is_empty() || !line.contains("Failed password") {
            continue;
        }

        let mut user = String::new();
        let mut ip = String::new();
        let mut timestamp = String::new();

        if let Some(after_for) = line.split("Failed password for ").nth(1) {
            let cleaned = after_for.strip_prefix("invalid user ").unwrap_or(after_for);
            user = cleaned.split_whitespace().next().unwrap_or("").to_string();
        }
        if let Some(after_from) = line.split(" from ").nth(1) {
            ip = after_from
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
        }
        let ts_parts: Vec<&str> = line.splitn(4, ' ').collect();
        if !ts_parts.is_empty() {
            if ts_parts[0].contains('T') || ts_parts[0].contains('-') {
                timestamp = ts_parts[0].to_string();
            } else if ts_parts.len() >= 3 {
                timestamp = ts_parts[..3].join(" ");
            }
        }

        let mut entry = hudhudscript_bytecode::ObjMap::default();
        entry.insert("user".to_string(), Value16::string(user));
        entry.insert("ip".to_string(), Value16::string(ip));
        entry.insert("timestamp".to_string(), Value16::string(timestamp));
        entries.push(Value16::object(entry));
    }
    Ok(Value16::array(entries))
}
