//! Test execution model. Each test runs on a FRESH VM (full stdlib + the
//! coverage-aware module resolver), so tests are isolated from each other
//! exactly like phpunit/pytest instances. Assertions come from the script
//! prelude (see [`crate::builtins`]); failures surface as runtime errors
//! from `VM::call_public`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::{register_vm_stdlib_modules, VM};

use crate::config::HudunitConfig;
use crate::coverage::{CoverageResolver, Marks, MarkMap, function_coverage, instrument, summarize};
use crate::{builtins::PRELUDE, discover::TestFile, groups};

/// Outcome of a single test.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Passed,
    Failed(String),
    Skipped,
}

#[derive(Debug, Clone)]
pub struct TestOutcome {
    pub name: String,
    pub outcome: Outcome,
    pub duration: Duration,
    pub groups: Vec<String>,
    /// 1-based declaration line in the original source.
    pub line: usize,
    /// Captured stdout of the test (`print` output), shown for failing
    /// tests and under `--verbose`.
    pub output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileOutcome {
    pub path: PathBuf,
    /// Display group header (deepest path segment or file stem).
    pub group: String,
    pub tests: Vec<TestOutcome>,
    /// Parse/compile-level failure that prevented running any test.
    pub error: Option<String>,
    /// True when `--fail-fast` stopped the run mid-file.
    pub fail_fast_stopped: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SuiteOutcome {
    pub files: Vec<FileOutcome>,
    pub duration: Duration,
    pub coverage: Option<CoverageData>,
    /// True when the configured coverage threshold was not met.
    pub coverage_threshold_failed: bool,
    /// True when `--fail-fast` stopped the whole run at the first failure.
    pub aborted: bool,
}

impl SuiteOutcome {
    pub fn passed(&self) -> usize {
        self.count(|o| o == &Outcome::Passed)
    }
    pub fn failed_count(&self) -> usize {
        self.count(|o| matches!(o, Outcome::Failed(_)))
    }
    pub fn skipped(&self) -> usize {
        self.count(|o| o == &Outcome::Skipped)
    }
    fn count(&self, matches: impl Fn(&Outcome) -> bool + Copy) -> usize {
        self.tests().filter(|t| matches(&t.outcome)).count()
    }
    pub fn tests(&self) -> impl Iterator<Item = &TestOutcome> {
        self.files.iter().flat_map(|f| f.tests.iter())
    }
    pub fn exit_ok(&self) -> bool {
        self.failed_count() == 0
            && self.files.iter().all(|f| f.error.is_none())
            && !self.coverage_threshold_failed
    }
}

/// Aggregated coverage produced after a run with `coverage: true`.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CoverageData {
    pub files: Vec<crate::coverage::FileCoverage>,
    pub functions: Vec<FunctionCoverageEntry>,
    pub overall_line_percent: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FunctionCoverageEntry {
    pub name: String,
    pub file: String,
    pub covered: bool,
}

/// All CLI-selectable run options.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub filter: Option<String>,
    pub include_groups: Vec<String>,
    pub exclude_groups: Vec<String>,
    pub coverage: bool,
    pub verbose: bool,
    pub quiet: bool,
    /// Stop at the first failing test (remaining tests are not reported).
    pub fail_fast: bool,
}

/// Shared per-run state for module instrumentation caching.
pub type ModuleCache = Arc<Mutex<std::collections::HashMap<PathBuf, String>>>;

/// Run the discovered test files.
pub fn run(files: &[TestFile], cfg: &HudunitConfig, opts: &RunOptions) -> SuiteOutcome {
    let started = Instant::now();
    let marks = Marks::new();
    let mark_map = Arc::new(Mutex::new(MarkMap::new()));
    // One instrumentation cache shared by every VM in this run: a module
    // imported by several test files keeps a single set of mark ids.
    let module_cache: ModuleCache = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let mut file_outcomes = Vec::new();
    let mut files_stopped = false;

    for file in files {
        let outcome = run_file(file, cfg, opts, &marks, &mark_map, &module_cache);
        let stopped = outcome.fail_fast_stopped;
        file_outcomes.push(outcome);
        if stopped && opts.fail_fast {
            files_stopped = true;
            break;
        }
    }

    let coverage = if opts.coverage {
        let map = mark_map.lock().unwrap();
        let file_cov = summarize(&map, &marks);
        let functions = function_coverage(&map, &marks)
            .into_iter()
            .map(|(name, file, covered)| FunctionCoverageEntry { name, file, covered })
            .collect();
        let overall = overall_line_percent(&file_cov);
        Some(CoverageData {
            files: file_cov,
            functions,
            overall_line_percent: overall,
        })
    } else {
        None
    };

    let coverage_threshold_failed = match (&coverage, cfg.coverage_threshold) {
        (Some(data), Some(threshold)) => data.overall_line_percent < threshold,
        _ => false,
    };

    SuiteOutcome {
        files: file_outcomes,
        duration: started.elapsed(),
        coverage,
        coverage_threshold_failed,
        aborted: opts.fail_fast && files_stopped,
    }
}

fn overall_line_percent(files: &[crate::coverage::FileCoverage]) -> f64 {
    let executable: usize = files.iter().map(|f| f.executable_lines.len()).sum();
    if executable == 0 {
        return 100.0;
    }
    let covered: usize = files.iter().map(|f| f.covered_lines.len()).sum();
    100.0 * covered as f64 / executable as f64
}

fn run_file(
    file: &TestFile,
    cfg: &HudunitConfig,
    opts: &RunOptions,
    marks: &Marks,
    mark_map: &Arc<Mutex<MarkMap>>,
    module_cache: &ModuleCache,
) -> FileOutcome {
    let group = groups::display_group(file);
    let source = match std::fs::read_to_string(&file.path) {
        Ok(source) => source,
        Err(e) => {
            return FileOutcome {
                path: file.path.clone(),
                group,
                tests: Vec::new(),
                error: Some(format!("cannot read file: {}", e)),
            fail_fast_stopped: false,
            }
        }
    };

    let selected: Vec<_> = file
        .tests
        .iter()
        .filter(|t| test_selected(file, t, opts))
        .collect();
    if selected.is_empty() {
        return FileOutcome {
            path: file.path.clone(),
            group,
            tests: Vec::new(),
            error: None,
        fail_fast_stopped: false,
        };
    }

    // User AST parsed from its own source (spans reference the real file),
    // optionally instrumented, then composed with the prelude AST.
    let mut ast = match parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            return FileOutcome {
                path: file.path.clone(),
                group,
                tests: Vec::new(),
                error: Some(format!("parse error: {:?}", e)),
            fail_fast_stopped: false,
            }
        }
    };
    if opts.coverage {
        let mut map = mark_map.lock().unwrap();
        instrument(&mut ast, &file.path, &mut map);
    }
    let mut program = parse(PRELUDE).unwrap_or_default();
    program.append(&mut ast);

    let mut compiler = Compiler::new();
    let bytecode = match compiler.compile(&program) {
        Ok(bytecode) => bytecode,
        Err(e) => {
            return FileOutcome {
                path: file.path.clone(),
                group,
                tests: Vec::new(),
                error: Some(format!("compile error: {:?}", e)),
                fail_fast_stopped: false,
            }
        }
    };

    let mut results: Vec<TestOutcome> = Vec::new();
    for test in selected {
        if test.skip {
            results.push(TestOutcome {
                name: test.name.clone(),
                outcome: Outcome::Skipped,
                duration: Duration::ZERO,
                groups: groups::groups_of(file, test),
                line: test.line,
                output: None,
            });
            continue;
        }

        let started = Instant::now();
        let (mut outcome, captured) = crate::capture::capture_prints(|| {
            execute_isolated(
                &file.path,
                &bytecode,
                file,
                cfg,
                opts,
                marks,
                mark_map,
                module_cache,
                &test.name,
            )
        });
        let mut duration = started.elapsed();

        // Wall-clock guard: fuel stops infinite loops; this flags slow-but-
        // finite tests that exceed the configured timeout.
        if cfg.timeout_ms > 0 && duration > Duration::from_millis(cfg.timeout_ms) {
            duration = Duration::from_millis(cfg.timeout_ms);
            outcome = Outcome::Failed(format!(
                "timeout: exceeded {} ms",
                cfg.timeout_ms
            ));
        }
        let output = if captured.trim().is_empty() {
            None
        } else {
            Some(captured)
        };
        results.push(TestOutcome {
            name: test.name.clone(),
            outcome,
            duration,
            groups: groups::groups_of(file, test),
            line: test.line,
            output,
        });

        if opts.fail_fast && matches!(results.last().map(|t| &t.outcome), Some(Outcome::Failed(_)))
        {
            return FileOutcome {
                path: file.path.clone(),
                group,
                tests: results,
                error: None,
                fail_fast_stopped: true,
            };
        }
    }

    FileOutcome {
        path: file.path.clone(),
        group,
        tests: results,
        error: None,
        fail_fast_stopped: false,
    }
}

fn test_selected(
    file: &TestFile,
    test: &crate::discover::DiscoveredTest,
    opts: &RunOptions,
) -> bool {
    if !groups::selected(file, test, &opts.include_groups, &opts.exclude_groups) {
        return false;
    }
    match &opts.filter {
        Some(pattern) => test.name.contains(pattern.as_str()),
        None => true,
    }
}

fn execute_isolated(
    path: &Path,
    bytecode: &hudhudscript_bytecode::Bytecode,
    file: &TestFile,
    cfg: &HudunitConfig,
    opts: &RunOptions,
    marks: &Marks,
    mark_map: &Arc<Mutex<MarkMap>>,
    module_cache: &ModuleCache,
    test_name: &str,
) -> Outcome {
    let mut vm = VM::new();
    register_vm_stdlib_modules(&mut vm);
    if let Some(fuel) = cfg.fuel {
        vm.with_fuel(fuel);
    }
    if let Some(parent) = path.parent() {
        vm.set_module_resolver(Box::new(CoverageResolver::new(
            opts.coverage,
            Arc::clone(mark_map),
            Arc::clone(module_cache),
            parent.to_path_buf(),
        )));
    }

    if let Err(e) = vm.execute(bytecode) {
        return Outcome::Failed(format!("load error: {}", render_error(&e)));
    }
    if file.has_setup {
        if let Err(e) = vm.call_public("setup", &[], bytecode) {
            return Outcome::Failed(format!("setup failed: {}", render_error(&e)));
        }
    }
    let result = vm.call_public(test_name, &[], bytecode);
    if file.has_teardown {
        if let Err(e) = vm.call_public("teardown", &[], bytecode) {
            // Teardown failure is reported but does not mask the test result.
            return match result {
                Ok(_) => Outcome::Failed(format!("teardown failed: {}", render_error(&e))),
                Err(test_err) => Outcome::Failed(format!(
                    "{} (teardown also failed: {})",
                    render_error(&test_err),
                    render_error(&e)
                )),
            };
        }
    }
    // Drain this test's coverage marks before the VM is dropped.
    if let Some(seen) = vm.get_variable("__hudunit_seen").and_then(|v| v.as_string()) {
        marks.absorb_seen(&seen);
    }
    match result {
        Ok(_) => Outcome::Passed,
        Err(e) => Outcome::Failed(render_error(&e)),
    }
}

fn render_error(e: &hudhudscript_errors::Error) -> String {
    format!("{}", e)
}
