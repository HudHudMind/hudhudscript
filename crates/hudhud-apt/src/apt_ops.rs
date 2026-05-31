//! Shared APT package manager wrapper — single source of truth (Kural 7).

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
use std::collections::HashMap;
use std::process::Command;

/// Enum identifying each operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethodId {
    ListInstalled,
    Search,
    Info,
    Install,
    Remove,
    Update,
    Upgradable,
    AddRepo,
    AddKey,
}

impl std::str::FromStr for ScriptMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "list_installed" => Ok(Self::ListInstalled),
            "search" => Ok(Self::Search),
            "info" => Ok(Self::Info),
            "install" => Ok(Self::Install),
            "remove" => Ok(Self::Remove),
            "update" => Ok(Self::Update),
            "upgradable" => Ok(Self::Upgradable),
            "add_repo" => Ok(Self::AddRepo),
            "add_key" => Ok(Self::AddKey),
            _ => Err(runtime_error(format!("Unknown method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch.
pub fn dispatch(method: ScriptMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        ScriptMethodId::ListInstalled => apt_list_installed(args),
        ScriptMethodId::Search => apt_search(args),
        ScriptMethodId::Info => apt_info(args),
        ScriptMethodId::Install => apt_install(args),
        ScriptMethodId::Remove => apt_remove(args),
        ScriptMethodId::Update => apt_update(args),
        ScriptMethodId::Upgradable => apt_upgradable(args),
        ScriptMethodId::AddRepo => apt_add_repo(args),
        ScriptMethodId::AddKey => apt_add_key(args),
    }
}

/// Main entry point (kept for backward compat).

pub fn apt_list_installed(_args: &[Value16]) -> HudHudResult<Value16> {
    let output = Command::new("dpkg-query")
        .args(["-W", "-f", "${Package}\t${Version}\t${Architecture}\n"])
        .output()
        .map_err(|e| runtime_error(format!("apt.list_installed: {e}")))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "apt.list_installed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages: Vec<Value16> = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() >= 2 {
            let mut pkg = HashMap::new();
            pkg.insert("name".to_string(), Value16::string(parts[0].to_string()));
            pkg.insert("version".to_string(), Value16::string(parts[1].to_string()));
            pkg.insert(
                "architecture".to_string(),
                Value16::string(parts.get(2).unwrap_or(&"").to_string()),
            );
            packages.push(Value16::object(pkg));
        }
    }
    Ok(Value16::array(packages))
}

pub fn apt_search(args: &[Value16]) -> HudHudResult<Value16> {
    let query = require_str(args, 0, "apt.search")?.to_string();
    let output = Command::new("apt-cache")
        .args(["search", &query])
        .output()
        .map_err(|e| runtime_error(format!("apt.search: {e}")))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "apt.search: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results: Vec<Value16> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut pkg = HashMap::new();
        if let Some((name, desc)) = line.split_once(" - ") {
            pkg.insert("name".to_string(), Value16::string(name.trim().to_string()));
            pkg.insert(
                "description".to_string(),
                Value16::string(desc.trim().to_string()),
            );
        } else {
            pkg.insert("name".to_string(), Value16::string(line.to_string()));
            pkg.insert("description".to_string(), Value16::string(String::new()));
        }
        pkg.insert("version".to_string(), Value16::string(String::new()));
        results.push(Value16::object(pkg));
    }
    Ok(Value16::array(results))
}

pub fn apt_info(args: &[Value16]) -> HudHudResult<Value16> {
    let package = require_str(args, 0, "apt.info")?.to_string();
    let output = Command::new("apt-cache")
        .args(["show", &package])
        .output()
        .map_err(|e| runtime_error(format!("apt.info: {e}")))?;
    if !output.status.success() {
        return Err(runtime_error(format!(
            "apt.info: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), Value16::string(package));
    obj.insert("version".to_string(), Value16::string(String::new()));
    obj.insert("description".to_string(), Value16::string(String::new()));
    obj.insert("depends".to_string(), Value16::string(String::new()));
    obj.insert("size".to_string(), Value16::string(String::new()));
    obj.insert("maintainer".to_string(), Value16::string(String::new()));

    let mut has_installed_size = false;
    for line in stdout.lines() {
        if let Some(val) = line.strip_prefix("Version: ") {
            obj.insert(
                "version".to_string(),
                Value16::string(val.trim().to_string()),
            );
        } else if let Some(val) = line.strip_prefix("Description: ") {
            obj.insert(
                "description".to_string(),
                Value16::string(val.trim().to_string()),
            );
        } else if let Some(val) = line.strip_prefix("Depends: ") {
            obj.insert(
                "depends".to_string(),
                Value16::string(val.trim().to_string()),
            );
        } else if let Some(val) = line.strip_prefix("Installed-Size: ") {
            obj.insert("size".to_string(), Value16::string(val.trim().to_string()));
            has_installed_size = true;
        } else if let Some(val) = line.strip_prefix("Size: ") {
            if !has_installed_size {
                obj.insert("size".to_string(), Value16::string(val.trim().to_string()));
            }
        } else if let Some(val) = line.strip_prefix("Maintainer: ") {
            obj.insert(
                "maintainer".to_string(),
                Value16::string(val.trim().to_string()),
            );
        }
    }
    Ok(Value16::object(obj))
}

pub fn apt_install(args: &[Value16]) -> HudHudResult<Value16> {
    let package = require_str(args, 0, "apt.install")?.to_string();
    run_cmd_result(
        Command::new("sudo").args(["apt-get", "install", "-y", &package]),
        "apt.install",
    )
}

pub fn apt_remove(args: &[Value16]) -> HudHudResult<Value16> {
    let package = require_str(args, 0, "apt.remove")?.to_string();
    run_cmd_result(
        Command::new("sudo").args(["apt-get", "remove", "-y", &package]),
        "apt.remove",
    )
}

pub fn apt_update(_args: &[Value16]) -> HudHudResult<Value16> {
    run_cmd_result(
        Command::new("sudo").args(["apt-get", "update"]),
        "apt.update",
    )
}

pub fn apt_upgradable(_args: &[Value16]) -> HudHudResult<Value16> {
    let output = Command::new("apt")
        .args(["list", "--upgradable"])
        .output()
        .map_err(|e| runtime_error(format!("apt.upgradable: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results: Vec<Value16> = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Listing") {
            continue;
        }
        let mut pkg = HashMap::new();
        let name = line.split('/').next().unwrap_or("").to_string();
        pkg.insert("name".to_string(), Value16::string(name));
        let after_slash = line.split('/').nth(1).unwrap_or("");
        let available = after_slash
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        pkg.insert("available".to_string(), Value16::string(available));
        let current = if let Some(idx) = line.find("upgradable from: ") {
            let rest = &line[idx + "upgradable from: ".len()..];
            rest.trim_end_matches(']').trim().to_string()
        } else {
            String::new()
        };
        pkg.insert("current".to_string(), Value16::string(current));
        results.push(Value16::object(pkg));
    }
    Ok(Value16::array(results))
}

pub fn apt_add_repo(args: &[Value16]) -> HudHudResult<Value16> {
    let repo_line = require_str(args, 0, "apt.add_repo")?.to_string();
    run_cmd_result(
        Command::new("sudo").args(["add-apt-repository", "-y", &repo_line]),
        "apt.add_repo",
    )
}

pub fn apt_add_key(args: &[Value16]) -> HudHudResult<Value16> {
    let url = require_str(args, 0, "apt.add_key")?.to_string();
    let key_name = url
        .rsplit('/')
        .next()
        .unwrap_or("custom-key")
        .replace(".asc", "")
        .replace(".gpg", "");
    let keyring_path = format!("/usr/share/keyrings/{key_name}.gpg");

    let curl = Command::new("curl")
        .args(["-fsSL", &url])
        .output()
        .map_err(|e| runtime_error(format!("apt.add_key: curl failed: {e}")))?;

    if !curl.status.success() {
        let mut obj = HashMap::new();
        obj.insert("ok".to_string(), Value16::bool_(false));
        obj.insert(
            "message".to_string(),
            Value16::string(format!(
                "curl failed: {}",
                String::from_utf8_lossy(&curl.stderr).trim()
            )),
        );
        return Ok(Value16::object(obj));
    }

    let gpg = Command::new("sudo")
        .args(["gpg", "--dearmor", "-o", &keyring_path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    let mut obj = HashMap::new();
    match gpg {
        Ok(mut child) => {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(&curl.stdout);
            }
            let output = child
                .wait_with_output()
                .map_err(|e| runtime_error(format!("apt.add_key: gpg failed: {e}")))?;
            let ok = output.status.success();
            let msg = if ok {
                format!("Key saved to {keyring_path}")
            } else {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            };
            obj.insert("ok".to_string(), Value16::bool_(ok));
            obj.insert("message".to_string(), Value16::string(msg));
        }
        Err(e) => {
            obj.insert("ok".to_string(), Value16::bool_(false));
            obj.insert(
                "message".to_string(),
                Value16::string(format!("gpg spawn failed: {e}")),
            );
        }
    }
    Ok(Value16::object(obj))
}

fn require_str<'a>(args: &'a [Value16], idx: usize, op: &str) -> HudHudResult<&'a str> {
    match args.get(idx) {
        Some(v) => v
            .as_str()
            .ok_or_else(|| type_error("string", v.type_name_str(), op)),
        None => Err(runtime_error(format!(
            "{}: missing argument at index {}",
            op, idx
        ))),
    }
}

fn run_cmd_result(cmd: &mut Command, op: &str) -> HudHudResult<Value16> {
    let output = cmd
        .output()
        .map_err(|e| runtime_error(format!("{}: {}", op, e)))?;
    let ok = output.status.success();
    let msg = if ok {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    };
    let mut obj = HashMap::new();
    obj.insert("ok".to_string(), Value16::bool_(ok));
    obj.insert("message".to_string(), Value16::string(msg));
    Ok(Value16::object(obj))
}
