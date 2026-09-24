//! `hudunit` — the HudHudScript unit test runner CLI.
//!
//! Usage:
//!   hudunit [paths...] [options]
//!   hudunit init                # scaffold src/ + tests/ + hudunit.toml
//!
//! Options:
//!   -f, --filter <substr>      Run only tests whose name contains substr
//!   -g, --group <name>         Run only the given group (repeatable)
//!   -G, --exclude-group <name> Exclude the group (repeatable)
//!       --list                 List discovered tests without running
//!       --list-groups          List discovered groups with test counts
//!       --json <path>          Also write a JSON report
//!       --junit <path>         Also write a JUnit XML report (CI)
//!       --html <path>          Also write a self-contained HTML report
//!   -c, --coverage             Collect line+function coverage
//!   -v, --verbose              Show failure messages immediately
//!   -q, --quiet                Summary only
//!       --no-color             Disable colored output
//!   -w, --watch                Re-run on file changes (Ctrl-C to stop)
//!   -h, --help                 Show this help
//!   -V, --version              Show version

use std::path::PathBuf;
use std::process::ExitCode;

use hudhudscript_hudunit::config::HudunitConfig;
use hudhudscript_hudunit::discover::{collect_files, discover_file};
use hudhudscript_hudunit::groups;
use hudhudscript_hudunit::report::{console, html, json, junit};
use hudhudscript_hudunit::runner::{self, RunOptions};

const HELP: &str = "\
hudunit — HudHudScript unit test runner

USAGE:
    hudunit [paths...] [options]

Paths default to `tests/` (or `test/` when only that exists). Test files use
the .hhs / .hud / .hudhud extensions. Tests are `test_*` functions; helpers
`setup()` / `teardown()` run around each test; `ignore_test_*` is skipped.

Groups: every test belongs to a directory-derived group, plus `@group name`
comment annotations directly above the test function.

OPTIONS:
    -f, --filter <substr>       Filter tests by name substring
    -g, --group <name>          Select group (repeatable, any-of)
    -G, --exclude-group <name>  Exclude group (repeatable)
        --list                  List discovered tests and exit
        --list-groups           List discovered groups and exit
        --json <path>           Write a JSON report to path
        --junit <path>          Write a JUnit XML report to path (CI)
        --html <path>           Write an HTML report to path
    -c, --coverage              Collect line+function coverage
    -v, --verbose               Verbose failure output
    -q, --quiet                 Summary only
        --no-color              Disable ANSI colors
    -w, --watch                 Re-run on file changes (Ctrl-C to stop)
        --fail-fast            Stop at the first failing test
    -h, --help                  Print help
    -V, --version               Print version

EXAMPLES:
    hudunit init                  # scaffold a testable project (src/ + tests/)
    hudunit                       # run tests/ recursively
    hudunit tests/math            # run one group
    hudunit --group hizli         # run an annotated group
    hudunit --coverage --html hudunit-report.html
    hudunit --junit junit.xml     # CI-ready test results
";

struct Cli {
    paths: Vec<PathBuf>,
    filter: Option<String>,
    include_groups: Vec<String>,
    exclude_groups: Vec<String>,
    list: bool,
    list_groups: bool,
    json: Option<PathBuf>,
    junit: Option<PathBuf>,
    html: Option<PathBuf>,
    init: bool,
    coverage: bool,
    verbose: bool,
    quiet: bool,
    no_color: bool,
    watch: bool,
    fail_fast: bool,
}

fn parse_args(args: &[String]) -> Result<Cli, String> {
    let mut cli = Cli {
        paths: Vec::new(),
        filter: None,
        include_groups: Vec::new(),
        exclude_groups: Vec::new(),
        list: false,
        list_groups: false,
        json: None,
        junit: None,
        html: None,
        init: false,
        coverage: false,
        verbose: false,
        quiet: false,
        no_color: false,
        watch: false,
        fail_fast: false,
    };
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        let mut take_value = |long: &str| -> Result<String, String> {
            iter.next()
                .cloned()
                .ok_or_else(|| format!("{} requires a value", long))
        };
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{}", HELP);
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("hudunit {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-f" | "--filter" => cli.filter = Some(take_value("--filter")?),
            "-g" | "--group" => cli.include_groups.push(take_value("--group")?),
            "-G" | "--exclude-group" => {
                cli.exclude_groups.push(take_value("--exclude-group")?)
            }
            "--list" => cli.list = true,
            "--list-groups" => cli.list_groups = true,
            "--json" => cli.json = Some(PathBuf::from(take_value("--json")?)),
            "--junit" => cli.junit = Some(PathBuf::from(take_value("--junit")?)),
            "--html" => cli.html = Some(PathBuf::from(take_value("--html")?)),
            "init" => cli.init = true,
            "-c" | "--coverage" => cli.coverage = true,
            "-v" | "--verbose" => cli.verbose = true,
            "-q" | "--quiet" => cli.quiet = true,
            "--no-color" => cli.no_color = true,
            "-w" | "--watch" => cli.watch = true,
            "--fail-fast" => cli.fail_fast = true,
            other if other.starts_with('-') => {
                return Err(format!("unknown option: {}", other));
            }
            other => cli.paths.push(PathBuf::from(other)),
        }
    }
    Ok(cli)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match parse_args(&args) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("hudunit: {}", message);
            eprintln!("Run `hudunit --help` for usage.");
            return ExitCode::from(2);
        }
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cfg = HudunitConfig::load(&cwd);

    // `hudunit init`: scaffold src/ + tests/ + hudunit.toml and exit.
    if cli.init {
        return match hudhudscript_hudunit::init::init(&cwd) {
            Ok(report) => {
                for rel in &report.created {
                    println!("oluşturuldu  {}", rel);
                }
                for rel in &report.skipped {
                    println!("atlandı    {} (zaten var)", rel);
                }
                println!("\nhudunit    # testleri koşturmak için");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("hudunit: init failed: {}", e);
                ExitCode::FAILURE
            }
        };
    }

    // Default paths: tests/ (or test/), else the config's test_dir.
    let paths: Vec<PathBuf> = if cli.paths.is_empty() {
        if PathBuf::from("tests").is_dir() {
            vec![PathBuf::from("tests")]
        } else if PathBuf::from("test").is_dir() {
            vec![PathBuf::from("test")]
        } else if cfg.test_dir.is_dir() {
            vec![cfg.test_dir.clone()]
        } else {
            eprintln!(
                "hudunit: no test directory found (looked for tests/, test/, {})",
                cfg.test_dir.display()
            );
            return ExitCode::from(2);
        }
    } else {
        cli.paths.clone()
    };

    // One full pass: discover → (list or run) → report.
    let run_once = || -> bool {
        let files = collect_files(&paths, &cfg);
        if files.is_empty() {
            eprintln!("hudunit: no test files found under the given paths");
            return false;
        }

        let discovered: Vec<_> = match files
            .iter()
            .map(|path| discover_file(path, common_root(&paths).as_deref()))
            .collect::<Result<_, _>>()
        {
            Ok(discovered) => discovered,
            Err(e) => {
                eprintln!("hudunit: {}", e);
                return false;
            }
        };

        if cli.list_groups {
            for (group, count) in groups::group_summary(&discovered) {
                println!("{:>4}  {}", count, group);
            }
            return true;
        }

        if cli.list {
            for file in &discovered {
                for test in &file.tests {
                    let marker = if test.skip { "s" } else { "-" };
                    let tags = groups::groups_of(file, test).join(",");
                    println!("{} {:>4}  {} [{}]", marker, test.line, test.name, tags);
                }
            }
            return true;
        }

        let opts = RunOptions {
            filter: cli.filter.clone(),
            include_groups: cli.include_groups.clone(),
            exclude_groups: cli.exclude_groups.clone(),
            coverage: cli.coverage,
            verbose: cli.verbose,
            quiet: cli.quiet,
            fail_fast: cli.fail_fast,
        };
        let outcome = runner::run(&discovered, &cfg, &opts);

        let color = !cli.no_color && std::env::var_os("NO_COLOR").is_none();
        console::ConsoleReporter::new(color, cli.verbose, cli.quiet).report(&outcome);

        if let Some(path) = &cli.json {
            if let Err(e) = json::write_to(path, &outcome) {
                eprintln!("hudunit: cannot write JSON report {}: {}", path.display(), e);
            }
        }
        if let Some(path) = &cli.junit {
            if let Err(e) = junit::write_to(path, &outcome) {
                eprintln!("hudunit: cannot write JUnit report {}: {}", path.display(), e);
            }
        }
        if let Some(path) = &cli.html {
            if let Err(e) = html::write_to(path, &outcome) {
                eprintln!("hudunit: cannot write HTML report {}: {}", path.display(), e);
            }
        }
        outcome.exit_ok()
    };

    if !cli.watch {
        return if run_once() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    // Watch mode: re-run whenever a watched file changes. Watched set =
    // the scanned test paths plus `src/` (imported code) plus hudunit.toml.
    let mut watch_roots = paths.clone();
    if PathBuf::from("src").is_dir() {
        watch_roots.push(PathBuf::from("src"));
    }
    let mut snapshot = watch_snapshot(&watch_roots, &cfg);
    println!(
        "hudunit --watch: {} dosya izleniyor (Ctrl-C ile çıkış)",
        snapshot.len()
    );
    if !run_once() {
        // A failing first run is fine in watch mode; keep watching.
    }
    loop {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let current = watch_snapshot(&watch_roots, &cfg);
        if current != snapshot {
            let changed: Vec<String> = current
                .iter()
                .filter_map(|(p, t)| match snapshot.get(p) {
                    None => Some(p.display().to_string()),
                    Some(old) if old != t => Some(p.display().to_string()),
                    Some(_) => None,
                })
                .collect();
            println!("\x1b[2J\x1b[H→ değişiklik: {} — yeniden koşuluyor", changed.join(", "));
            snapshot = current;
            run_once();
        }
    }
}

/// mtime fingerprint of every watched file: path → modified time. A changed
/// mtime, a new file, or a removed file all produce a different snapshot.
fn watch_snapshot(roots: &[PathBuf], cfg: &HudunitConfig) -> std::collections::HashMap<PathBuf, std::time::SystemTime> {
    let mut out = std::collections::HashMap::new();
    for root in roots {
        for file in collect_files(std::slice::from_ref(root), cfg) {
            if let Ok(meta) = std::fs::metadata(&file) {
                if let Ok(modified) = meta.modified() {
                    out.insert(file, modified);
                }
            }
        }
    }
    if let Ok(meta) = std::fs::metadata("hudunit.toml") {
        if let Ok(modified) = meta.modified() {
            out.insert(PathBuf::from("hudunit.toml"), modified);
        }
    }
    out
}

/// Common parent directory of the scanned paths (for group derivation).
fn common_root(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.len() == 1 && paths[0].is_dir() {
        return Some(paths[0].clone());
    }
    paths.iter().find(|p| p.is_dir()).cloned()
}
