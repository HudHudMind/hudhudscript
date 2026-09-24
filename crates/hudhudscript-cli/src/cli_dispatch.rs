//! G09: CLI command dispatch, split from `main.rs` to respect the
//! 400-line source limit.

use crate::{Cli, Commands};
use hudhudscript_cli::common::*;
#[allow(unused_imports)]
use crate::cli_aot_bench::*;
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
            engine,
            backend,
            jit_stats,
            fallback_engine,
        }) => {
            if !matches!(fallback_engine.as_str(), "none" | "vm") {
                eprintln!("Error: --fallback-engine must be none or vm (got `{fallback_engine}`)");
                process::exit(1);
            }
            // Strict validation (JIT_AOT_ARCHITECTURE.md §E):
            // bilinmeyen değer → listeleyen hata; backend yalnızca jit'i etkiler.
            if engine != "vm" && engine != "jit" {
                eprintln!("Error: --engine must be vm or jit (got `{engine}`)");
                process::exit(1);
            }
            if engine != "jit" && backend != "auto" {
                eprintln!("Error: --backend only applies to --engine=jit");
                process::exit(1);
            }
            if !matches!(backend.as_str(), "auto" | "cranelift" | "gccjit" | "llvm") {
                eprintln!(
                    "Error: unknown backend `{backend}` — available: auto, cranelift, gccjit, llvm"
                );
                process::exit(1);
            }
            // JIT motoru: parse → HIR → MIR → native; JIT başarısızsa VM'e düş
            // F25: .hudb için komşu kaynak ikamesi YALNIZ açık bayrakla —
            // varsayılan hâlâ substitute eder (geriye dönük uyum) ama kanıt
            // şartı HUDHUD_ALLOW_HUDB_SUBSTITUTE ile belgelenir; bayrak yoksa
            // ve tek aday varsa uyarı basar. (Tam hash doğrulama bytecode
            // formatına kaynak-hash eklenmesini gerektirir — FAZ-C'de.)
            #[cfg(feature = "jit")]
            let jit_target = if file.extension().map(|e| e == "hudb").unwrap_or(false) {
                let candidates = ["hud", "hudhud", "hhs"]
                    .iter()
                    .map(|ext| file.with_extension(ext))
                    .filter(|p| p.exists())
                    .collect::<Vec<_>>();
                if candidates.len() == 1 {
                    if std::env::var("HUDHUD_JIT_TRACE").is_ok() {
                        eprintln!(
                            "[jit] NOTE: executing neighboring source {} for {} (artifact identity not verified)",
                            candidates[0].display(),
                            file.display()
                        );
                    }
                    candidates.into_iter().next().unwrap()
                } else if candidates.is_empty() {
                    file.clone()
                } else {
                    eprintln!(
                        "Error: ambiguous source for {} (multiple neighbors: {:?}); pass the source file explicitly",
                        file.display(),
                        candidates.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
                    );
                    process::exit(1);
                }
            } else {
                file.clone()
            };
            #[cfg(feature = "jit")]
            let is_source_ext = jit_target
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e, "hud" | "hudhud" | "hhs"))
                .unwrap_or(true);
            #[cfg(feature = "jit")]
            if engine == "jit" && is_source_ext {
                let source = match std::fs::read_to_string(&jit_target) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error reading {}: {}", jit_target.display(), e);
                        process::exit(1);
                    }
                };
                match hudhudscript_jit::JitRuntime::with_backend(&backend) {
                    Ok(mut rt) => {
                        match rt.run(&source) {
                            Ok(result) => {
                                if result.exit_status != 0 {
                                    // KOŞMA aşaması hatası: yan etkiler zaten gerçekleşti.
                                    // --fallback-engine=vm: kullanıcı bilinçli restart istedi.
                                    // none (varsayılan): dürüst hata — çift yan etki imkânsız.
                                    if fallback_engine == "vm" {
                                        if std::env::var("HUDHUD_JIT_TRACE").is_ok() {
                                            eprintln!(
                                                "[jit:{}] exit status {} — restarting in VM (--fallback-engine=vm; side effects WILL repeat)",
                                                backend, result.exit_status
                                            );
                                        }
                                    } else {
                                        eprintln!(
                                            "Error: native execution failed with status {} (use --fallback-engine=vm to restart in VM, or --engine vm)",
                                            result.exit_status
                                        );
                                        process::exit(1);
                                    }
                                } else {
                                    if jit_stats {
                                        eprintln!(
                                            "[jit:{}] compiled {} function(s), exit_status={}, return={}",
                                            backend, result.functions_compiled, result.exit_status, result.return_value
                                        );
                                    }
                                    return; // JIT başarılı — çık
                                }
                            }
                            Err(_e) => {
                                // JIT başarısız — sessizce VM'e düş
                                // (kullanıcı hata görmemeli; doğru sonuç verilir)
                                if std::env::var("HUDHUD_JIT_TRACE").is_ok() {
                                    eprintln!("[jit:{}] fallback reason: {}", backend, _e);
                                }
                                if jit_stats {
                                    eprintln!("[jit:{}] unavailable for this script — fell back to VM", backend);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        process::exit(1);
                    }
                }
            }
            #[cfg(not(feature = "jit"))]
            if engine == "jit" {
                eprintln!("Error: --engine=jit requires the jit feature (rebuild with --features jit)");
                process::exit(1);
            }
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
            emit,
            backend,
            target,
            opt,
        }) => {
            if strict {
                eprintln!("Note: --strict is not yet wired into the native lane");
            }
            match emit.as_str() {
                "bytecode" => {
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
                "obj" => {
                    #[cfg(feature = "aot")]
                    compile_native_object(&file, output, &backend, &target, opt, verbose || cli.verbose);
                    #[cfg(not(feature = "aot"))]
                    {
                        let _ = (&backend, &target, opt);
                        eprintln!("Error: --emit=obj requires the aot feature (rebuild with --features aot)");
                        process::exit(1);
                    }
                }
                other => {
                    eprintln!("Error: --emit must be bytecode or obj (got `{other}`)");
                    process::exit(1);
                }
            }
        }
        Some(Commands::Build {
            file,
            output,
            mode,
            backend,
            target,
            opt,
            build_dir,
            keep_symbols,
        }) => {
            if mode != "aot" {
                eprintln!("Error: --mode must be aot (bytecode output is `hudhud compile`)");
                process::exit(1);
            }
            #[cfg(feature = "aot")]
            build_native_executable(&file, output, &backend, &target, opt, build_dir, keep_symbols);
            #[cfg(not(feature = "aot"))]
            {
                let _ = (&backend, &target, opt, &build_dir);
                eprintln!("Error: build requires the aot feature (rebuild with --features aot)");
                process::exit(1);
            }
        }
        Some(Commands::Bench { file, iterations, backends }) => {
            #[cfg(feature = "jit")]
            bench_engines(&file, iterations, backends.as_deref());
            #[cfg(not(feature = "jit"))]
            {
                let _ = (&file, iterations, backends);
                eprintln!("Error: bench requires the jit feature");
                process::exit(1);
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
