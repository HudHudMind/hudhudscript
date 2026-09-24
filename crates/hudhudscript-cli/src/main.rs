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

mod cli_aot_bench;
mod cli_dispatch;
mod startup;

#[derive(ClapParser)]
#[command(name = "hudhud")]
#[command(version, long_about = None)]
pub(crate) struct Cli {
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
pub(crate) enum Commands {
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

        /// Execution engine: vm (default) or jit
        #[arg(long, default_value = "vm")]
        engine: String,

        /// Native backend for --engine=jit: auto, cranelift (llvm/gccjit arrive later)
        #[arg(long, default_value = "auto")]
        backend: String,

        /// Print JIT compilation stats to stderr (opt-in; default stays silent)
        #[arg(long)]
        jit_stats: bool,

        /// Fallback policy when JIT fails at RUNTIME: vm = restart in VM
        /// (side effects may repeat!); none = stop with honest error (default)
        #[arg(long, default_value = "none")]
        fallback_engine: String,
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

    /// Compile a HudHudScript file to bytecode or native object
    Compile {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output file path (defaults to input with .hudb/.o extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show detailed compilation info
        #[arg(short, long)]
        verbose: bool,

        /// Enable strict type checking (Issue #866 TYPE-001)
        #[arg(long)]
        strict: bool,

        /// Emission target: bytecode (default) or obj (native object, AOT)
        #[arg(long, default_value = "bytecode")]
        emit: String,

        /// Cross-compilation target triple (e.g. aarch64-unknown-linux-gnu)
        #[arg(long, default_value = "native")]
        target: String,

        /// Native backend for --emit=obj: auto, cranelift
        #[arg(long, default_value = "auto")]
        backend: String,

        /// Optimization level for native emission: 0, 1, 2, 3
        #[arg(long, default_value = "2")]
        opt: u8,
    },

    /// Build a native executable from a HudHudScript file (AOT)
    Build {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output executable path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Build mode: aot (native executable); bytecode is `compile`
        #[arg(long, default_value = "aot")]
        mode: String,

        /// Native backend: auto, cranelift
        #[arg(long, default_value = "auto")]
        backend: String,

        /// Cross-compilation target triple (native or e.g. aarch64-unknown-linux-gnu)
        #[arg(long, default_value = "native")]
        target: String,

        /// Optimization level: 0, 1, 2, 3
        #[arg(long, default_value = "2")]
        opt: u8,

        /// Build directory for intermediate artifacts (default: ./_build)
        #[arg(long)]
        build_dir: Option<PathBuf>,

        /// Sembol tablosunu koru (varsayılan: strip — 4.8MB→1.2MB)
        #[arg(long)]
        keep_symbols: bool,
    },

    /// Benchmark execution engines and native backends on a script
    Bench {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Repetitions per engine (default 3, best-of reported)
        #[arg(long, default_value = "3")]
        iterations: u32,

        /// Comma-separated backends (default: all compiled-in)
        #[arg(long, value_delimiter = ',')]
        backends: Option<Vec<String>>,
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
    let config = load_hudhud_config_with_path(cli.verbose, cli.config.as_deref());
    let env_value = std::env::var("HUDHUD_THREAD_STACK_MB").ok();
    let selected =
        startup::resolve_thread_stack_mb(env_value.as_deref(), config.runtime.thread_stack_mb)
            .unwrap_or_else(|error| {
                eprintln!("Error: {}", error);
                std::process::exit(2);
            });

    match selected {
        Some(stack_mb) => startup::run_with_stack(stack_mb, move || cli_dispatch::run_cli(cli))
            .unwrap_or_else(|error| {
                eprintln!("Error: {}", error);
                std::process::exit(1);
            }),
        None => cli_dispatch::run_cli(cli),
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn help_does_not_reduce_hudhudscript_to_mcp_orchestration() {
        let help = Cli::command().render_help().to_string();
        assert!(!help.contains("MCP-based orchestration language"));
    }
}
