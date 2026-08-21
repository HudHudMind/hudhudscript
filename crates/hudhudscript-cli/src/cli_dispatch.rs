//! G09: CLI command dispatch, split from `main.rs` to respect the
//! 400-line source limit.

use crate::{Cli, Commands};
use hudhudscript_cli::common::*;
use std::process;

pub(crate) fn run_cli(cli: Cli) {
    // Set up logging based on verbose flag
    if cli.verbose {
        eprintln!("Verbose mode enabled");
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
                if let Err(e) =
                    watch_and_run_with_config(&file, debug || cli.verbose, config_path, timing)
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
                run_file_vm_with_config(
                    &file,
                    debug || cli.verbose,
                    config_path,
                    timing,
                    telemetry_json.as_deref(),
                )
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
            if let Err(e) = compile_file(
                &file,
                output,
                verbose || cli.verbose,
                strict,
                &Default::default(),
            ) {
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
            eprintln!(
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
