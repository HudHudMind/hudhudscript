//! Machine-readable JSON report (CI-friendly, group-structured).

use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::runner::SuiteOutcome;

#[derive(Serialize)]
struct JsonReport<'a> {
    generator: &'a str,
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    duration_ms: u128,
    exit_ok: bool,
    coverage_threshold_failed: bool,
    files: Vec<JsonFile<'a>>,
    coverage: Option<&'a crate::runner::CoverageData>,
}

#[derive(Serialize)]
struct JsonFile<'a> {
    path: String,
    group: &'a str,
    error: Option<&'a str>,
    tests: Vec<JsonTest<'a>>,
}

#[derive(Serialize)]
struct JsonTest<'a> {
    name: &'a str,
    status: &'static str,
    message: Option<&'a str>,
    output: Option<&'a str>,
    duration_ms: f64,
    groups: &'a [String],
    line: usize,
}

/// Render the report as pretty-printed JSON.
pub fn render(outcome: &SuiteOutcome) -> String {
    let files = outcome
        .files
        .iter()
        .map(|file| JsonFile {
            path: file.path.display().to_string(),
            group: &file.group,
            error: file.error.as_deref(),
            tests: file
                .tests
                .iter()
                .map(|test| {
                    let (status, message) = match &test.outcome {
                        crate::runner::Outcome::Passed => ("passed", None),
                        crate::runner::Outcome::Skipped => ("skipped", None),
                        crate::runner::Outcome::Failed(message) => ("failed", Some(message.as_str())),
                    };
                    JsonTest {
                        name: &test.name,
                        status,
                        message,
                        output: test.output.as_deref(),
                        duration_ms: test.duration.as_secs_f64() * 1000.0,
                        groups: &test.groups,
                        line: test.line,
                    }
                })
                .collect(),
        })
        .collect();
    let report = JsonReport {
        generator: "hudunit",
        total: outcome.tests().count(),
        passed: outcome.passed(),
        failed: outcome.failed_count(),
        skipped: outcome.skipped(),
        duration_ms: outcome.duration.as_millis(),
        exit_ok: outcome.exit_ok(),
        coverage_threshold_failed: outcome.coverage_threshold_failed,
        files,
        coverage: outcome.coverage.as_ref(),
    };
    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
}

/// Write the JSON report to `path`, creating parent directories as needed.
pub fn write_to(path: &Path, outcome: &SuiteOutcome) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    file.write_all(render(outcome).as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{FileOutcome, Outcome, TestOutcome};
    use std::time::Duration;

    #[test]
    fn json_contains_status_fields() {
        let suite = SuiteOutcome {
            files: vec![FileOutcome {
                path: std::path::PathBuf::from("/t/tests/test_a.hud"),
                group: "test_a".into(),
                tests: vec![TestOutcome {
                    name: "test_x".into(),
                    outcome: Outcome::Failed("assert_eq failed".into()),
                    duration: Duration::from_millis(1),
                    groups: vec![],
                    line: 2,
                    output: None,
                }],
                error: None,
                fail_fast_stopped: false,
            }],
            duration: Duration::from_millis(3),
            ..Default::default()
        };
        let text = render(&suite);
        assert!(text.contains("\"status\": \"failed\""));
        assert!(text.contains("\"message\": \"assert_eq failed\""));
        assert!(text.contains("\"exit_ok\": false"));
    }
}
