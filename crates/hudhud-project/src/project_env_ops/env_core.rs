use std::collections::HashMap;
use std::path::Path;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::helpers::{require_string, MARKERS};

pub fn detect(args: &[Value16]) -> HudHudResult<Value16> {
    let dir = require_string(args, 0, "project.detect")?;
    let base = Path::new(&dir);

    let mut language = "unknown";
    let mut toolchain = "unknown";
    let mut package_manager = "unknown";
    let mut files_found: Vec<Value16> = Vec::new();
    let mut venv_type = "none";

    for m in MARKERS {
        if base.join(m.file).exists() {
            if language == "unknown" {
                language = m.language;
                toolchain = m.toolchain;
                package_manager = m.package_manager;
            }
            files_found.push(Value16::string(m.file.to_string()));
        }
    }

    let venv_markers: &[(&str, &str)] = &[
        (".venv", "venv"),
        ("venv", "venv"),
        (".nvm", "nvm"),
        (".nvmrc", "nvm"),
        (".node-version", "nvm"),
        (".python-version", "pyenv"),
        (".ruby-version", "rbenv"),
        (".conda", "conda"),
        ("environment.yml", "conda"),
    ];

    for (file, vt) in venv_markers {
        if base.join(file).exists() {
            venv_type = vt;
            files_found.push(Value16::string(file.to_string()));
            break;
        }
    }

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert(
        "language".to_string(),
        Value16::string(language.to_string()),
    );
    result.insert(
        "toolchain".to_string(),
        Value16::string(toolchain.to_string()),
    );
    result.insert(
        "package_manager".to_string(),
        Value16::string(package_manager.to_string()),
    );
    result.insert(
        "venv_type".to_string(),
        Value16::string(venv_type.to_string()),
    );
    result.insert("files_found".to_string(), Value16::array(files_found));
    Ok(Value16::object(result))
}

pub fn detect_venv(args: &[Value16]) -> HudHudResult<Value16> {
    let dir = require_string(args, 0, "project.detect_venv")?;
    let base = Path::new(&dir);

    let venv_dirs: &[(&str, &str)] = &[
        (".venv", "venv"),
        ("venv", "venv"),
        ("env", "venv"),
        (".conda", "conda"),
    ];

    for (d, vtype) in venv_dirs {
        let p = base.join(d);
        if p.is_dir() {
            let active = std::env::var("VIRTUAL_ENV")
                .map(|v| Path::new(&v) == p)
                .unwrap_or(false);

            let mut obj = hudhudscript_bytecode::ObjMap::default();
            obj.insert("type".to_string(), Value16::string(vtype.to_string()));
            obj.insert(
                "path".to_string(),
                Value16::string(p.to_string_lossy().to_string()),
            );
            obj.insert("active".to_string(), Value16::bool_(active));
            return Ok(Value16::object(obj));
        }
    }

    if base.join(".nvmrc").exists() || base.join(".node-version").exists() {
        let active = std::env::var("NVM_DIR").is_ok();
        let mut obj = hudhudscript_bytecode::ObjMap::default();
        obj.insert("type".to_string(), Value16::string("nvm".to_string()));
        obj.insert(
            "path".to_string(),
            Value16::string(std::env::var("NVM_DIR").unwrap_or_else(|_| "~/.nvm".to_string())),
        );
        obj.insert("active".to_string(), Value16::bool_(active));
        return Ok(Value16::object(obj));
    }

    if base.join(".ruby-version").exists() {
        let active = std::env::var("RBENV_ROOT").is_ok();
        let mut obj = hudhudscript_bytecode::ObjMap::default();
        obj.insert("type".to_string(), Value16::string("rbenv".to_string()));
        obj.insert(
            "path".to_string(),
            Value16::string(std::env::var("RBENV_ROOT").unwrap_or_else(|_| "~/.rbenv".to_string())),
        );
        obj.insert("active".to_string(), Value16::bool_(active));
        return Ok(Value16::object(obj));
    }

    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("type".to_string(), Value16::string("none".to_string()));
    obj.insert("path".to_string(), Value16::null());
    obj.insert("active".to_string(), Value16::bool_(false));
    Ok(Value16::object(obj))
}
