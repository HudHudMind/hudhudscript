use crate::common::{
    load_hudhud_config_with_path, register_vm_stdlib_modules, CliError, HudHudConfig,
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

pub fn run_deploy(
    path: &PathBuf,
    adapter_name: Option<&str>,
    dry_run: bool,
    debug: bool,
) -> Result<(), CliError> {
    run_deploy_with_config(path, adapter_name, dry_run, debug, None)
}

/// Deploy with an optional explicit config path (Issue #1006).
#[allow(dead_code)]
pub fn run_deploy_with_config(
    path: &PathBuf,
    adapter_name: Option<&str>,
    dry_run: bool,
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
        println!("[deploy] Parsing: {}", path.display());
    }

    // Execute script via compile + VM (Issue #1006; Faz 4 migration).
    let config = load_hudhud_config_with_path(debug, config_path);
    let mut compiler = Compiler::new();
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

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CliError::Runtime(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        vm.execute(&bytecode).map_err(|e| {
            let unified: hudhudscript_errors::Error = e;
            CliError::Runtime(unified.render_full())
        })
    })?;

    // Resolve adapter
    let adapter_str = adapter_name.unwrap_or("github");
    let adapter_enum = Adapter::parse(adapter_str)
        .ok_or_else(|| CliError::Runtime(format!("Unknown deploy adapter: {}", adapter_str)))?;

    let adapter = create_adapter(&adapter_enum)
        .map_err(|e| CliError::Runtime(format!("Failed to create deploy adapter: {}", e)))?;

    if debug {
        println!("[deploy] Adapter '{}' created", adapter.name());
    }

    // Build a minimal DeployPlan from the script's filename.
    //
    // v0.4.47.9 — Issue #810: The full implementation should walk the
    // interpreter environment for `deploy { ... }` declarations and extract
    // targets, pipelines, docker, and kubernetes configs. Currently builds
    // an empty plan so the CLI command runs end-to-end and the deploy
    // adapters can return their honest errors.
    let plan = hudhudscript_deploy_core::DeployPlan {
        app_name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };

    // Generate artifacts
    let artifacts = adapter
        .generate(&plan)
        .map_err(|e| CliError::Runtime(format!("Artifact generation failed: {}", e)))?;

    for artifact in &artifacts {
        if dry_run {
            println!("[deploy] Would generate: {}", artifact.filename);
            println!("{}", artifact.content);
        } else {
            println!("[deploy] Generated: {}", artifact.filename);
            // Write artifact to disk
            if let Some(parent) = std::path::Path::new(&artifact.filename).parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&artifact.filename, &artifact.content).map_err(|e| {
                CliError::Io(format!("Failed to write {}: {}", artifact.filename, e))
            })?;
        }
    }

    if artifacts.is_empty() {
        println!("[deploy] No artifacts generated (stub adapter)");
    }

    // Deploy (unless dry-run)
    if !dry_run {
        let result = adapter
            .deploy(&plan)
            .map_err(|e| CliError::Runtime(format!("Deploy failed: {}", e)))?;
        if result.success {
            println!("[deploy] Success: {}", result.message);
            if let Some(url) = result.url {
                println!("[deploy] URL: {}", url);
            }
        } else {
            return Err(CliError::Runtime(format!(
                "Deploy failed: {}",
                result.message
            )));
        }
    }

    Ok(())
}
