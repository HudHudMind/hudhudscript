use super::config_include::load_toml_with_includes;
use super::config_types::*;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Cached default configuration (system + user + project local) loaded once per process.
static CACHED_DEFAULT_CONFIG: OnceLock<HudHudConfig> = OnceLock::new();
static CACHED_EXPLICIT_CONFIG: Mutex<Option<(PathBuf, HudHudConfig)>> = Mutex::new(None);

#[allow(dead_code)]
pub fn load_hudhud_config(debug: bool) -> HudHudConfig {
    load_hudhud_config_with_path(debug, None)
}

/// Load hudhud.toml with optional explicit path override (Issue #1006, JIT_AOT_ARCHITECTURE §20).
pub fn load_hudhud_config_with_path(
    debug: bool,
    explicit_path: Option<&Path>,
) -> HudHudConfig {
    if !debug {
        if let Some(explicit) = explicit_path {
            if let Ok(guard) = CACHED_EXPLICIT_CONFIG.lock() {
                if let Some((ref cached_path, ref cfg)) = *guard {
                    if cached_path == explicit {
                        return cfg.clone();
                    }
                }
            }
        } else if let Some(cfg) = CACHED_DEFAULT_CONFIG.get() {
            return cfg.clone();
        }
    }

    let config = resolve_hudhud_config(debug, explicit_path);

    if !debug {
        if let Some(explicit) = explicit_path {
            if let Ok(mut guard) = CACHED_EXPLICIT_CONFIG.lock() {
                *guard = Some((explicit.to_path_buf(), config.clone()));
            }
        } else {
            let _ = CACHED_DEFAULT_CONFIG.set(config.clone());
        }
    }

    config
}

fn resolve_hudhud_config(
    debug: bool,
    explicit_path: Option<&Path>,
) -> HudHudConfig {
    let mut config = HudHudConfig::default();

    // Layer 1: System global
    let system_paths: Vec<PathBuf> = if cfg!(target_os = "macos") {
        vec![PathBuf::from(
            "/Library/Application Support/hudhud/script/hudhud.toml",
        )]
    } else if cfg!(target_os = "windows") {
        vec![std::env::var("PROGRAMDATA")
            .map(|d| PathBuf::from(d).join("hudhud/script/hudhud.toml"))
            .unwrap_or_else(|_| PathBuf::from("C:/ProgramData/hudhud/script/hudhud.toml"))]
    } else {
        vec![PathBuf::from("/etc/hudhud/script/hudhud.toml")]
    };
    for path in &system_paths {
        if let Some(loaded) = try_load_config(path, debug) {
            config = merge_config(config, loaded);
        }
    }

    // Layer 2: User global
    let user_path = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(|d| PathBuf::from(d).join("hudhud/script/hudhud.toml"))
            .unwrap_or_else(|_| dirs_fallback_home().join("hudhud/script/hudhud.toml"))
    } else {
        let xdg = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs_fallback_home().join(".config"));
        xdg.join("hudhud/script/hudhud.toml")
    };
    if let Some(loaded) = try_load_config(&user_path, debug) {
        config = merge_config(config, loaded);
    }

    // Layer 3: Project local (walk up from cwd)
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = start.as_path();
    loop {
        let candidate = dir.join("hudhud.toml");
        if candidate.is_file() {
            if let Some(loaded) = try_load_config(&candidate, debug) {
                config = merge_config(config, loaded);
            }
            break;
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    // Layer 4: Explicit --config flag (highest priority, Issue #1006)
    if let Some(explicit) = explicit_path {
        if let Some(loaded) = try_load_config(explicit, debug) {
            if debug {
                eprintln!("[config] Using explicit config: {}", explicit.display());
            }
            config = merge_config(config, loaded);
        } else {
            eprintln!(
                "Warning: --config file not found or invalid: {}",
                explicit.display()
            );
        }
    }

    config
}

/// Try to load a config file with recursive includes, returning None if not found or invalid.
fn try_load_config(path: &Path, debug: bool) -> Option<HudHudConfig> {
    if !path.is_file() {
        return None;
    }
    match load_toml_with_includes(path) {
        Ok(val) => match val.try_into::<HudHudConfig>() {
            Ok(mut cfg) => {
                if cfg.gc.growth_factor >= 50 {
                    cfg.gc.growth_factor = (cfg.gc.growth_factor / 100).max(1);
                }
                if debug {
                    eprintln!("[config] Loaded: {}", path.display());
                }
                Some(cfg)
            }
            Err(e) => {
                if debug {
                    eprintln!("[config] Parse error in {}: {}", path.display(), e);
                }
                None
            }
        },
        Err(e) => {
            if debug {
                eprintln!("[config] Failed to include/read {}: {}", path.display(), e);
            }
            None
        }
    }
}

/// Merge two configs: values from `overlay` override `base`.
fn merge_config(base: HudHudConfig, overlay: HudHudConfig) -> HudHudConfig {
    let overlay_max_recursion = overlay
        .runtime
        .vm
        .as_ref()
        .and_then(|v| v.max_call_depth)
        .unwrap_or(overlay.runtime.max_recursion);

    let max_recursion = if overlay_max_recursion != default_max_recursion() {
        overlay_max_recursion
    } else {
        base.runtime
            .vm
            .as_ref()
            .and_then(|v| v.max_call_depth)
            .unwrap_or(base.runtime.max_recursion)
    };

    let jit = match (base.runtime.jit, overlay.runtime.jit) {
        (Some(b), Some(o)) => Some(RuntimeJitConfig {
            backend: o.backend.or(b.backend),
            fallback: o.fallback.or(b.fallback),
            opt_level: o.opt_level.or(b.opt_level),
            opt_goal: o.opt_goal.or(b.opt_goal),
            mir_opt_rounds: o.mir_opt_rounds.or(b.mir_opt_rounds),
            policy: o.policy.or(b.policy),
            hot_threshold: o.hot_threshold.or(b.hot_threshold),
            loop_threshold: o.loop_threshold.or(b.loop_threshold),
            stats: o.stats.or(b.stats),
            verify_with_vm: o.verify_with_vm.or(b.verify_with_vm),
            cache: o.cache.or(b.cache),
            code_cache_mb: o.code_cache_mb.or(b.code_cache_mb),
        }),
        (None, Some(o)) => Some(o),
        (Some(b), None) => Some(b),
        (None, None) => None,
    };

    let vm = match (base.runtime.vm, overlay.runtime.vm) {
        (Some(b), Some(o)) => Some(RuntimeVmConfig {
            max_call_depth: o.max_call_depth.or(b.max_call_depth),
            stack_limit: o.stack_limit.or(b.stack_limit),
            fuel_limit: o.fuel_limit.or(b.fuel_limit),
        }),
        (None, Some(o)) => Some(o),
        (Some(b), None) => Some(b),
        (None, None) => None,
    };

    let mut profile = base.profile;
    profile.extend(overlay.profile);

    HudHudConfig {
        include: overlay.include,
        runtime: RuntimeConfig {
            max_recursion,
            stack_limit: if overlay.runtime.stack_limit != default_stack_limit() {
                overlay.runtime.stack_limit
            } else {
                base.runtime.stack_limit
            },
            fuel_limit: if overlay.runtime.fuel_limit != 0 {
                overlay.runtime.fuel_limit
            } else {
                base.runtime.fuel_limit
            },
            thread_stack_mb: if overlay.runtime.thread_stack_mb != default_thread_stack_mb() {
                overlay.runtime.thread_stack_mb
            } else {
                base.runtime.thread_stack_mb
            },
            register_arena_kb: overlay.runtime.register_arena_kb,
            mailbox_capacity: overlay.runtime.mailbox_capacity,
            max_mcp_servers: overlay.runtime.max_mcp_servers,
            execution_timeout_ms: overlay.runtime.execution_timeout_ms,
            builtin_max_iter: overlay.runtime.builtin_max_iter,
            max_call_depth_hard_ceiling: overlay.runtime.max_call_depth_hard_ceiling,
            default_stack_bytes: overlay.runtime.default_stack_bytes,
            provider_timeout_secs: if overlay.runtime.provider_timeout_secs
                != hudhudscript_runtime::provider::DEFAULT_PROVIDER_TIMEOUT_SECS
            {
                overlay.runtime.provider_timeout_secs
            } else {
                base.runtime.provider_timeout_secs
            },
            allow_network: overlay.runtime.allow_network || base.runtime.allow_network,
            allow_process: overlay.runtime.allow_process || base.runtime.allow_process,
            allow_insecure_http: overlay.runtime.allow_insecure_http
                || base.runtime.allow_insecure_http,
            allow_privileged: overlay.runtime.allow_privileged || base.runtime.allow_privileged,
            engine: overlay.runtime.engine.or(base.runtime.engine),
            backend: overlay.runtime.backend.or(base.runtime.backend),
            vm,
            jit,
        },
        _stream: base._stream,
        _security: base._security,
        host_access: base
            .host_access
            .clone()
            .map(|c| c.merge(overlay.host_access.as_ref()))
            .or_else(|| overlay.host_access.clone()),
        providers: overlay.providers,
        lint: overlay.lint,
        mcp: overlay.mcp,
        gc: overlay.gc,
        build: overlay.build.or(base.build),
        target: overlay.target.or(base.target),
        optimization: overlay.optimization.or(base.optimization),
        link: overlay.link.or(base.link),
        profile,
    }
}

fn dirs_fallback_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

#[path = "config_tests.rs"]
#[cfg(test)]
mod tests;
