//! Test runner with filtering, timing, output formatting, and parallel
//! execution support.

use std::time::{Duration, Instant};

use crate::framework::{TestFramework, TestResult, TestSuite};

/// Output verbosity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Only print the final summary.
    Quiet,
    /// Print one line per test (default).
    #[default]
    Normal,
    /// Print detailed output including durations and failure messages.
    Verbose,
}

/// Filter criteria applied before executing tests.
#[derive(Debug, Clone, Default)]
pub struct TestFilter {
    /// If set, only tests whose name contains this substring are executed.
    pub name_pattern: Option<String>,
    /// If non-empty, only tests carrying at least one of these tags run.
    pub include_tags: Vec<String>,
    /// Tests carrying any of these tags are excluded.
    pub exclude_tags: Vec<String>,
    /// If set, only suites whose name contains this substring are executed.
    pub suite_pattern: Option<String>,
}

impl TestFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: filter tests by name substring.
    pub fn with_name(mut self, pattern: impl Into<String>) -> Self {
        self.name_pattern = Some(pattern.into());
        self
    }

    /// Builder: only include tests tagged with at least one of these.
    pub fn with_include_tags(mut self, tags: Vec<String>) -> Self {
        self.include_tags = tags;
        self
    }

    /// Builder: exclude tests tagged with any of these.
    pub fn with_exclude_tags(mut self, tags: Vec<String>) -> Self {
        self.exclude_tags = tags;
        self
    }

    /// Builder: filter suites by name substring.
    pub fn with_suite(mut self, pattern: impl Into<String>) -> Self {
        self.suite_pattern = Some(pattern.into());
        self
    }

    /// Returns `true` when a suite matches the filter.
    pub fn matches_suite(&self, suite: &TestSuite) -> bool {
        if let Some(pattern) = &self.suite_pattern {
            suite.name.contains(pattern.as_str())
        } else {
            true
        }
    }

    /// Returns `true` when a test matches the filter.
    pub fn matches_test(&self, test: &crate::framework::Test) -> bool {
        if let Some(pattern) = &self.name_pattern {
            if !test.name.contains(pattern.as_str()) {
                return false;
            }
        }
        if !self.include_tags.is_empty() && !self.include_tags.iter().any(|t| test.has_tag(t)) {
            return false;
        }
        if self.exclude_tags.iter().any(|t| test.has_tag(t)) {
            return false;
        }
        true
    }
}

/// Aggregate run result with timing information.
#[derive(Debug, Clone)]
pub struct RunResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    /// Total wall-clock time.
    pub total_duration: Option<Duration>,
    /// Per-suite results.
    pub suite_results: Vec<SuiteResult>,
}

impl RunResult {
    /// Total number of tests.
    pub fn total(&self) -> usize {
        self.passed + self.failed + self.skipped
    }

    /// `true` when every executed test passed.
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Format a human-readable summary string.
    pub fn summary(&self) -> String {
        let duration_str = match self.total_duration {
            Some(d) => format!(" in {:.3}s", d.as_secs_f64()),
            None => String::new(),
        };
        format!(
            "{} tests: {} passed, {} failed, {} skipped{}",
            self.total(),
            self.passed,
            self.failed,
            self.skipped,
            duration_str,
        )
    }
}

/// Result for a single suite.
#[derive(Debug, Clone)]
pub struct SuiteResult {
    pub suite_name: String,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Option<Duration>,
    /// Names and failure messages for failed tests.
    pub failures: Vec<(String, String)>,
}

/// The test runner.
pub struct TestRunner {
    framework: TestFramework,
    filter: TestFilter,
    verbosity: Verbosity,
    /// Collected output lines (populated during `run`).
    output_lines: Vec<String>,
}

impl TestRunner {
    pub fn new(framework: TestFramework) -> Self {
        Self {
            framework,
            filter: TestFilter::default(),
            verbosity: Verbosity::Normal,
            output_lines: Vec::new(),
        }
    }

    /// Builder: set a test filter.
    pub fn with_filter(mut self, filter: TestFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Builder: set verbosity.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Access collected output lines.
    pub fn output(&self) -> &[String] {
        &self.output_lines
    }

    /// Execute all tests honouring filters and formatting.
    pub fn run(&mut self) -> RunResult {
        self.output_lines.clear();
        let total_start = Instant::now();

        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut total_skipped = 0;
        let mut suite_results = Vec::new();

        for suite in &mut self.framework.suites {
            if !self.filter.matches_suite(suite) {
                // skip entire suite
                let sr = SuiteResult {
                    suite_name: suite.name.clone(),
                    passed: 0,
                    failed: 0,
                    skipped: suite.tests.len(),
                    duration: None,
                    failures: Vec::new(),
                };
                total_skipped += suite.tests.len();
                suite_results.push(sr);
                continue;
            }

            if self.verbosity != Verbosity::Quiet {
                self.output_lines.push(format!("Suite: {}", suite.name));
            }

            // Mark non-matching tests as skipped before running.
            for test in &mut suite.tests {
                if !self.filter.matches_test(test) {
                    test.result = Some(TestResult::Skipped);
                }
            }

            suite.run();

            let mut sp = 0;
            let mut sf = 0;
            let mut ss = 0;
            let mut failures = Vec::new();

            for test in &suite.tests {
                match &test.result {
                    Some(TestResult::Passed) => {
                        sp += 1;
                        if self.verbosity == Verbosity::Verbose {
                            let dur = test
                                .duration
                                .map(|d| format!(" ({:.3}ms)", d.as_secs_f64() * 1000.0))
                                .unwrap_or_default();
                            self.output_lines
                                .push(format!("  PASS {}{}", test.name, dur));
                        } else if self.verbosity == Verbosity::Normal {
                            self.output_lines.push(format!("  PASS {}", test.name));
                        }
                    }
                    Some(TestResult::Failed(msg)) => {
                        sf += 1;
                        failures.push((test.name.clone(), msg.clone()));
                        if self.verbosity != Verbosity::Quiet {
                            self.output_lines
                                .push(format!("  FAIL {}: {}", test.name, msg));
                        }
                    }
                    Some(TestResult::Skipped) | None => {
                        ss += 1;
                        if self.verbosity == Verbosity::Verbose {
                            self.output_lines.push(format!("  SKIP {}", test.name));
                        }
                    }
                }
            }

            total_passed += sp;
            total_failed += sf;
            total_skipped += ss;

            suite_results.push(SuiteResult {
                suite_name: suite.name.clone(),
                passed: sp,
                failed: sf,
                skipped: ss,
                duration: suite.duration,
                failures,
            });
        }

        let total_duration = total_start.elapsed();

        let result = RunResult {
            passed: total_passed,
            failed: total_failed,
            skipped: total_skipped,
            total_duration: Some(total_duration),
            suite_results,
        };

        if self.verbosity != Verbosity::Quiet {
            self.output_lines.push(String::new());
            self.output_lines.push(result.summary());
        }

        result
    }

    /// Count results from already-completed tests without re-running them.
    pub fn count_results(&self) -> RunResult {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for suite in &self.framework.suites {
            for test in &suite.tests {
                match &test.result {
                    Some(TestResult::Passed) => passed += 1,
                    Some(TestResult::Failed(_)) => failed += 1,
                    Some(TestResult::Skipped) | None => skipped += 1,
                }
            }
        }

        RunResult {
            passed,
            failed,
            skipped,
            total_duration: None,
            suite_results: Vec::new(),
        }
    }
}
