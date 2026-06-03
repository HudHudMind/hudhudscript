//! hudi - HudHud Interpreter
//!
//! Standalone interpreter CLI for HudHudScript.
//! Supports both .hud and .hudhud file extensions.

// P7.2 — swap the system allocator for mimalloc.  Callgrind profiling of the
// interpreter hot path (`fib(30)`) showed ~24% of instructions in malloc/free;
// mimalloc reduces that to ~11% and is safe (same API, thread-safe).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

mod common;
use common::*;

#[derive(Parser)]
#[command(name = "hudi")]
#[command(version, about = "HudHud Interpreter - Run HudHudScript files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Script file to run (default action)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a HudHudScript file (uses the VM by default)
    Run {
        /// Path to the script file (.hud or .hudhud)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,

        /// Deprecated alias — VM is the only runtime; accepting the
        /// flag silently so older scripts / docs don't break.
        #[arg(long = "use-vm", hide = true)]
        use_vm: bool,
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
        /// Path to the script file (.hud or .hudhud)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show detailed AST
        #[arg(long)]
        ast: bool,
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run {
            file,
            debug,
            use_vm: _use_vm,
        }) => {
            let dbg = debug || cli.debug;
            if let Err(e) = run_file_vm(&file, dbg) {
                eprintln!("Error: {}", e);
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Repl { debug, load }) => {
            if let Err(e) = run_repl(debug || cli.debug, load) {
                eprintln!("Error: {}", e);
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Check { file, ast }) => {
            if let Err(e) = check_file(&file, ast, false, &Default::default()) {
                eprintln!("Error: {}", e);
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Format { path, write, check }) => {
            if let Err(e) = format_path(&path, write, check) {
                eprintln!("Error: {}", e);
                process::exit(e.exit_code());
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
                eprintln!("Error: {}", e);
                process::exit(e.exit_code());
            }
        }
        Some(Commands::Debug {
            file,
            breakpoint,
            stop_on_entry,
        }) => {
            if let Err(e) = run_debug(&file, &breakpoint, stop_on_entry) {
                eprintln!("Error: {}", e);
                process::exit(e.exit_code());
            }
        }
        None => {
            // Compile + VM is the sole runtime after Faz 5.
            if let Some(file) = cli.file {
                if let Err(e) = run_file_vm(&file, cli.debug) {
                    eprintln!("Error: {}", e);
                    process::exit(e.exit_code());
                }
            } else {
                println!("HudHudScript v{}", env!("CARGO_PKG_VERSION"));
                println!("Use --help for more information");
                println!();
                println!("Quick start:");
                println!("  hudi script.hud              # Run script");
                println!("  hudi run script.hud          # Run script (compile + VM)");
                println!("  hudi repl                                # Start REPL");
                println!("  hudi check script.hud        # Check syntax");
                println!("  hudi check --ast script.hud  # Show AST");
                println!("  hudi format script.hud       # Format code");
                println!("  hudi format -w script.hud    # Format and write back");
                println!("  hudi debug script.hud        # Interactive debugger");
                println!("  hudi dap script.hud          # DAP server (IDE integration)");
                println!("  hudi version                 # Show version");
                println!("  hudi info                    # System info");
            }
        }
    }
}
