//! Build configuration and discovery engine (JIT_AOT_ARCHITECTURE §20.1, §20.4).

use std::path::{Path, PathBuf};
use crate::common::{load_hudhud_config_with_path, HudHudConfig};
use super::config_include::load_toml_with_includes;

/// Loads build configuration with two-file discovery rule (JIT_AOT_ARCHITECTURE §20.1):
/// 1. If explicit path is given, loads it directly.
/// 2. Searches for `hudhud.build.toml` walking up from cwd.
/// 3. If missing, falls back to `hudhud.toml`.
pub fn load_hudhud_build_config(debug: bool, explicit_path: Option<&Path>) -> HudHudConfig {
    if let Some(explicit) = explicit_path {
        return load_config_file(explicit, debug)
            .unwrap_or_else(|| load_hudhud_config_with_path(debug, Some(explicit)));
    }

    // Walk up looking for hudhud.build.toml
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = start.as_path();
    loop {
        let candidate = dir.join("hudhud.build.toml");
        if candidate.is_file() {
            if let Some(cfg) = load_config_file(&candidate, debug) {
                if debug {
                    eprintln!("[config] Using build config: {}", candidate.display());
                }
                return cfg;
            }
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // Fall back to standard hudhud.toml
    load_hudhud_config_with_path(debug, None)
}

fn load_config_file(path: &Path, debug: bool) -> Option<HudHudConfig> {
    match load_toml_with_includes(path) {
        Ok(val) => match val.try_into::<HudHudConfig>() {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                if debug {
                    eprintln!("[config] Failed to deserialize {}: {}", path.display(), e);
                }
                None
            }
        },
        Err(e) => {
            if debug {
                eprintln!("[config] Error loading {}: {}", path.display(), e);
            }
            None
        }
    }
}

/// Applies a named profile from `[profile.NAME]` if present (JIT_AOT_ARCHITECTURE §20.4).
pub fn apply_profile(mut config: HudHudConfig, profile_name: &str) -> HudHudConfig {
    if let Some(p) = config.profile.get(profile_name).cloned() {
        if let Some(build_mode) = p.build {
            let build = config.build.get_or_insert_with(Default::default);
            build.mode = Some(build_mode);
        }
        if let Some(runtime_engine) = p.runtime {
            config.runtime.engine = Some(runtime_engine);
        }
        if let Some(backend) = p.backend {
            let build = config.build.get_or_insert_with(Default::default);
            let aot = build.aot.get_or_insert_with(Default::default);
            aot.backend = Some(backend);
        }
        if let Some(opt_val) = p.optimization {
            let opt = config.optimization.get_or_insert_with(Default::default);
            match opt_val {
                toml::Value::Integer(i) if (0..=3).contains(&i) => {
                    opt.level = Some(i as u8);
                }
                toml::Value::String(s) => {
                    opt.goal = Some(s);
                }
                _ => {}
            }
        }
        if let Some(target) = p.target {
            let tgt = config.target.get_or_insert_with(Default::default);
            tgt.triple = Some(target);
        }
    }
    config
}

/// Strict configuration cross-validation (JIT_AOT_ARCHITECTURE §20.4).
pub fn validate_config(config: &HudHudConfig) -> Result<(), String> {
    if let (Some(b), Some(r)) = (&config.build, &config.runtime.engine) {
        if b.mode.as_deref() == Some("aot") && r == "jit" {
            return Err("runtime.engine=\"jit\" has no meaning when build.mode=\"aot\"".to_string());
        }
    }
    if let Some(ref eng) = config.runtime.engine {
        if eng != "vm" && eng != "jit" {
            return Err(format!("unknown runtime.engine `{eng}` — available: vm, jit"));
        }
    }
    if let Some(ref jit) = config.runtime.jit {
        if let Some(ref b) = jit.backend {
            if !matches!(b.as_str(), "auto" | "cranelift" | "llvm" | "gccjit") {
                return Err(format!(
                    "unknown runtime.jit.backend `{b}` — available: auto, cranelift, llvm, gccjit"
                ));
            }
        }
        if let Some(ref p) = jit.policy {
            if !matches!(p.as_str(), "hot" | "lazy" | "eager") {
                return Err(format!(
                    "unknown runtime.jit.policy `{p}` — available: hot, lazy, eager"
                ));
            }
        }
        if let Some(ref f) = jit.fallback {
            if !matches!(f.as_str(), "none" | "vm") {
                return Err(format!(
                    "unknown runtime.jit.fallback `{f}` — available: none, vm"
                ));
            }
        }
    }
    if let Some(ref b) = config.build {
        if let Some(ref m) = b.mode {
            if !matches!(m.as_str(), "bytecode" | "aot") {
                return Err(format!("unknown build.mode `{m}` — available: bytecode, aot"));
            }
        }
        if let Some(ref aot) = b.aot {
            if let Some(ref backend) = aot.backend {
                if !matches!(backend.as_str(), "auto" | "cranelift" | "llvm" | "gccjit") {
                    return Err(format!(
                        "unknown build.aot.backend `{backend}` — available: auto, cranelift, llvm, gccjit"
                    ));
                }
            }
            if let Some(ref fmt) = aot.format {
                if !matches!(fmt.as_str(), "executable" | "shared") {
                    return Err(format!(
                        "unknown build.aot.format `{fmt}` — available: executable, shared"
                    ));
                }
            }
        }
    }
    Ok(())
}
