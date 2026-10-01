//! Clap command-line specification of the `hudhud` binary.
//!
//! Subcommands: run, compile, repl, format, lint, check, package.
//!
//! Relocated verbatim from the binary crate (`src/main.rs`) so the external
//! unit-test suite can render its help text; the binary imports it from the
//! library unchanged.

use std::path::PathBuf;

use clap::{Parser as ClapParser, Subcommand};

#[doc(hidden)]
#[derive(ClapParser)]
#[command(name = "hudhud")]
#[command(version, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[doc(hidden)]
#[derive(Subcommand)]
pub enum Commands {
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

        /// Execution engine: vm or jit (default: from hudhud.toml [runtime.engine] or "vm")
        #[arg(long)]
        engine: Option<String>,

        /// Native backend for --engine=jit: auto, cranelift, llvm, gccjit (default: from hudhud.toml or "auto")
        #[arg(long)]
        backend: Option<String>,

        /// Print JIT compilation stats to stderr (opt-in; default stays silent)
        #[arg(long)]
        jit_stats: bool,

        /// Fallback policy when JIT fails at RUNTIME: vm = restart in VM
        /// (side effects may repeat!); none = stop with honest error (default: none, or from hudhud.toml)
        #[arg(long)]
        fallback_engine: Option<String>,
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
