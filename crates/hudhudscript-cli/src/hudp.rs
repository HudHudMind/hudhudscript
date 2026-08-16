//! hudp — HudHudScript Package Manager CLI
//!
//! Provides `hudp init`, `hudp build`, `hudp run`, and the existing package
//! management commands (install, add, remove, update, search, publish, audit).

// P7.2 — swap the system allocator for mimalloc (same as hudhud/hudi/hudc).
// G0: allow sysalloc-profile feature for heaptrack/valgrind.
#[cfg(not(feature = "sysalloc-profile"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Share the `common` module with hudi/hudhud/hudc bins so `hudp run` can
// reuse `register_vm_stdlib_modules` etc.

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use colored::Colorize;
use hudhudscript_cli::common::*;
use hudhudscript_package::{
    get_exported_modules, is_application_package, HudhudConfig, PackageBuilder, PackageManager,
    PackageRunner,
};

#[derive(Parser)]
#[command(name = "hudp")]
#[command(version, about = "HudHudScript Package Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new HudHudScript project
    Init {
        /// Project name (also used as directory name)
        name: String,

        /// Create a library package instead of an application
        #[arg(long)]
        library: bool,

        /// Target directory (defaults to `./<name>`)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Build the package — validate sources and create a .hudpkg bundle
    Build {
        /// Package directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Run the package entry point (main.hud)
    Run {
        /// Package directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,
    },

    /// Install all dependencies from hudhud.toml
    Install {
        /// Force re-install even if cached
        #[arg(short, long)]
        force: bool,
    },

    /// Add a dependency
    Add {
        /// Package name
        package: String,

        /// Version constraint (e.g. "^1.0")
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Remove a dependency
    Remove {
        /// Package name
        package: String,
    },

    /// Update all dependencies to latest compatible versions
    Update,

    /// Search the registry
    Search {
        /// Search query
        query: String,
    },

    /// Publish the package to the registry
    Publish {
        /// Perform a dry run without actually publishing
        #[arg(long)]
        dry_run: bool,
    },

    /// Audit dependencies for known vulnerabilities
    Audit,

    /// List exported modules of a library package
    Modules {
        /// Package directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "error:".red().bold(), e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> std::result::Result<(), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;

    match cli.command {
        // ── init ─────────────────────────────────────────────────────────
        Commands::Init {
            name,
            library,
            path,
        } => {
            let target = path.unwrap_or_else(|| name.clone());
            let config = HudhudConfig::default();
            let pm = PackageManager::new(config)
                .map_err(|e| format!("Failed to create package manager: {}", e))?;
            rt.block_on(async {
                if library {
                    pm.init_library(&name, &target).await
                } else {
                    pm.init(&name, &target).await
                }
            })
            .map_err(|e| format!("{}", e))?;
        }

        // ── build ────────────────────────────────────────────────────────
        Commands::Build { path } => {
            let dir = path.unwrap_or_else(|| PathBuf::from("."));
            let builder = PackageBuilder::new(&dir).map_err(|e| format!("{}", e))?;
            let result = builder.build().map_err(|e| format!("{}", e))?;

            if cli.verbose {
                println!(
                    "   Sources: {}, Archive size: {} bytes",
                    result.source_file_count, result.total_size
                );
            }
        }

        // ── run ──────────────────────────────────────────────────────────
        Commands::Run { path, debug } => {
            let dir = path.unwrap_or_else(|| PathBuf::from("."));
            let runner = PackageRunner::new(&dir).map_err(|e| format!("{}", e))?;

            if !is_application_package(&runner.config) {
                return Err("This is a library package and has no entry point to run.".into());
            }

            let entry = runner.entry_point().map_err(|e| format!("{}", e))?;

            if debug || cli.verbose {
                println!("{} Entry point: {}", ">>".green(), entry.display());
                let providers = runner.ai_providers();
                if !providers.is_empty() {
                    println!("   AI providers: {}", providers.join(", "));
                }
                let servers = runner.mcp_servers();
                if !servers.is_empty() {
                    println!("   MCP servers:  {}", servers.join(", "));
                }
            }

            // Read and execute the entry point through the compile + VM
            // path (Faz 5 — interpreter crate is being removed).
            let source = std::fs::read_to_string(&entry)
                .map_err(|e| format!("Failed to read {}: {}", entry.display(), e))?;

            let ast =
                hudhudscript_parser::parse(&source).map_err(|e| format!("Parse error: {}", e))?;

            let canonical_script = std::fs::canonicalize(&entry).unwrap_or(entry.clone());
            let module_base = canonical_script
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            let mut compiler = hudhudscript_compiler::Compiler::new();
            compiler.set_module_base_dir(module_base.clone());
            let bytecode = compiler
                .compile(&ast)
                .map_err(|e| format!("Compile error: {}", e))?;

            let mut vm = hudhudscript_vm::VM::new();
            register_vm_stdlib_modules(&mut vm);
            vm.set_current_file(entry.to_string_lossy().to_string());

            // TOKIO T-2: conditional runtime
            #[cfg(not(feature = "mcp"))]
            {
                vm.execute(&bytecode)
                    .map_err(|e| format!("Runtime: {}", e))?;
            }
            #[cfg(feature = "mcp")]
            {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("Runtime: {}", e))?;
                rt.block_on(async {
                    let servers = std::collections::HashMap::new();
                    match setup_mcp_clients(&servers, debug || cli.verbose).await {
                        Ok(c) => {
                            for (n, c) in c {
                                vm.register_mcp_client(n, c);
                            }
                        }
                        Err(e) => {
                            if debug || cli.verbose {
                                eprintln!("⚠ MCP: {}", e);
                            }
                        }
                    }
                    let r = vm.execute(&bytecode).map_err(|e| format!("Runtime: {}", e));
                    vm.shutdown_mcp_clients().await;
                    r
                })?;
            }

            if debug {
                println!("{} Script completed", ">>".green());
            }
        }

        // ── install ──────────────────────────────────────────────────────
        Commands::Install { force: _ } => {
            let config = load_config()?;
            let pm = PackageManager::new(config).map_err(|e| format!("{}", e))?;
            rt.block_on(pm.install()).map_err(|e| format!("{}", e))?;
        }

        // ── add ──────────────────────────────────────────────────────────
        Commands::Add { package, version } => {
            let config = load_config()?;
            let pm = PackageManager::new(config).map_err(|e| format!("{}", e))?;
            rt.block_on(pm.add(&package, version.as_deref()))
                .map_err(|e| format!("{}", e))?;
        }

        // ── remove ───────────────────────────────────────────────────────
        Commands::Remove { package } => {
            let config = load_config()?;
            let pm = PackageManager::new(config).map_err(|e| format!("{}", e))?;
            rt.block_on(pm.remove(&package))
                .map_err(|e| format!("{}", e))?;
        }

        // ── update ───────────────────────────────────────────────────────
        Commands::Update => {
            let config = load_config()?;
            let pm = PackageManager::new(config).map_err(|e| format!("{}", e))?;
            rt.block_on(pm.update()).map_err(|e| format!("{}", e))?;
        }

        // ── search ───────────────────────────────────────────────────────
        Commands::Search { query } => {
            let config = HudhudConfig::default();
            let pm = PackageManager::new(config).map_err(|e| format!("{}", e))?;
            let results = rt
                .block_on(pm.search(&query))
                .map_err(|e| format!("{}", e))?;
            if results.is_empty() {
                println!("No packages found for '{}'", query);
            } else {
                for pkg in &results {
                    println!(
                        "  {} v{} — {}",
                        pkg.name.bold(),
                        pkg.latest_version,
                        pkg.description
                    );
                }
            }
        }

        // ── publish ──────────────────────────────────────────────────────
        Commands::Publish { dry_run } => {
            let config = load_config()?;
            let pm = PackageManager::new(config).map_err(|e| format!("{}", e))?;
            let opts = hudhudscript_package::PublishOptions {
                dry_run,
                allow_dirty: false,
            };
            rt.block_on(pm.publish(opts))
                .map_err(|e| format!("{}", e))?;
        }

        // ── audit ────────────────────────────────────────────────────────
        Commands::Audit => {
            let config = load_config()?;
            let pm = PackageManager::new(config).map_err(|e| format!("{}", e))?;
            let issues = rt.block_on(pm.audit()).map_err(|e| format!("{}", e))?;
            if issues.is_empty() {
                println!("{} No vulnerabilities found.", ">>".green());
            } else {
                for issue in &issues {
                    println!("  {} {}", "!!".red(), issue);
                }
            }
        }

        // ── modules ──────────────────────────────────────────────────────
        Commands::Modules { path } => {
            let dir = path.unwrap_or_else(|| PathBuf::from("."));
            let modules = get_exported_modules(&dir);
            if modules.is_empty() {
                println!("No exported modules found.");
            } else {
                println!("Exported modules:");
                for m in &modules {
                    println!("  - {}", m);
                }
            }
        }
    }

    Ok(())
}

/// Load hudhud.toml from the current directory.
fn load_config() -> std::result::Result<HudhudConfig, String> {
    HudhudConfig::load("hudhud.toml").map_err(|e| format!("Failed to load hudhud.toml: {}", e))
}
