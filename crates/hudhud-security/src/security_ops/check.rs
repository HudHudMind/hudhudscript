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

pub fn sec_check_ssl(args: &[Value16]) -> HudHudResult<Value16> {
    let host = require_str(args, 0, "security.check_ssl")?.to_string();
    let port = if args.len() > 1 {
        if let Some(n) = args[1].as_number() {
            format!("{}", n as u16)
        } else if let Some(s) = args[1].as_str() {
            s.to_string()
        } else {
            "443".to_string()
        }
    } else {
        "443".to_string()
    };

    let connect_addr = format!("{}:{}", host, port);

    let output = Command::new("openssl")
        .args([
            "s_client",
            "-connect",
            &connect_addr,
            "-servername",
            &host,
            "-brief",
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output();

    let mut result: HashMap<String, Value16> = HashMap::new();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}\n{}", stdout, stderr);

            let valid =
                combined.contains("Verification: OK") || combined.contains("Verify return code: 0");
            result.insert("valid".to_string(), Value16::bool_(valid));

            let protocol = combined
                .lines()
                .find(|l| l.contains("Protocol version:") || l.contains("Protocol  :"))
                .map(|l| {
                    l.split(':')
                        .next_back()
                        .unwrap_or("unknown")
                        .trim()
                        .to_string()
                })
                .unwrap_or_else(|| "unknown".to_string());
            result.insert("protocol".to_string(), Value16::string(protocol));

            let cipher = combined
                .lines()
                .find(|l| l.contains("Ciphersuite:") || l.contains("Cipher    :"))
                .map(|l| {
                    l.split(':')
                        .next_back()
                        .unwrap_or("unknown")
                        .trim()
                        .to_string()
                })
                .unwrap_or_else(|| "unknown".to_string());
            result.insert("cipher".to_string(), Value16::string(cipher));

            let cert_output = Command::new("sh")
                .args([
                    "-c",
                    &format!(
                        "echo | openssl s_client -connect {} -servername {} 2>/dev/null | openssl x509 -noout -issuer -enddate 2>/dev/null",
                        connect_addr, host
                    ),
                ])
                .output();

            if let Ok(cert_out) = cert_output {
                let cert_text = String::from_utf8_lossy(&cert_out.stdout);
                let issuer = cert_text
                    .lines()
                    .find(|l| l.starts_with("issuer=") || l.starts_with("issuer ="))
                    .map(|l| {
                        l.split_once('=')
                            .map(|x| x.1)
                            .unwrap_or("unknown")
                            .trim()
                            .to_string()
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                result.insert("issuer".to_string(), Value16::string(issuer));

                let expiry_line = cert_text
                    .lines()
                    .find(|l| l.starts_with("notAfter=") || l.starts_with("notAfter ="));

                if let Some(exp) = expiry_line {
                    let date_str = exp
                        .split_once('=')
                        .map(|x| x.1)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let days_output = Command::new("sh")
                        .args([
                            "-c",
                            &format!(
                                "echo $(( ( $(date -d '{}' +%s) - $(date +%s) ) / 86400 ))",
                                date_str
                            ),
                        ])
                        .output();
                    if let Ok(d) = days_output {
                        let days_str = String::from_utf8_lossy(&d.stdout).trim().to_string();
                        if let Ok(days) = days_str.parse::<f64>() {
                            result.insert("expiry_days".to_string(), Value16::number(days));
                        } else {
                            result.insert("expiry_days".to_string(), Value16::number(-1.0));
                        }
                    } else {
                        result.insert("expiry_days".to_string(), Value16::number(-1.0));
                    }
                } else {
                    result.insert("expiry_days".to_string(), Value16::number(-1.0));
                }
            } else {
                result.insert("issuer".to_string(), Value16::string("unknown".to_string()));
                result.insert("expiry_days".to_string(), Value16::number(-1.0));
            }
        }
        Err(_) => {
            result.insert("valid".to_string(), Value16::bool_(false));
            result.insert("issuer".to_string(), Value16::string("unknown".to_string()));
            result.insert("expiry_days".to_string(), Value16::number(-1.0));
            result.insert("cipher".to_string(), Value16::string("unknown".to_string()));
            result.insert(
                "protocol".to_string(),
                Value16::string("unknown".to_string()),
            );
        }
    }

    Ok(Value16::object(result))
}

pub fn sec_check_permissions(args: &[Value16]) -> HudHudResult<Value16> {
    let path = require_str(args, 0, "security.check_permissions")?.to_string();

    let output = Command::new("stat")
        .args(["-c", "%U %G %a", &path])
        .output()
        .map_err(|e| {
            runtime_error(format!(
                "security.check_permissions: failed to run stat: {}",
                e
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(runtime_error(format!(
            "security.check_permissions: stat failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = stdout.split_whitespace().collect();

    let (owner, group, mode) = if parts.len() >= 3 {
        (
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        )
    } else {
        (
            "unknown".to_string(),
            "unknown".to_string(),
            "000".to_string(),
        )
    };

    let last_digit = mode.chars().last().unwrap_or('0').to_digit(8).unwrap_or(0);
    let is_secure = last_digit == 0;

    let mut result = HashMap::new();
    result.insert("owner".to_string(), Value16::string(owner));
    result.insert("group".to_string(), Value16::string(group));
    result.insert("mode".to_string(), Value16::string(mode));
    result.insert("is_secure".to_string(), Value16::bool_(is_secure));
    Ok(Value16::object(result))
}
