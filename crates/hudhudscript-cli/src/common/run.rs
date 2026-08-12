use hudhudscript_bytecode::Value16;
use crate::common::{detect_locale, load_hudhud_config_with_path, CliError, HudHudConfig};
use crate::common::provider::setup_provider_registry;
use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
#[cfg(feature = "telemetry")]
use crate::common::telemetry_writer::write_telemetry_json;
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_lang_directive, parse_with_recovery};
use hudhudscript_vm::{OutputLocale, VM};
use hudhudscript_modules::ModuleLoader;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

pub fn watch_and_run(path: &PathBuf, debug: bool) -> Result<(), CliError> {
    watch_and_run_with_config(path, debug, None, false)
}

/// Watch and run with an optional explicit config path (Issue #1006).
pub fn watch_and_run_with_config(
    path: &PathBuf,
    debug: bool,
    config_path: Option<&std::path::Path>,
    timing: bool,
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
    if let Err(e) = run_file_vm_with_config(path, debug, config_path, timing, None) {
        eprintln!("{}", crate::common::render_error(&e));
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

            if let Err(e) = run_file_vm_with_config(path, debug, config_path, timing, None) {
                eprintln!("{}", crate::common::render_error(&e));
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
    run_file_vm_with_config(path, debug, None, false, None)
}

/// Run a source script via the VM path with an optional explicit config path.
pub fn run_file_vm_with_config(
    path: &PathBuf,
    debug: bool,
    config_path: Option<&std::path::Path>,
    timing: bool,
    telemetry_json: Option<&std::path::Path>,
) -> Result<(), CliError> {
    let total_start = Instant::now();
    // If a .hudb bytecode file was passed, delegate to the bytecode runner.
    if path.extension().and_then(|s| s.to_str()) == Some("hudb") {
        return run_bytecode_with_config(path, debug, config_path);
    }

    // Read source file
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;
    let read_done = Instant::now();

    let canonical_script = fs::canonicalize(path)
        .map_err(|e| CliError::Io(format!("Failed to resolve path: {}", e)))?;
    let module_base = canonical_script
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    // Detect locale — directive (#!dil=tr) > script detection > default
    let directive_locale = hudhudscript_parser::parse_lang_directive(&source);
    let locale_str = directive_locale.unwrap_or_else(|| detect_locale(&source).to_string());
    if locale_str != "default" {
        std::env::set_var("HUDHUD_LOCALE", &locale_str);
    }
    // ERR-5: keep external HUDHUD_LOCALE if no directive (was: remove_var deleted external env)

    if debug {
        println!("Running (VM): {}", path.display());
        if locale_str != "default" {
            println!("Detected locale: {}", locale_str);
        }
    }

    // Parse
    let parse_start = Instant::now();
    let ast = parse(&source).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;
    let parse_done = Instant::now();

    if debug {
        println!("Parsed {} statements", ast.len());
    }

    // Compile
    let compile_start = Instant::now();
    let mut compiler = Compiler::new();
    compiler.set_module_base_dir(module_base.clone());
    let bytecode = compiler.compile(&ast).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;
    let compile_done = Instant::now();

    if debug {
        println!(
            "Compiled: {} constants, {} instructions",
            bytecode.constants.len(),
            bytecode.instructions.len()
        );
    }
    // Load hudhud.toml runtime config (Issue #446, #1006)
    let config = load_hudhud_config_with_path(debug, config_path);

    // ISSUE-1: apply GC tuning from [gc] before any VM execution.
    hudhudscript_bytecode::gc::set_gc_tuning(config.gc.min_objects, config.gc.growth_factor);

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
    vm.with_provider_timeout_secs(config.runtime.provider_timeout_secs);
    vm.set_toml_providers(config.providers.clone());
    vm.set_toml_config_object(build_config_value(&config));

    // ENV: sandbox capabilities (network for provider calls, process for stdio
    // MCP) via hudhud.toml [runtime]
    apply_runtime_capabilities(&mut vm, &config.runtime);

    // Provider registry: shared with REPL (Kural 7 — single source).
    // Uses OLLAMA_BASE_URL env var for ollama endpoint; no hardcoded model/URL.
    match setup_provider_registry(debug, None) {
        Ok(registry) => { vm.set_provider_registry(registry); }
        Err(e) => { if debug { eprintln!("Provider registry: {}", e); } }
    }

    // HOST-4: apply [host_access] policy to VM
    if let Some(ref host_access) = config.host_access {
        vm.set_host_access_policy(host_access.to_policy());
    }

    // Register all stdlib modules (shared with run_bytecode_with_config)
    register_vm_stdlib_modules(&mut vm);

    vm.set_module_resolver(Box::new(ModuleLoader::new(module_base.clone())));

    // TOKIO T-2+T-3: single conditional current_thread runtime.
    let needs_async = bytecode.needs_async;
    #[cfg(feature = "mcp")]
    let needs_mcp = !config.mcp.servers.is_empty();
    #[cfg(not(feature = "mcp"))]
    let needs_mcp = false;

    let exec_start = Instant::now();
    let exec_result: Result<(), CliError> = if needs_async || needs_mcp {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| CliError::Runtime(format!("Runtime: {}", e)))?;
        rt.block_on(async {
            if needs_mcp {
                match crate::common::provider::setup_mcp_clients(&config.mcp.servers, debug).await {
                    Ok(mcp_clients) => {
                        for (name, client) in mcp_clients { vm.register_mcp_client(name, client); }
                    }
                    Err(e) => { if debug { eprintln!("⚠ MCP: {}", e); } }
                }
            }
            let result = vm.execute(&bytecode)
                .map_err(|e| CliError::Runtime(format!("VM error: {}", e)));
            vm.shutdown_mcp_clients().await;
            result
        })
    } else {
        vm.execute(&bytecode)
            .map_err(|e| CliError::Runtime(format!("VM error: {}", e)))
    };
    let exec_done = Instant::now();

    // GATE-2: Write telemetry JSON if requested (after exec, before propagate)
    if let Some(out_path) = telemetry_json {
        #[cfg(feature = "telemetry")]
        {
            write_telemetry_json(&vm, out_path, exec_result.is_ok())?;
        }
        #[cfg(not(feature = "telemetry"))]
        {
            return Err(CliError::Runtime(
                "--telemetry-json requires a telemetry-enabled build. Rebuild with: cargo build --features telemetry".into()
            ));
        }
    }

    exec_result?;

    if timing {
        let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
        let read_ms = read_done.duration_since(total_start).as_secs_f64() * 1000.0;
        let parse_ms = parse_done.duration_since(parse_start).as_secs_f64() * 1000.0;
        let compile_ms = compile_done.duration_since(compile_start).as_secs_f64() * 1000.0;
        let exec_ms = exec_done.duration_since(exec_start).as_secs_f64() * 1000.0;
        eprintln!(
            "timing: read={:.3}ms parse={:.3}ms compile={:.3}ms vm-exec={:.3}ms total={:.3}ms",
            read_ms, parse_ms, compile_ms, exec_ms, total_ms
        );
    }

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

    let canonical_script = fs::canonicalize(path)
        .map_err(|e| CliError::Io(format!("Failed to resolve path: {}", e)))?;
    let module_base = canonical_script
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

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
        for (name, chunk) in bytecode.function_names.borrow().iter().map(|(n, &i)| (n.clone(), bytecode.functions.borrow()[i].clone())) {
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

    // ISSUE-1: apply GC tuning from [gc] before any VM execution.
    hudhudscript_bytecode::gc::set_gc_tuning(config.gc.min_objects, config.gc.growth_factor);

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
    vm.with_provider_timeout_secs(config.runtime.provider_timeout_secs);
    vm.set_toml_providers(config.providers.clone());
    vm.set_toml_config_object(build_config_value(&config));

    apply_runtime_capabilities(&mut vm, &config.runtime);

    // HOST-4: apply [host_access] policy to VM
    if let Some(ref host_access) = config.host_access {
        vm.set_host_access_policy(host_access.to_policy());
    }

    // Register stdlib modules from shared-builtins (#928) — Kural 7: single source.
    register_vm_stdlib_modules(&mut vm);

    vm.set_module_resolver(Box::new(ModuleLoader::new(module_base.clone())));

    vm.execute(&bytecode)
        .map_err(|e| CliError::Runtime(format!("VM error: {}", e)))?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Issue #661: Debugger integration
// ═══════════════════════════════════════════════════════════════════════════════

// ── config() builtin helper ────────────────────────────────────────────────
/// Apply the `[runtime]` sandbox capabilities from `hudhud.toml` to the VM.
///
/// Single source (Kural 7) for both the `run_file_vm_with_config` and the
/// `run_bytecode_with_config` paths, which previously each carried their own
/// copy of the `allow_network` block — so a capability added to one silently
/// stayed missing from the other.
///
/// `allow_process` gates stdio MCP servers (which spawn a child process);
/// `allow_network` gates SSE MCP servers and provider/LLM calls. Both default
/// to false and are opt-in via `hudhud.toml`.
fn apply_runtime_capabilities(vm: &mut VM, runtime: &crate::common::RuntimeConfig) {
    if runtime.allow_network {
        vm.allow_network();
    }
    if runtime.allow_process {
        vm.allow_process();
    }
    // M2: unencrypted-http opt-in — SSE MCP'nin http:// + loopback muafiyeti.
    // `allow_network`'ten ayrı izin (biri ağ, diğeri şifresiz http).
    if runtime.allow_insecure_http {
        vm.allow_insecure_http();
    }
    // Privilege escalation (`sudo`) is a separate grant from process spawning:
    // the firewall and apt modules change host state, so they stay off unless the
    // project asks for them explicitly.
    if runtime.allow_privileged {
        hudhudscript_bytecode::privileged_ops::allow_privileged_ops();
    }
}

fn build_config_value(config: &HudHudConfig) -> Value16 {
    use std::collections::HashMap;
    let mut root = hudhudscript_bytecode::ObjMap::default();

    // [runtime]
    let mut runtime = hudhudscript_bytecode::ObjMap::default();
    runtime.insert("max_recursion".to_string(), Value16::int(config.runtime.max_recursion as i64));
    runtime.insert("fuel_limit".to_string(), Value16::int(config.runtime.fuel_limit as i64));
    runtime.insert("allow_network".to_string(), Value16::bool_(config.runtime.allow_network));
    runtime.insert("allow_process".to_string(), Value16::bool_(config.runtime.allow_process));
    runtime.insert("allow_insecure_http".to_string(), Value16::bool_(config.runtime.allow_insecure_http));
    runtime.insert("allow_privileged".to_string(), Value16::bool_(config.runtime.allow_privileged));
    root.insert("runtime".to_string(), Value16::object(runtime));

    // [providers]
    let mut providers = hudhudscript_bytecode::ObjMap::default();
    for (name, fields) in &config.providers {
        let mut p = hudhudscript_bytecode::ObjMap::default();
        for (k, v) in fields {
            p.insert(k.clone(), Value16::string(v.clone()));
        }
        providers.insert(name.clone(), Value16::object(p));
    }
    root.insert("providers".to_string(), Value16::object(providers));

    Value16::object(root)
}
