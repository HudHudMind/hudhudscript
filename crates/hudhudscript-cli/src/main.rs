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

use std::process;

use clap::Parser as ClapParser;
use hudhudscript_cli::common::cli_spec::Cli;
use hudhudscript_cli::common::*;

mod cli_aot_bench;
mod cli_dispatch;
mod cli_run;

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
