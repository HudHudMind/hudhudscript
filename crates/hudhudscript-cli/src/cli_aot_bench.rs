//! AOT compilation and engine benchmarking CLI handlers.
//! Split from cli_dispatch.rs to respect the 400-line source limit.

use hudhudscript_cli::common::*;
use std::process;

// ── Native AOT handlers (JIT_AOT_ARCHITECTURE.md §E compile/build) ──

#[cfg(feature = "aot")]
pub(crate) fn opt_level_of(n: u8) -> hudhudscript_aot::AotOptLevel {
    match n {
        0 => hudhudscript_aot::AotOptLevel::O0,
        1 => hudhudscript_aot::AotOptLevel::O1,
        2 => hudhudscript_aot::AotOptLevel::O2,
        _ => hudhudscript_aot::AotOptLevel::O3,
    }
}

#[cfg(feature = "aot")]
pub(crate) fn check_native_backend(backend: &str) {
    if !matches!(backend, "auto" | "cranelift" | "gccjit" | "llvm") {
        eprintln!(
            "Error: unknown backend `{backend}` — available: auto, cranelift, gccjit, llvm"
        );
        process::exit(1);
    }
}

#[cfg(feature = "aot")]
pub(crate) fn compile_native_object(
    file: &std::path::Path,
    output: Option<std::path::PathBuf>,
    backend: &str,
    target: &str,
    opt: u8,
    verbose: bool,
) {
    check_native_backend(backend);
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", file.display(), e);
            process::exit(1);
        }
    };
    let out = output.unwrap_or_else(|| file.with_extension("o"));
    match hudhudscript_aot::compile_object_only(&source, &out, opt_level_of(opt), target, backend) {
        Ok((path, symbols, entry, _entries)) => {
            if verbose {
                eprintln!("object: {}", path.display());
                eprintln!("entry: {entry}");
                for s in &symbols {
                    eprintln!("symbol: {s}");
                }
            } else {
                println!("{}", path.display());
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

#[cfg(feature = "aot")]
pub(crate) fn build_native_executable(
    file: &std::path::Path,
    output: Option<std::path::PathBuf>,
    backend: &str,
    target: &str,
    opt: u8,
    build_dir: Option<std::path::PathBuf>,
    keep_symbols: bool,
) {
    check_native_backend(backend);
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", file.display(), e);
            process::exit(1);
        }
    };
    let out = output.unwrap_or_else(|| {
        file.file_stem()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "hudhud_out".into())
    });
    let workdir = build_dir.unwrap_or_else(|| std::path::PathBuf::from("_build"));
    match hudhudscript_aot::build_executable(&source, &out, &workdir, opt_level_of(opt), target, backend) {
        Ok(r) => {
            if !keep_symbols {
                strip_executable(&r.executable);
            }
            println!("{} (entry: {}, linker: cc)", r.executable.display(), r.entry_symbol);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}

// ── Bench (JIT_AOT_ARCHITECTURE.md §E: bench --all-backends) ──

#[cfg(feature = "jit")]
pub(crate) fn silence_stdout() -> (i32, i32) {
    #[cfg(unix)]
    let (stdout_fd, devnull_path) = (libc::STDOUT_FILENO, b"/dev/null\0".as_ptr() as *const _);
    #[cfg(windows)]
    let (stdout_fd, devnull_path) = (1, b"NUL\0".as_ptr() as *const _);
    #[cfg(not(any(unix, windows)))]
    let (stdout_fd, devnull_path) = (1, b"/dev/null\0".as_ptr() as *const _);

    // Ölçüm sırasında print çıktısını /dev/null veya NUL'a yönlendir
    unsafe {
        let saved = libc::dup(stdout_fd);
        let devnull = libc::open(devnull_path, libc::O_WRONLY);
        libc::dup2(devnull, stdout_fd);
        libc::close(devnull);
        (saved, devnull)
    }
}

#[cfg(feature = "jit")]
pub(crate) fn restore_stdout(saved: i32) {
    #[cfg(unix)]
    let stdout_fd = libc::STDOUT_FILENO;
    #[cfg(not(unix))]
    let stdout_fd = 1;

    unsafe {
        libc::dup2(saved, stdout_fd);
        libc::close(saved);
    }
}

#[cfg(feature = "jit")]
pub(crate) fn bench_engines(file: &std::path::Path, iterations: u32, backends: Option<&[String]>) {
    let source = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", file.display(), e);
            process::exit(1);
        }
    };
    let want = |name: &str| backends.map(|b| b.iter().any(|x| x == name)).unwrap_or(true);
    let mut rows: Vec<(&str, f64, &str)> = Vec::new();

    // VM
    if want("vm") {
        let mut best = f64::MAX;
        for _ in 0..iterations {
            let saved = silence_stdout();
            let t = std::time::Instant::now();
            let tmp = std::env::temp_dir().join(format!("hudhud_bench_{}.hud", std::process::id()));
            std::fs::write(&tmp, &source).unwrap();
            let _ = run_file_vm(&tmp, false);
            let _ = std::fs::remove_file(&tmp);
            let el = t.elapsed().as_secs_f64();
            restore_stdout(saved.0);
            best = best.min(el);
        }
        rows.push(("vm", best, "interpreter"));
    }

    // JIT backends
    for (name, feats) in [("jit-cranelift", "cranelift"), ("jit-gccjit", "gccjit"), ("jit-llvm", "llvm")] {
        if !want(feats) && !want(name) {
            continue;
        }
        if let Ok(mut rt) = hudhudscript_jit::JitRuntime::with_backend(feats) {
            let saved0 = silence_stdout();
            let warm = rt.run(&source).is_ok();
            restore_stdout(saved0.0);
            if !warm {
                rows.push((name, f64::NAN, "unsupported-script"));
                continue;
            }
            let mut best = f64::MAX;
            for _ in 0..iterations {
                let saved = silence_stdout();
                let t = std::time::Instant::now();
                let _ = rt.run(&source);
                let el = t.elapsed().as_secs_f64();
                restore_stdout(saved.0);
                best = best.min(el);
            }
            rows.push((name, best, "compile+run"));
        }
    }

    // AOT (build + çalıştırma)
    #[cfg(feature = "aot")]
    {
        for (name, b) in [("aot-cranelift", "cranelift"), ("aot-gccjit", "gccjit"), ("aot-llvm", "llvm")] {
            if !want(b) && !want(name) {
                continue;
            }
            let workdir = std::path::PathBuf::from("_bench");
            let out = workdir.join(format!("bench_{b}"));
            let t0 = std::time::Instant::now();
            match hudhudscript_aot::build_executable(&source, &out, &workdir, hudhudscript_aot::AotOptLevel::O2, "native", b) {
                Ok(r) => {
                    let build_s = t0.elapsed().as_secs_f64();
                    let mut best = f64::MAX;
                    for _ in 0..iterations {
                        let saved = silence_stdout();
                        let t = std::time::Instant::now();
                        let _ = std::process::Command::new(&r.executable).output();
                        best = best.min(t.elapsed().as_secs_f64());
                        restore_stdout(saved.0);
                    }
                    let _ = std::fs::remove_file(&r.executable);
                    let _ = std::fs::remove_file(&r.object);
                    rows.push((name, best, Box::leak(format!("build {build_s:.3}s").into_boxed_str())));
                }
                Err(e) => rows.push((name, f64::NAN, Box::leak(e.into_boxed_str()))),
            }
        }
    }

    println!("{:<14} {:>12}  {}", "engine", "best (s)", "note");
    for (n, t, note) in &rows {
        if t.is_nan() {
            println!("{n:<14} {:>12}  {note}", "-");
        } else {
            println!("{n:<14} {t:>12.6}  {note}");
        }
    }
}

/// Üretim binary'sini küçült: sembol tablosunu at (4.8MB→1.2MB).
/// strip PATH'te yoksa sessiz atlanır (binary çalışır, sadece büyüktür).
#[cfg(feature = "aot")]
fn strip_executable(bin: &std::path::Path) {
    let strip = std::env::var_os("PATH").and_then(|p| {
        p.to_string_lossy()
            .split(':')
            .map(|d| std::path::PathBuf::from(d).join("strip"))
            .find(|c| c.is_file())
    });
    let Some(strip) = strip else { return };
    if let Ok(before) = std::fs::metadata(bin) {
        if std::process::Command::new(&strip).arg(bin).status().map(|s| s.success()).unwrap_or(false) {
            if let Ok(after) = std::fs::metadata(bin) {
                let b = before.len() as f64 / 1024.0 / 1024.0;
                let a = after.len() as f64 / 1024.0 / 1024.0;
                println!("strip: {b:.1}MB → {a:.1}MB");
            }
        }
    }
}
