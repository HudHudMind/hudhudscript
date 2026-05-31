use hudhudscript_bytecode::Value16;
use crate::common::{detect_locale, load_hudhud_config_with_path, CliError, HudHudConfig};
use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_with_recovery};
use hudhudscript_vm::{OutputLocale, VM};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn watch_and_run(path: &PathBuf, debug: bool) -> Result<(), CliError> {
    watch_and_run_with_config(path, debug, None)
}

/// Watch and run with an optional explicit config path (Issue #1006).
pub fn watch_and_run_with_config(
    path: &PathBuf,
    debug: bool,
    config_path: Option<&std::path::Path>,
) -> Result<(), CliError> {
    use std::time::{Duration, SystemTime};

    let canonical = fs::canonicalize(path)
        .map_err(|e| CliError::Io(format!("Failed to resolve path: {}", e)))?;

    let watch_dir = canonical
        .parent()
        .ok_or_else(|| CliError::Io("Cannot determine parent directory".to_string()))?;

    println!("[watch] Watching for changes... (press Ctrl+C to stop)");
    println!("[watch] Target: {}", canonical.display());
    println!();

    // Collect modification times for all relevant files in a directory
    let collect_mtimes =
        |dir: &std::path::Path, main_file: &std::path::Path| -> HashMap<PathBuf, SystemTime> {
            let mut times = HashMap::new();
            // Always include the main file
            if let Ok(meta) = fs::metadata(main_file) {
                if let Ok(mtime) = meta.modified() {
                    times.insert(main_file.to_path_buf(), mtime);
                }
            }
            // Scan directory for .hud and .hudhud files
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                            if ext == "hud" || ext == "hudhud" {
                                if let Ok(meta) = fs::metadata(&p) {
                                    if let Ok(mtime) = meta.modified() {
                                        times.insert(p, mtime);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            times
        };

    let mut prev_mtimes = collect_mtimes(watch_dir, &canonical);

    // Initial run
    println!("[watch] Running {}...", path.display());
    println!("{}", "=".repeat(60));
    if let Err(e) = run_file_vm_with_config(path, debug, config_path) {
        eprintln!("Error: {}", e);
    }
    println!("{}", "=".repeat(60));

    let debounce = Duration::from_millis(200);
    let poll_interval = Duration::from_millis(200);

    loop {
        std::thread::sleep(poll_interval);

        let current_mtimes = collect_mtimes(watch_dir, &canonical);
        let mut changed_file: Option<PathBuf> = None;

        // Check for any file that has a newer modification time or is new
        for (file, mtime) in &current_mtimes {
            match prev_mtimes.get(file) {
                Some(prev_mtime) if prev_mtime == mtime => {}
                _ => {
                    changed_file = Some(file.clone());
                    break;
                }
            }
        }

        if let Some(changed) = changed_file {
            // Debounce: wait and re-check to avoid triggering on partial writes
            std::thread::sleep(debounce);
            prev_mtimes = collect_mtimes(watch_dir, &canonical);

            println!();
            println!("[watch] Change detected: {}", changed.display());
            println!("[watch] Re-running {}...", path.display());
            println!("{}", "=".repeat(60));

            if let Err(e) = run_file_vm_with_config(path, debug, config_path) {
                eprintln!("Error: {}", e);
            }

            println!("{}", "=".repeat(60));
        } else {
            prev_mtimes = current_mtimes;
        }
    }
}

#[allow(dead_code)]
pub fn run_bytecode(path: &PathBuf, debug: bool) -> Result<(), CliError> {
    run_bytecode_with_config(path, debug, None)
}

/// Register all stdlib modules from shared-builtins on the given VM (#928).
///
/// Thin re-export of `hudhudscript_vm::register_vm_stdlib_modules`. The body
/// used to live here; it moved into the VM crate so external consumers
/// (test harness, FFI) can register the same module surface without
/// depending on the CLI bin crate. Kural 7 — single source of truth.
#[allow(dead_code)]
pub fn register_vm_stdlib_modules(vm: &mut VM) {
    hudhudscript_vm::register_vm_stdlib_modules(vm)
}

/// Run a source script via the VM path (parse → compile → VM execute).
///
/// This is the **default** execution path for `hudi run` (Issue: VM-first
/// default). Slightly faster than the AST walker on large recursive workloads
/// and enforces Kural 7 (single execution path).
///
/// For the legacy AST walker path, use [`run_file_with_config`].
#[allow(dead_code)]
pub fn run_file_vm(path: &PathBuf, debug: bool) -> Result<(), CliError> {
    run_file_vm_with_config(path, debug, None)
}

/// Run a source script via the VM path with an optional explicit config path.
pub fn run_file_vm_with_config(
    path: &PathBuf,
    debug: bool,
    config_path: Option<&std::path::Path>,
) -> Result<(), CliError> {
    // If a .hudb bytecode file was passed, delegate to the bytecode runner.
    if path.extension().and_then(|s| s.to_str()) == Some("hudb") {
        return run_bytecode_with_config(path, debug, config_path);
    }

    // Read source file
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;

    // Detect locale — mirror the interpreter path so Arabic numerals etc. work.
    let locale_str = detect_locale(&source);
    if locale_str != "default" {
        std::env::set_var("HUDHUD_LOCALE", locale_str);
    } else {
        std::env::remove_var("HUDHUD_LOCALE");
    }

    if debug {
        println!("Running (VM): {}", path.display());
        if locale_str != "default" {
            println!("Detected locale: {}", locale_str);
        }
    }

    // Parse
    let ast = parse(&source).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;

    if debug {
        println!("Parsed {} statements", ast.len());
    }

    // Compile
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;

    if debug {
        println!(
            "Compiled: {} constants, {} instructions",
            bytecode.constants.len(),
            bytecode.instructions.len()
        );
    }
    // Load hudhud.toml runtime config (Issue #446, #1006)
    let config = load_hudhud_config_with_path(debug, config_path);

    // Create VM with detected locale and configure
    let vm_locale = VM::detect_locale(&source);
    let mut vm = VM::with_locale(vm_locale);

    if config.runtime.fuel_limit > 0 {
        vm.with_fuel(config.runtime.fuel_limit);
    }
    vm.with_max_call_depth(config.runtime.max_recursion);
    vm.with_max_call_depth_ceiling(config.runtime.max_call_depth_hard_ceiling);
    vm.with_register_arena_kb(config.runtime.register_arena_kb);
    vm.with_max_builtin_iter(config.runtime.builtin_max_iter);
    vm.with_default_stack_bytes(config.runtime.default_stack_bytes);
    vm.set_toml_providers(config.providers.clone());
    vm.set_toml_config_object(build_config_value(&config));

    // ENV: allow network for provider calls via hudhud.toml [runtime]
    if config.runtime.allow_network {
        vm.allow_network();
    }

    // Register all stdlib modules (shared with run_bytecode_with_config)
    register_vm_stdlib_modules(&mut vm);

    // Execute — use a Tokio runtime in case any shared-builtins (http, tcp, ...)
    // need an async context, matching run_file_with_config.
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::Runtime(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        vm.execute(&bytecode)
            .map_err(|e| CliError::Runtime(format!("VM error: {}", e)))
    })?;

    Ok(())
}

/// Run bytecode with an optional explicit config path (Issue #1006).
fn run_bytecode_with_config(
    path: &PathBuf,
    debug: bool,
    config_path: Option<&std::path::Path>,
) -> Result<(), CliError> {
    if debug {
        println!("Running bytecode: {}", path.display());
    }

    // Read bytecode file
    let bytes =
        fs::read(path).map_err(|e| CliError::Io(format!("Failed to read bytecode file: {}", e)))?;

    // Deserialize bytecode
    let bytecode = Bytecode::from_bytes(&bytes)
        .map_err(|e| CliError::ParseCompile(format!("Failed to deserialize bytecode: {}", e)))?;

    if debug {
        println!("Bytecode version: {}", bytecode.version);
        println!("Constants: {}", bytecode.constants.len());
        println!("Instructions: {}", bytecode.instructions.len());
        println!("   Top-level instructions:");
        for (i, instruction) in bytecode.instructions.iter().enumerate() {
            println!("     [{}] {:?}", i, instruction);
        }
        println!("   Functions:");
        for (name, chunk) in bytecode.functions.borrow().iter() {
            println!("     Function: {} (local_count={}, local_names={:?})", name, chunk.local_count, chunk.local_names);
            for (i, instruction) in chunk.instructions.iter().enumerate() {
                println!("       [{}] {:?}", i, instruction);
            }
        }
    }

    // Try to read source file to detect locale — try both .hudhud and .hud
    let source_path = path.with_extension("hudhud");
    let source_path = if source_path.exists() {
        source_path
    } else {
        path.with_extension("hud")
    };
    let locale = if source_path.exists() {
        if let Ok(source) = fs::read_to_string(&source_path) {
            VM::detect_locale(&source)
        } else {
            hudhudscript_vm::OutputLocale::Default
        }
    } else {
        OutputLocale::Default
    };

    if locale == OutputLocale::Arabic {
        std::env::set_var("HUDHUD_LOCALE", "ar");
    } else {
        std::env::remove_var("HUDHUD_LOCALE");
    }

    // Load hudhud.toml runtime config (Issue #446, #1006)
    let config = load_hudhud_config_with_path(debug, config_path);

    // Execute on VM with detected locale
    let mut vm = VM::with_locale(locale);

    // Apply fuel limit from config
    if config.runtime.fuel_limit > 0 {
        vm.with_fuel(config.runtime.fuel_limit);
    }
    vm.with_max_call_depth(config.runtime.max_recursion);
    vm.with_register_arena_kb(config.runtime.register_arena_kb);
    vm.with_max_builtin_iter(config.runtime.builtin_max_iter);
    vm.with_max_call_depth_ceiling(config.runtime.max_call_depth_hard_ceiling);
    vm.with_default_stack_bytes(config.runtime.default_stack_bytes);
    vm.set_toml_providers(config.providers.clone());
    vm.set_toml_config_object(build_config_value(&config));

    if config.runtime.allow_network {
        vm.allow_network();
    }

    // Register stdlib modules from shared-builtins (#928) — Kural 7: single source.
    register_vm_stdlib_modules(&mut vm);

    vm.execute(&bytecode)
        .map_err(|e| CliError::Runtime(format!("VM error: {}", e)))?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Issue #661: Debugger integration
// ═══════════════════════════════════════════════════════════════════════════════

// ── config() builtin helper ────────────────────────────────────────────────
fn build_config_value(config: &HudHudConfig) -> Value16 {
    use std::collections::HashMap;
    let mut root = HashMap::new();

    // [runtime]
    let mut runtime = HashMap::new();
    runtime.insert("max_recursion".to_string(), Value16::int(config.runtime.max_recursion as i64));
    runtime.insert("fuel_limit".to_string(), Value16::int(config.runtime.fuel_limit as i64));
    runtime.insert("allow_network".to_string(), Value16::bool_(config.runtime.allow_network));
    root.insert("runtime".to_string(), Value16::object(runtime));

    // [providers]
    let mut providers = HashMap::new();
    for (name, fields) in &config.providers {
        let mut p = HashMap::new();
        for (k, v) in fields {
            p.insert(k.clone(), Value16::string(v.clone()));
        }
        providers.insert(name.clone(), Value16::object(p));
    }
    root.insert("providers".to_string(), Value16::object(providers));

    Value16::object(root)
}
