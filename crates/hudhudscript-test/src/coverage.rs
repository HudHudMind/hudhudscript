//! Coverage tracking structures for HudHudScript tests.
//!
//! These types are **data-only** — they model coverage information that the
//! interpreter/VM collects during test execution. The actual instrumentation
//! lives in those crates; this module provides the shared vocabulary types.

use std::collections::HashMap;

/// Coverage information for a single source file.
#[derive(Debug, Clone)]
pub struct FileCoverage {
    /// Absolute or relative path of the source file.
    pub path: String,
    /// Total number of executable lines.
    pub total_lines: usize,
    /// Number of lines that were executed at least once.
    pub covered_lines: usize,
    /// Per-line execution count (1-indexed line number -> count).
    pub line_hits: HashMap<usize, usize>,
    /// Total number of functions defined in this file.
    pub total_functions: usize,
    /// Number of functions that were entered at least once.
    pub covered_functions: usize,
}

impl FileCoverage {
    /// Create a new empty coverage record for the given file.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            total_lines: 0,
            covered_lines: 0,
            line_hits: HashMap::new(),
            total_functions: 0,
            covered_functions: 0,
        }
    }

    /// Line coverage percentage (0.0 – 100.0). Returns 0.0 when there are no
    /// executable lines.
    pub fn line_coverage_percent(&self) -> f64 {
        if self.total_lines == 0 {
            return 0.0;
        }
        (self.covered_lines as f64 / self.total_lines as f64) * 100.0
    }

    /// Function coverage percentage (0.0 – 100.0).
    pub fn function_coverage_percent(&self) -> f64 {
        if self.total_functions == 0 {
            return 0.0;
        }
        (self.covered_functions as f64 / self.total_functions as f64) * 100.0
    }

    /// Record a hit on the given line number.
    pub fn record_line_hit(&mut self, line: usize) {
        let counter = self.line_hits.entry(line).or_insert(0);
        if *counter == 0 {
            self.covered_lines += 1;
        }
        *counter += 1;
    }
}

/// Aggregate coverage report spanning multiple files.
#[derive(Debug, Clone)]
pub struct CoverageReport {
    /// Per-file coverage data.
    pub files: Vec<FileCoverage>,
    /// Optional minimum coverage threshold (0.0 – 100.0).
    pub threshold: Option<f64>,
}

impl Default for CoverageReport {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageReport {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            threshold: None,
        }
    }

    /// Set a minimum line-coverage threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Add coverage data for a file.
    pub fn add_file(&mut self, file: FileCoverage) {
        self.files.push(file);
    }

    /// Total executable lines across all files.
    pub fn total_lines(&self) -> usize {
        self.files.iter().map(|f| f.total_lines).sum()
    }

    /// Total covered lines across all files.
    pub fn covered_lines(&self) -> usize {
        self.files.iter().map(|f| f.covered_lines).sum()
    }

    /// Overall line coverage percentage.
    pub fn overall_line_coverage(&self) -> f64 {
        let total = self.total_lines();
        if total == 0 {
            return 0.0;
        }
        (self.covered_lines() as f64 / total as f64) * 100.0
    }

    /// Total functions across all files.
    pub fn total_functions(&self) -> usize {
        self.files.iter().map(|f| f.total_functions).sum()
    }

    /// Covered functions across all files.
    pub fn covered_functions(&self) -> usize {
        self.files.iter().map(|f| f.covered_functions).sum()
    }

    /// Overall function coverage percentage.
    pub fn overall_function_coverage(&self) -> f64 {
        let total = self.total_functions();
        if total == 0 {
            return 0.0;
        }
        (self.covered_functions() as f64 / total as f64) * 100.0
    }

    /// Check whether the overall line coverage meets the configured threshold.
    /// Returns `Ok(())` when no threshold is set or coverage is sufficient.
    pub fn check_threshold(&self) -> Result<(), String> {
        if let Some(threshold) = self.threshold {
            let actual = self.overall_line_coverage();
            if actual < threshold {
                return Err(format!(
                    "Coverage {:.1}% is below threshold {:.1}%",
                    actual, threshold
                ));
            }
        }
        Ok(())
    }

    /// Produce a human-readable summary.
    pub fn summary(&self) -> String {
        let mut lines = vec![format!(
            "Coverage: {:.1}% lines ({}/{}), {:.1}% functions ({}/{})",
            self.overall_line_coverage(),
            self.covered_lines(),
            self.total_lines(),
            self.overall_function_coverage(),
            self.covered_functions(),
            self.total_functions(),
        )];

        for file in &self.files {
            lines.push(format!(
                "  {}: {:.1}% lines, {:.1}% functions",
                file.path,
                file.line_coverage_percent(),
                file.function_coverage_percent(),
            ));
        }

        if let Some(threshold) = self.threshold {
            let status = if self.overall_line_coverage() >= threshold {
                "PASS"
            } else {
                "FAIL"
            };
            lines.push(format!("Threshold: {:.1}% — {}", threshold, status));
        }

        lines.join("\n")
    }
}
