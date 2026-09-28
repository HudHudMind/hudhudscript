//! Handler for `hudhud run` command with TOML runtime resolution.

use std::path::{Path, PathBuf};
use std::process;

use hudhudscript_cli::common::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_run_command(
    file: PathBuf,
    debug: bool,
    watch: bool,
    ui: Option<String>,
    strict: bool,
    gc_stats: bool,
    timing: bool,
    #[cfg(feature = "telemetry")] telemetry_json: Option<PathBuf>,
    engine: Option<String>,
    backend: Option<String>,
    jit_stats: bool,
    fallback_engine: Option<String>,
    config_path: Option<&Path>,
    verbose: bool,
) {
    let config = load_hudhud_config_with_path(debug || verbose, config_path);

    if let Err(e) = validate_config(&config) {
        eprintln!("Error in configuration: {e}");
        process::exit(1);
    }

    let jit_stats = jit_stats
        || config
            .runtime
            .jit
            .as_ref()
            .and_then(|j| j.stats)
            .unwrap_or(false);

    let engine = engine
        .or_else(|| config.runtime.engine.clone())
        .unwrap_or_else(|| "vm".to_string());

    let cli_backend = backend.clone();
    let backend = backend
        .or_else(|| config.runtime.jit.as_ref().and_then(|j| j.backend.clone()))
        .or_else(|| config.runtime.backend.clone())
        .unwrap_or_else(|| "auto".to_string());

    let fallback_engine = fallback_engine
        .or_else(|| config.runtime.jit.as_ref().and_then(|j| j.fallback.clone()))
        .unwrap_or_else(|| "none".to_string());

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
    if engine != "jit" && cli_backend.is_some() {
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
                // M4: göreli importlar dosyanın dizinine göre çözümlenir
                if let Some(dir) = file.parent() {
                    rt.set_base_dir(dir);
                }
                if let Some(jit_cfg) = config.runtime.jit.as_ref() {
                    if let Some(ref opt_lvl) = jit_cfg.opt_level {
                        rt.set_opt_level_str(opt_lvl);
                    }
                    if let Some(ref opt_g) = jit_cfg.opt_goal {
                        rt.set_opt_goal_str(opt_g);
                    }
                    if let Some(rounds) = jit_cfg.mir_opt_rounds {
                        rt.set_mir_opt_rounds(rounds);
                    }
                    if let Some(ref pol) = jit_cfg.policy {
                        rt.set_policy(pol);
                    }
                    if let Some(ht) = jit_cfg.hot_threshold {
                        rt.set_hot_threshold(ht);
                    }
                    if let Some(lt) = jit_cfg.loop_threshold {
                        rt.set_loop_threshold(lt);
                    }
                    if let Some(c) = jit_cfg.cache {
                        rt.set_cache_enabled(c);
                    }
                    if let Some(mb) = jit_cfg.code_cache_mb {
                        rt.set_code_cache_mb(mb);
                    }
                    if let Some(v) = jit_cfg.verify_with_vm {
                        rt.set_verify_with_vm(v);
                    }
                }
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
    let telemetry_json: Option<PathBuf> = None;
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
            watch_and_run_with_config(&file, debug || verbose, config_path, timing)
        {
            eprintln!("{}", render_error(&e));
            process::exit(e.exit_code());
        }
    } else if let Some(framework) = ui {
        if let Err(e) =
            run_ui_with_config(&file, &framework, debug || verbose, config_path)
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
            debug || verbose,
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
