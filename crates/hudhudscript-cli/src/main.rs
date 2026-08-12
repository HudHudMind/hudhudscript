//! HudHudScript CLI
//!
//! Command-line interface for HudHudScript.
//! Main binary: `hudhud` with subcommands: run, compile, repl, format, lint, check, package.

// P7.2 — swap the system allocator for mimalloc.  Callgrind profiling of the
// interpreter hot path (`fib(30)`) showed ~24% of instructions in malloc/free;
// mimalloc reduces that to ~11% and is safe (same API, thread-safe).
// G0: allow sysalloc-profile feature for heaptrack/valgrind.
#[cfg(not(feature = "sysalloc-profile"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::path::PathBuf;
use std::process;

use clap::{Parser as ClapParser, Subcommand};
use hudhudscript_cli::common::*;

mod startup;

#[derive(ClapParser)]
#[command(name = "hudhud")]
#[command(version, about = "HudHudScript - MCP-based orchestration language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a HudHudScript file
    Run {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Watch mode - rerun on file changes
        #[arg(short, long)]
        watch: bool,

        /// UI framework: web, tauri, flutter, wasm, gtk, qt, iced
        #[arg(long)]
        ui: Option<String>,

        /// Enable strict type checking (Issue #866 TYPE-001)
        #[arg(long)]
        strict: bool,

        /// Print GC statistics after execution
        #[arg(long)]
        gc_stats: bool,

        /// Print timing breakdown: parse | compile | VM-exec | total
        #[arg(long)]
        timing: bool,

        /// Write telemetry counters to JSON file (requires telemetry feature)
        #[cfg(feature = "telemetry")]
        #[arg(long, value_name = "PATH")]
        telemetry_json: Option<PathBuf>,
    },

    /// Deploy a HudHudScript app
    Deploy {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Deploy adapter: github, gitlab, docker, vercel, k8s
        #[arg(short, long)]
        adapter: Option<String>,

        /// Dry run — generate artifacts without deploying
        #[arg(long)]
        dry_run: bool,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,
    },

    /// Compile a HudHudScript file to bytecode
    Compile {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output file path (defaults to input with .hudb extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show detailed compilation info
        #[arg(short, long)]
        verbose: bool,

        /// Enable strict type checking (Issue #866 TYPE-001)
        #[arg(long)]
        strict: bool,
    },

    /// Start interactive REPL
    Repl {
        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Load script file before starting REPL
        #[arg(short, long)]
        load: Option<PathBuf>,
    },

    /// Check syntax without running
    Check {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show detailed AST
        #[arg(long)]
        ast: bool,

        /// Enable strict type checking (Issue #866 TYPE-001)
        #[arg(long)]
        strict: bool,
    },

    /// Format a HudHudScript file or directory
    Format {
        /// Path to a script file or directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Write formatted output to file
        #[arg(short, long)]
        write: bool,

        /// Check if files are formatted (exit 1 if not), without modifying them
        #[arg(long)]
        check: bool,
    },

    /// Lint a HudHudScript file for style and correctness issues
    Lint {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Package management (delegates to hudp)
    Package {
        /// Arguments passed through to the package manager
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Show version information
    Version {
        /// Show detailed version info
        #[arg(long)]
        detailed: bool,
    },

    /// Show detailed system and build information
    Info,

    /// Start DAP debug server for IDE integration (Issue #661)
    Dap {
        /// Path to the script file (.hud or .hudhud)
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Run a script with an interactive debugger (Issue #661)
    Debug {
        /// Path to the script file (.hud or .hudhud)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Breakpoints in format file:line (can be repeated)
        #[arg(short, long)]
        breakpoint: Vec<String>,

        /// Stop on the first statement
        #[arg(long)]
        stop_on_entry: bool,
    },

    /// Start Language Server Protocol (LSP) server for IDE integration
    Lsp {
        /// Transport: stdio (default) or tcp
        #[arg(long, default_value = "stdio")]
        transport: String,

        /// TCP port (only for tcp transport)
        #[arg(long, default_value = "9257")]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(stack_mb) = startup::requested_thread_stack_mb() {
        startup::run_with_stack(stack_mb, move || run_cli(cli));
    } else {
        run_cli(cli);
    }
}

fn run_cli(cli: Cli) {

    // Set up logging based on verbose flag
    if cli.verbose {
        println!("Verbose mode enabled");
    }

    let config_path = cli.config.as_deref();

    match cli.command {
        Some(Commands::Run {
            file,
            debug,
            watch,
            ui,
            strict,
            gc_stats,
            timing,
            #[cfg(feature = "telemetry")]
            telemetry_json,
        }) => {
            #[cfg(not(feature = "telemetry"))]
            let telemetry_json: Option<std::path::PathBuf> = None;
            // Early reject: --watch, --ui, and .hudb paths don't support telemetry
            #[cfg(feature = "telemetry")]
            if telemetry_json.is_some() {
                if watch {
                    eprintln!("Error: --telemetry-json is not supported with --watch");
                    process::exit(1);
                }
                if ui.is_some() {
                    eprintln!("Error: --telemetry-json is not supported with --ui");
                    process::exit(1);
                }
                if file.extension().and_then(|s| s.to_str()) == Some("hudb") {
                    eprintln!("Error: --telemetry-json is not supported with .hudb bytecode files");
                    process::exit(1);
                }
            }
            if watch {
                if let Err(e) = watch_and_run_with_config(
                    &file,
                    debug || cli.verbose,
                    config_path,
                    timing,
                )
                {
                    eprintln!("{}", render_error(&e));
                    process::exit(e.exit_code());
                }
            } else if let Some(framework) = ui {
                if let Err(e) =
                    run_ui_with_config(&file, &framework, debug || cli.verbose, config_path)
                {
                    eprintln!("{}", render_error(&e));
                    process::exit(e.exit_code());
                }
            } else if let Err(e) = {
                if strict {
                    eprintln!("Warning: --strict type checking is not yet wired into the VM path; running without it.");
                }
                run_file_vm_with_config(&file, debug || cli.verbose, config_path, timing, telemetry_json.as_deref())
            } {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
            if gc_stats {
                let stats = hudhudscript_bytecode::gc::stats();
                println!("GC stats: {:?}", stats);
            }
        }
        Some(Commands::Deploy {
            file,
            adapter,
            dry_run,
            debug,
        }) => {
            if let Err(e) = run_deploy_with_config(
                &file,
                adapter.as_deref(),
                dry_run,
                debug || cli.verbose,
                config_path,
            ) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Compile {
            file,
            output,
            verbose,
            strict,
        }) => {
            if let Err(e) = compile_file(&file, output, verbose || cli.verbose, strict, &Default::default()) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Repl { debug, load }) => {
            if let Err(e) = run_repl(debug || cli.verbose, load) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Check { file, ast, strict }) => {
            let config = load_hudhud_config_with_path(false, None);
            if let Err(e) = check_file(&file, ast, strict, &config.lint) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Format { path, write, check }) => {
            if let Err(e) = format_path(&path, write, check) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Lint { file }) => {
            if let Err(e) = lint_file(&file) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Package { args }) => {
            // Delegate to hudp by re-executing with the package manager
            let status = process::Command::new("hudp").args(&args).status();
            match status {
                Ok(s) if s.success() => {}
                Ok(s) => process::exit(s.code().unwrap_or(1)),
                Err(e) => {
                    eprintln!("Error: failed to run hudp: {}", e);
                    eprintln!("Hint: make sure hudp is installed (cargo install --path crates/hudhudscript-cli)");
                    process::exit(3); // IO error — hudp binary not found
                }
            }
        }
        Some(Commands::Version { detailed }) => {
            show_version(detailed);
        }
        Some(Commands::Info) => {
            show_info();
        }
        Some(Commands::Dap { file }) => {
            if let Err(e) = run_dap_server(&file) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Debug {
            file,
            breakpoint,
            stop_on_entry,
        }) => {
            if let Err(e) = run_debug_with_config(&file, &breakpoint, stop_on_entry, config_path) {
                eprintln!("{}", render_error(&e));
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Lsp { transport, port }) => {
            println!(
                "Starting HudHudScript LSP server ({} transport)...",
                transport
            );
            if let Err(e) = run_lsp_server(&transport, port) {
                eprintln!("LSP error: {}", e);
                process::exit(e.exit_code());
            }
        }
        None => {
            // No command provided, show help
            println!("HudHudScript v{}", env!("CARGO_PKG_VERSION"));
            println!("Use --help for more information");
            println!();
            println!("Quick start:");
            println!("  hudhud run script.hud       # Run a script");
            println!("  hudhud compile script.hud   # Compile to bytecode");
            println!("  hudhud repl                 # Start REPL");
            println!("  hudhud check script.hud     # Check syntax");
            println!("  hudhud format script.hud    # Format code");
            println!("  hudhud lint script.hud      # Lint code");
            println!("  hudhud debug script.hud     # Interactive debugger");
            println!("  hudhud dap script.hud       # DAP server (IDE)");
            println!("  hudhud lsp                  # Start LSP server");
            println!("  hudhud package install       # Package management");
        }
    }
}
