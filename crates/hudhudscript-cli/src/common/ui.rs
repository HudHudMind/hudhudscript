use crate::common::{
    create_bridge, load_hudhud_config_with_path, register_vm_stdlib_modules, CliError, Framework,
    HudHudConfig,
};
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

pub fn run_ui(path: &PathBuf, framework_name: &str, debug: bool) -> Result<(), CliError> {
    run_ui_with_config(path, framework_name, debug, None)
}

/// Run UI with an optional explicit config path (Issue #1006).
#[allow(dead_code)]
pub fn run_ui_with_config(
    path: &PathBuf,
    framework_name: &str,
    debug: bool,
    config_path: Option<&std::path::Path>,
) -> Result<(), CliError> {
    let source = fs::read_to_string(path)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;

    let ast = parse(&source).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;

    if debug {
        println!(
            "[ui] Running: {} with framework: {}",
            path.display(),
            framework_name
        );
    }

    // Resolve framework
    let framework = Framework::parse(framework_name)
        .ok_or_else(|| CliError::Runtime(format!("Unknown UI framework: {}", framework_name)))?;

    // Create bridge
    let mut bridge = create_bridge(&framework)
        .map_err(|e| CliError::Runtime(format!("Failed to create UI bridge: {}", e)))?;

    // Init bridge
    bridge
        .init()
        .map_err(|e| CliError::Runtime(format!("Failed to init UI bridge: {}", e)))?;

    if debug {
        println!("[ui] Bridge '{}' initialized", bridge.name());
    }

    // Load config (Issue #1006)
    let config = load_hudhud_config_with_path(debug, config_path);

    // Execute script via compile + VM (Faz 4 migration — interpreter
    // crate is on the path to removal).
    let canonical_script = fs::canonicalize(path)
        .map_err(|e| CliError::Io(format!("Failed to resolve path: {}", e)))?;
    let module_base = canonical_script
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    let mut compiler = Compiler::new();
    compiler.set_module_base_dir(module_base.clone());
    let bytecode = compiler.compile(&ast).map_err(|e| {
        let unified: hudhudscript_errors::Error = e;
        CliError::ParseCompile(unified.render_full())
    })?;

    let mut vm = if config.runtime.fuel_limit > 0 {
        let mut v = VM::new();
        v.with_fuel(config.runtime.fuel_limit);
        v
    } else {
        VM::new()
    };
    register_vm_stdlib_modules(&mut vm);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::Runtime(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        vm.execute(&bytecode).map_err(|e| {
            let unified: hudhudscript_errors::Error = e;
            CliError::Runtime(unified.render_full())
        })
    })?;

    if debug {
        println!("[ui] Script executed");
    }

    // Build a minimal App from interpreter environment.
    //
    // v0.4.47.9 — Issue #810: Currently the CLI builds an empty App with no
    // screens or components. The full implementation needs to walk the
    // interpreter environment to find UiApp declarations and extract their
    // screens/components into the App struct. Tracked separately as a
    // dedicated CLI feature; this minimal version exists so the `ui` command
    // works end-to-end with the bridge layer (which itself returns
    // BridgeError::Unsupported for stub frameworks — see #799).
    let app = hudhudscript_ui_core::App {
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("App")
            .to_string(),
        entry_screen: "Main".to_string(),
        screens: vec![],
        components: vec![],
    };

    // Render
    bridge
        .render(&app)
        .map_err(|e| CliError::Runtime(format!("Render error: {}", e)))?;

    if debug {
        println!("[ui] App rendered");
    }

    // Event loop (poll until no more events or bridge signals shutdown)
    loop {
        match bridge.poll_event() {
            Ok(Some(event)) => {
                if debug {
                    println!("[ui] Event: {:?}", event);
                }
                // Event dispatch will be expanded when state management is wired
            }
            Ok(None) => break, // No more events (stub always returns None)
            Err(e) => {
                eprintln!("[ui] Event error: {}", e);
                break;
            }
        }
    }

    // Shutdown
    bridge
        .shutdown()
        .map_err(|e| CliError::Runtime(format!("Shutdown error: {}", e)))?;

    if debug {
        println!("[ui] Bridge shutdown complete");
    }

    Ok(())
}
